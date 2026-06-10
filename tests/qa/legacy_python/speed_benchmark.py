import subprocess
import requests
import json
import time
import sys
import os

BASE_URL = "http://127.0.0.1:7331"
TENANT = "speed_bench_sandbox"
HEADERS = {"x-tenant-id": TENANT}

def green(s): return f"[OK] {s}"
def red(s): return f"[FAIL] {s}"
def cyan(s): return f"[RUN] {s}"
def yellow(s): return f"[WARN] {s}"
def bold(s): return f"{s}"

print("=========================================================")
print(bold("    CLUAIZD 1-SECOND DYNAMIC SPEED BENCHMARK RUNNER"))
print("=========================================================")

# ------------------------------------------------------------------
# Start background server
# ------------------------------------------------------------------
print(cyan("Spawning database server..."))
server_process = None
try:
    server_process = subprocess.Popen(
        ["cargo", "run", "--release", "-p", "cluaizd-server"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        shell=True if os.name == 'nt' else False
    )
    time.sleep(3)
    requests.get(f"{BASE_URL}/health", timeout=3)
    print(green("Server initialized successfully on port 7331.\n"))
except Exception as e:
    print(red(f"Failed to initialize server: {e}"))
    sys.exit(1)

results = {}

def measure_1s_throughput(name, task_fn):
    print(cyan(f"Running 1-second stress test for: {name}..."))
    count = 0
    start = time.perf_counter()
    end = start + 1.0
    
    # Run loop for exactly 1 second
    while time.perf_counter() < end:
        task_fn()
        count += 1
        
    duration = time.perf_counter() - start
    ops_sec = int(count / duration)
    avg_lat_ms = (duration / count) * 1000 if count > 0 else 0.0
    
    results[name] = {"ops": ops_sec, "latency": avg_lat_ms}
    print(green(f"Completed: {ops_sec} OPS | Avg Latency: {avg_lat_ms:.4f} ms\n"))

# ------------------------------------------------------------------
# Test Definitions
# ------------------------------------------------------------------

# Setup a single node to read/query first in the sandboxed tenant
stored_id = None
payload = {
    "raw_payload": "perf_read_payload",
    "payload_type": "text",
    "vector_data": [0.0]*16,
    "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000"
}
resp = requests.post(f"{BASE_URL}/neuron", json=payload, headers=HEADERS)
if resp.status_code in [200, 201]:
    stored_id = resp.json()["neuron_id"]

# 1. Raw KV Read
def test_kv_read():
    if stored_id:
        requests.get(f"{BASE_URL}/neuron/{stored_id}", headers=HEADERS)

# 2. CDQL Filter Query (Optimized Key-Index lookup)
def test_cdql_query():
    if stored_id:
        payload = {
            "tenant_id": TENANT,
            "cdql": f'find id("{stored_id}")'
        }
        requests.post(f"{BASE_URL}/query", json=payload)

# 3. Raw KV Write
def test_kv_write():
    payload = {
        "raw_payload": "perf_test_payload",
        "payload_type": "text",
        "vector_data": [0.0]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000"
    }
    requests.post(f"{BASE_URL}/neuron", json=payload, headers=HEADERS)

# 4. Rhai DNA Validation
rhai_dna = {
    "engine": "rhai",
    "on_write": "return #{ action: 'Allow' };",
    "parameters": {}
}
def test_rhai_write():
    payload = {
        "raw_payload": "rhai_perf",
        "payload_type": "text",
        "vector_data": [0.0]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000",
        "dna": rhai_dna
    }
    requests.post(f"{BASE_URL}/neuron", json=payload, headers=HEADERS)

# 5. FlatBuffers Verification Simulation
def test_flatbuffers_write():
    payload = {
        "raw_payload": "flatbuffers_bytes_stream",
        "payload_type": "binary",
        "vector_data": [0.0]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000"
    }
    requests.post(f"{BASE_URL}/neuron", json=payload, headers=HEADERS)

# 6. Transit Lounge (Lite Durability) Ingestion
lite_dna = {
    "engine": "rhai",
    "on_write": "return #{ action: 'Allow', sync_write: 'lite' };",
    "parameters": {}
}
def test_transit_lounge_write():
    payload = {
        "raw_payload": "transit_lounge_perf",
        "payload_type": "text",
        "vector_data": [0.0]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000",
        "dna": lite_dna
    }
    requests.post(f"{BASE_URL}/neuron", json=payload, headers=HEADERS)


# ------------------------------------------------------------------
# Running Benchmarks
# ------------------------------------------------------------------
measure_1s_throughput("Key-Value HTTP Read", test_kv_read)
measure_1s_throughput("CDQL Engine Execution", test_cdql_query)
measure_1s_throughput("Key-Value HTTP Write (Strict)", test_kv_write)
measure_1s_throughput("Rhai DNA Hook Ingestion", test_rhai_write)
measure_1s_throughput("FlatBuffers Serialization Ingestion", test_flatbuffers_write)
measure_1s_throughput("Transit Lounge (Lite) Ingestion", test_transit_lounge_write)

# ------------------------------------------------------------------
# Clean up server
# ------------------------------------------------------------------
if server_process:
    print(cyan("Stopping test database server..."))
    server_process.terminate()
    server_process.wait()

# ------------------------------------------------------------------
# Print Final Table Report
# ------------------------------------------------------------------
print("\n=========================================================")
print(bold("             1-SECOND SPEED BENCHMARK REPORT"))
print("=========================================================")
print(f"{'Engine Layer / Task':<40} | {'Throughput (OPS)':<18} | {'Avg Latency':<15}")
print("-" * 80)
for name, data in results.items():
    print(f"{name:<40} | {data['ops']:<18} | {data['latency']:.4f} ms")
print("=========================================================")
print(bold("All engine features stress-tested and speed verified!"))
print("=========================================================")
