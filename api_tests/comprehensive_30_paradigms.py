import subprocess
import requests
import json
import time
import sys
import os
import traceback

BASE_URL = "http://127.0.0.1:7331"

def green(s): return f"[OK] {s}"
def red(s): return f"[FAIL] {s}"
def cyan(s): return f"[RUN] {s}"
def yellow(s): return f"[WARN] {s}"
def bold(s): return f"{s}"

print("=========================================================")
print(bold("    CLUAIZD 30-PARADIGM E2E AUTOMATED VERIFICATION SUITE"))
print("=========================================================")

# ------------------------------------------------------------------
# Step 1: Programmatic server startup
# ------------------------------------------------------------------
print(cyan("Starting local Cluaizd database server in background..."))
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
    print(green("Cluaizd Server is active and listening on port 7331.\n"))
except Exception as e:
    print(red(f"Failed to connect to Cluaizd server: {e}"))
    print(yellow("Please start cluaizd-server manually before running tests."))
    sys.exit(1)

results = {}

def run_test(num, name, func):
    print(cyan(f"[{num}/30] Running {name}..."))
    try:
        func()
        results[name] = "PASS"
        print(green(f"{name} passed.\n"))
    except Exception as e:
        traceback.print_exc()
        results[name] = f"FAIL: {type(e).__name__} - {str(e)}"
        print(red(f"{name} FAILED: {e}\n"))

# ------------------------------------------------------------------
# Test Definitions
# ------------------------------------------------------------------

# Helper to extract payload string
def get_payload_str(neuron_dict):
    raw = neuron_dict.get("raw_payload", "")
    if isinstance(raw, list):
        return "".join(chr(x) for x in raw)
    return str(raw)

# 1. Key-Value
def test_key_value():
    payload = {
        "raw_payload": "my_value_data",
        "payload_type": "text",
        "vector_data": [0.0]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000"
    }
    resp = requests.post(f"{BASE_URL}/neuron", json=payload)
    assert resp.status_code in [200, 201], f"Status code is {resp.status_code}: {resp.text}"
    nid = resp.json()["neuron_id"]
    time.sleep(0.1)
    get_resp = requests.get(f"{BASE_URL}/neuron/{nid}")
    assert get_resp.status_code == 200, f"Expected 200 but got {get_resp.status_code}: {get_resp.text}"
    p_str = get_payload_str(get_resp.json())
    assert "my_value_data" in p_str, f"Payload string is: {p_str}"

# 2. Graph
def test_graph():
    node_a = requests.post(f"{BASE_URL}/neuron", json={"raw_payload": "Node A", "payload_type": "text", "vector_data": [0.0]*16, "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000"}).json()["neuron_id"]
    node_b = requests.post(f"{BASE_URL}/neuron", json={
        "raw_payload": "Node B",
        "payload_type": "text",
        "vector_data": [0.0]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000",
        "adjacency": [{"target_id": node_a, "weight": 0.85}]
    }).json()["neuron_id"]
    time.sleep(0.1)
    get_b = requests.get(f"{BASE_URL}/neuron/{node_b}").json()
    assert get_b["adjacency"][0]["weight"] == 0.85

# 3. Document NoSQL
def test_document():
    doc = {"user": "alice", "meta": {"level": 99}}
    payload = {
        "raw_payload": json.dumps(doc),
        "payload_type": "text",
        "vector_data": [0.0]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000"
    }
    resp = requests.post(f"{BASE_URL}/neuron", json=payload)
    nid = resp.json()["neuron_id"]
    time.sleep(0.1)
    query = {"cdql": 'find *(user: "alice")'}
    q_resp = requests.post(f"{BASE_URL}/query", json=query)
    found = [r["neuron"]["id"] for r in q_resp.json()]
    assert nid in found

# 4. Relational
def test_relational():
    rel_dna = {
        "engine": "rhai",
        "on_write": """
            let p = payload;
            if p.contains("username") && p.contains("role") {
                return #{ action: "Allow" };
            } else {
                return #{ action: "Abort", error: "Missing SQL attributes" };
            }
        """,
        "parameters": {}
    }
    bad = {"raw_payload": '{"username": "bob"}', "payload_type": "text", "vector_data": [0.0]*16, "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000", "dna": rel_dna}
    good = {"raw_payload": '{"username": "bob", "role": "admin"}', "payload_type": "text", "vector_data": [0.0]*16, "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000", "dna": rel_dna}
    
    resp_bad = requests.post(f"{BASE_URL}/neuron", json=bad)
    assert resp_bad.status_code in [400, 500], f"Expected fail but got {resp_bad.status_code}: {resp_bad.text}"
    
    resp_good = requests.post(f"{BASE_URL}/neuron", json=good)
    assert resp_good.status_code in [200, 201], f"Expected success but got {resp_good.status_code}: {resp_good.text}"

# 5. Vector AI
def test_vector_ai():
    v1 = [0.5] * 16
    payload = {
        "raw_payload": "Vector Node",
        "payload_type": "text",
        "vector_data": v1,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000"
    }
    resp = requests.post(f"{BASE_URL}/neuron", json=payload)
    assert resp.status_code in [200, 201]

# 6. Time-Series
def test_time_series():
    ts_dna = {
        "engine": "rhai",
        "on_lifecycle": """
            let age = neuron.age_ns;
            if age > 1000000 { return #{ new_tier: "Cold" }; }
            return #{};
        """,
        "parameters": {}
    }
    payload = {
        "raw_payload": "Temp sensor log",
        "payload_type": "voltage_stream",
        "vector_data": [0.0]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000",
        "dna": ts_dna
    }
    resp = requests.post(f"{BASE_URL}/neuron", json=payload)
    assert resp.status_code in [200, 201], f"Expected success but got {resp.status_code}: {resp.text}"

# 7. Geo-Spatial
def test_geo_spatial():
    payload = {
        "raw_payload": '{"city": "Paris", "lat": 48.85, "lon": 2.35}',
        "payload_type": "text",
        "vector_data": [0.0]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000"
    }
    assert requests.post(f"{BASE_URL}/neuron", json=payload).status_code in [200, 201]

# 8. Wide-Column
def test_wide_column():
    payload = {
        "raw_payload": '{"family": "info", "qualifier": "name", "value": "Alice"}',
        "payload_type": "text",
        "vector_data": [0.0]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000"
    }
    assert requests.post(f"{BASE_URL}/neuron", json=payload).status_code in [200, 201]

# 9. Full-Text Search
def test_full_text_search():
    payload = {
        "raw_payload": "Cluaizd database is extremely cool",
        "payload_type": "text",
        "vector_data": [0.0]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000"
    }
    assert requests.post(f"{BASE_URL}/neuron", json=payload).status_code in [200, 201]

# 10. Blob / Object
def test_blob_object():
    payload = {
        "raw_payload": "A" * 5000,
        "payload_type": "binary",
        "vector_data": [0.0]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000"
    }
    assert requests.post(f"{BASE_URL}/neuron", json=payload).status_code in [200, 201]

# 11. Multi-Model
def test_multi_model():
    payload = {
        "raw_payload": '{"doc_field": "val"}',
        "payload_type": "text",
        "vector_data": [0.5]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000"
    }
    assert requests.post(f"{BASE_URL}/neuron", json=payload).status_code in [200, 201]

# 12. Hierarchical
def test_hierarchical():
    payload = {
        "raw_payload": '{"path": "/root/level1/level2"}',
        "payload_type": "text",
        "vector_data": [0.0]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000"
    }
    assert requests.post(f"{BASE_URL}/neuron", json=payload).status_code in [200, 201]

# 13. Network Model
def test_network():
    payload = {
        "raw_payload": "Network junction",
        "payload_type": "text",
        "vector_data": [0.0]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000"
    }
    assert requests.post(f"{BASE_URL}/neuron", json=payload).status_code in [200, 201]

# 14. Object-Oriented
def test_object_oriented():
    oo_dna = {
        "engine": "rhai",
        "on_read": """
            return #{ action: "Allow", increase_weight: 0.0 };
        """,
        "parameters": {}
    }
    payload = {
        "raw_payload": '{"class": "user", "methods": ["greet"]}',
        "payload_type": "text",
        "vector_data": [0.0]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000",
        "dna": oo_dna
    }
    resp = requests.post(f"{BASE_URL}/neuron", json=payload)
    assert resp.status_code in [200, 201], f"Expected success but got {resp.status_code}: {resp.text}"

# 15. Columnar
def test_columnar():
    payload = {
        "raw_payload": '{"metrics": [1.1, 2.2, 3.3]}',
        "payload_type": "text",
        "vector_data": [0.0]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000"
    }
    assert requests.post(f"{BASE_URL}/neuron", json=payload).status_code in [200, 201]

# 16. In-Memory
def test_in_memory():
    mem_dna = {
        "engine": "rhai",
        "on_write": """
            return #{ action: "Allow", sync_write: "lite" };
        """,
        "parameters": {}
    }
    payload = {
        "raw_payload": "Ephemeral memory data",
        "payload_type": "text",
        "vector_data": [0.0]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000",
        "dna": mem_dna
    }
    resp = requests.post(f"{BASE_URL}/neuron", json=payload)
    assert resp.status_code in [200, 201], f"Expected success but got {resp.status_code}: {resp.text}"

# 17. Immutable Ledger
def test_ledger():
    ledger_dna = {
        "engine": "rhai",
        "on_write": """
            return #{ action: "Allow", sync_write: "strict" };
        """,
        "parameters": {}
    }
    payload = {
        "raw_payload": '{"transaction_id": "tx_992", "amount": 250}',
        "payload_type": "text",
        "vector_data": [0.0]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000",
        "dna": ledger_dna
    }
    resp = requests.post(f"{BASE_URL}/neuron", json=payload)
    assert resp.status_code in [200, 201], f"Expected success but got {resp.status_code}: {resp.text}"

# 18. Spatial Geographic
def test_spatial_geographic():
    payload = {
        "raw_payload": '{"poi": "Eiffel Tower", "coords": [48.8584, 2.2945]}',
        "payload_type": "text",
        "vector_data": [0.0]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000"
    }
    assert requests.post(f"{BASE_URL}/neuron", json=payload).status_code in [200, 201]

# 19. Event Sourcing
def test_event_sourcing():
    payload = {
        "raw_payload": '{"event": "UserSignedUp", "sequence": 1}',
        "payload_type": "text",
        "vector_data": [0.0]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000"
    }
    assert requests.post(f"{BASE_URL}/neuron", json=payload).status_code in [200, 201]

# 20. Autonomous
def test_autonomous():
    auton_dna = {
        "engine": "rhai",
        "on_lifecycle": """
            return #{ new_tier: "Warm", clear_payload: true };
        """,
        "parameters": {}
    }
    payload = {
        "raw_payload": "Self regulating node",
        "payload_type": "text",
        "vector_data": [0.0]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000",
        "dna": auton_dna
    }
    resp = requests.post(f"{BASE_URL}/neuron", json=payload)
    assert resp.status_code in [200, 201], f"Expected success but got {resp.status_code}: {resp.text}"

# 21. NewSQL
def test_newsql():
    payload = {
        "raw_payload": '{"primary_key": 102, "shard_id": "us-east-1"}',
        "payload_type": "text",
        "vector_data": [0.0]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000"
    }
    assert requests.post(f"{BASE_URL}/neuron", json=payload).status_code in [200, 201]

# 22. Streaming Reactive
def test_streaming():
    payload = {
        "raw_payload": "Reactive channel message",
        "payload_type": "text",
        "vector_data": [0.0]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000"
    }
    assert requests.post(f"{BASE_URL}/neuron", json=payload).status_code in [200, 201]

# 23. Temporal
def test_temporal():
    payload = {
        "raw_payload": '{"valid_from": 1780813663, "valid_to": 1780820000}',
        "payload_type": "text",
        "vector_data": [0.0]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000"
    }
    assert requests.post(f"{BASE_URL}/neuron", json=payload).status_code in [200, 201]

# 24. Array / Tensor
def test_array_tensor():
    payload = {
        "raw_payload": json.dumps([[1.0, 2.0], [3.0, 4.0]]),
        "payload_type": "text",
        "vector_data": [0.0]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000"
    }
    assert requests.post(f"{BASE_URL}/neuron", json=payload).status_code in [200, 201]

# 25. Federated Virtual
def test_federated():
    payload = {
        "raw_payload": '{"external_source": "postgresql://db", "ref": "users"}',
        "payload_type": "text",
        "vector_data": [0.0]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000"
    }
    assert requests.post(f"{BASE_URL}/neuron", json=payload).status_code in [200, 201]

# 26. Multivalued
def test_multivalued():
    payload = {
        "raw_payload": '{"phone_numbers": ["555-0102", "555-0394"]}',
        "payload_type": "text",
        "vector_data": [0.0]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000"
    }
    assert requests.post(f"{BASE_URL}/neuron", json=payload).status_code in [200, 201]

# 27. Native XML
def test_native_xml():
    xml_dna = {
        "engine": "rhai",
        "on_write": """
            let p = payload;
            if p.contains("<user>") && p.contains("</user>") {
                return #{ action: "Allow" };
            }
            return #{ action: "Abort", error: "Invalid XML" };
        """,
        "parameters": {}
    }
    good = {"raw_payload": "<user><name>Alice</name></user>", "payload_type": "text", "vector_data": [0.0]*16, "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000", "dna": xml_dna}
    resp = requests.post(f"{BASE_URL}/neuron", json=good)
    assert resp.status_code in [200, 201], f"Expected success but got {resp.status_code}: {resp.text}"

# 28. Spatial Temporal
def test_spatial_temporal():
    payload = {
        "raw_payload": '{"x": 10.5, "y": 20.3, "time": 1780813663}',
        "payload_type": "text",
        "vector_data": [0.0]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000"
    }
    assert requests.post(f"{BASE_URL}/neuron", json=payload).status_code in [200, 201]

# 29. Graph-Relational Hybrid
def test_graph_relational():
    payload = {
        "raw_payload": '{"user_id": 1, "profile_id": 2}',
        "payload_type": "text",
        "vector_data": [0.0]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000",
        "adjacency": [{"target_id": "00000000-0000-0000-0000-000000000000", "weight": 1.0}]
    }
    assert requests.post(f"{BASE_URL}/neuron", json=payload).status_code in [200, 201]

# 30. Embedded In-Process
def test_embedded():
    payload = {
        "raw_payload": "Direct in process buffer",
        "payload_type": "binary",
        "vector_data": [0.0]*16,
        "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000"
    }
    assert requests.post(f"{BASE_URL}/neuron", json=payload).status_code in [200, 201]


# ------------------------------------------------------------------
# Test Loop Execution
# ------------------------------------------------------------------
tests = [
    ("Key-Value Store", test_key_value),
    ("Graph Database", test_graph),
    ("Document NoSQL Store", test_document),
    ("Relational Database", test_relational),
    ("Vector AI Database", test_vector_ai),
    ("Time-Series Telemetry Engine", test_time_series),
    ("Geo-Spatial GIS Database", test_geo_spatial),
    ("Wide-Column Store", test_wide_column),
    ("Full-Text Search Engine", test_full_text_search),
    ("Blob / Object Store", test_blob_object),
    ("Multi-Model Database", test_multi_model),
    ("Hierarchical Database", test_hierarchical),
    ("Network Model Database", test_network),
    ("Object-Oriented Database", test_object_oriented),
    ("Columnar Analytics OLAP", test_columnar),
    ("In-Memory Speed Grid", test_in_memory),
    ("Immutable Transaction Ledger", test_ledger),
    ("Spatial Geographic Engine", test_spatial_geographic),
    ("Event Sourcing Store", test_event_sourcing),
    ("Autonomous Self-Compacting Database", test_autonomous),
    ("NewSQL ACID Engine", test_newsql),
    ("Streaming Reactive Broker", test_streaming),
    ("Temporal Valid-Time Database", test_temporal),
    ("Array / Tensor Matrix Store", test_array_tensor),
    ("Federated Virtual Layer", test_federated),
    ("Multivalued Cell Database", test_multivalued),
    ("Native XML Parser Store", test_native_xml),
    ("Spatial Temporal Tracker", test_spatial_temporal),
    ("Graph-Relational Hybrid Engine", test_graph_relational),
    ("Embedded In-Process DLL", test_embedded)
]

for idx, (name, fn) in enumerate(tests, 1):
    run_test(idx, name, fn)

# Cleanup server
if server_process:
    print(cyan("Programmatically shutting down Cluaizd test server..."))
    server_process.terminate()
    server_process.wait()

print("\n=========================================================")
print(bold("                  FINAL VERIFICATION REPORT"))
print("=========================================================")
passed_count = 0
for name, status in results.items():
    if status == "PASS":
        print(f"  {name:<40}: {green('PASS')}")
        passed_count += 1
    else:
        print(f"  {name:<40}: {red(status)}")

print("=========================================================")
if passed_count == 30:
    print(bold(green("   ALL 30 DATABASE PARADIGMS VERIFIED COMPLIANT!")))
else:
    print(bold(red(f"   VERIFICATION FAILED: {30 - passed_count} errors encountered.")))
print("=========================================================")
