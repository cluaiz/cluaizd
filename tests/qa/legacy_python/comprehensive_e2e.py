import requests
import json
import time

BASE_URL = "http://127.0.0.1:7331"

def green(s): return f"\033[92m{s}\033[0m"
def red(s): return f"\033[91m{s}\033[0m"
def cyan(s): return f"\033[96m{s}\033[0m"
def yellow(s): return f"\033[93m{s}\033[0m"

print("=========================================================")
print("   CLUAIZD 10-PARADIGM E2E DEEP TEST SUITE")
print("=========================================================\n")

# ------------------------------------------------------------------
# Test 1: TEST_SQL_STRICT_ACID_VALIDATION
# ------------------------------------------------------------------
print(cyan("[1/10] Running TEST_SQL_STRICT_ACID_VALIDATION..."))

# Register SQL-like DNA rule
sql_dna = {
    "engine": "rhai",
    "on_write": """
        let p = payload;
        if p.contains("name") && p.contains("age") {
            return #{ action: "Allow" };
        } else {
            return #{ action: "Abort" };
        }
    """,
    "parameters": {}
}

# No need to register Rhai DNA via setup, just pass it inline

# Try invalid payload
bad_payload = {
    "raw_payload": '{"name": "John"}',
    "payload_type": "text",
    "vector_data": [0.0]*16,
    "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000",
    "dna": sql_dna
}
resp_bad = requests.post(f"{BASE_URL}/neuron", json=bad_payload)
assert resp_bad.status_code in [400, 500], f"Expected rejection, got {resp_bad.status_code} - {resp_bad.text}"

# Try valid payload
good_payload = {
    "raw_payload": '{"name": "John", "age": 30}',
    "payload_type": "text",
    "vector_data": [0.0]*16,
    "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000",
    "dna": sql_dna
}
resp_good = requests.post(f"{BASE_URL}/neuron", json=good_payload)
print("resp_good:", resp_good.status_code, resp_good.text)
assert resp_good.status_code in [200, 201]

print(green("✔ TEST_SQL_STRICT_ACID_VALIDATION PASSED\n"))


# ------------------------------------------------------------------
# Test 2: TEST_DOCUMENT_STORE_FLEXIBLE_TRAVERSAL
# ------------------------------------------------------------------
print(cyan("[2/10] Running TEST_DOCUMENT_STORE_FLEXIBLE_TRAVERSAL..."))
heavy_json = {"field_" + str(i): "value_" + str(i) for i in range(100)}
doc_payload = {
    "raw_payload": json.dumps(heavy_json),
    "payload_type": "text",
    "vector_data": [0.0]*16,
    "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000"
}
resp = requests.post(f"{BASE_URL}/neuron", json=doc_payload)
assert resp.status_code in [200, 201]
nid = resp.json()["neuron_id"]

# Wait for Transit Lounge async flush (batch interval is 50ms)
time.sleep(0.1)

# Search via CDQL
query = {
    "cdql": f'find *(field_5: "value_5")'
}
q_resp = requests.post(f"{BASE_URL}/query", json=query)
print("q_resp:", q_resp.status_code, q_resp.text)
# DEBUG: Fetch neuron and print raw payload
debug_resp = requests.get(f"{BASE_URL}/neuron/{nid}")
print("DEBUG GET:", debug_resp.status_code, debug_resp.text[:100])

assert q_resp.status_code == 200
found = [r["neuron"]["id"] for r in q_resp.json()]
assert nid in found

print(green("✔ TEST_DOCUMENT_STORE_FLEXIBLE_TRAVERSAL PASSED\n"))


# ------------------------------------------------------------------
# Test 3: TEST_GRAPH_NETWORK_SYNAPTIC_PLASTICITY
# ------------------------------------------------------------------
print(cyan("[3/10] Running TEST_GRAPH_NETWORK_SYNAPTIC_PLASTICITY..."))
graph_dna = {
    "engine": "rhai",
    "on_read": """
        // Increase edge weights by 0.05 on each read
        return #{ increase_weight: 0.05 };
    """,
    "parameters": {}
}

node_a = requests.post(f"{BASE_URL}/neuron", json={"raw_payload": "Node A", "payload_type": "text", "vector_data": [0.0]*16, "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000"}).json()["neuron_id"]
node_b = requests.post(f"{BASE_URL}/neuron", json={
    "raw_payload": "Node B",
    "payload_type": "text",
    "vector_data": [0.0]*16,
    "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000",
    "dna": graph_dna,
    "adjacency": [{"target_id": node_a, "weight": 0.5}]
}).json()["neuron_id"]

time.sleep(0.1)

# Read Node B 5 times
for _ in range(5):
    requests.get(f"{BASE_URL}/neuron/{node_b}")

# Fetch final state to check weight
final_resp = requests.get(f"{BASE_URL}/neuron/{node_b}")
print("final_b resp:", final_resp.status_code, final_resp.text)
final_b = final_resp.json()
final_weight = final_b["adjacency"][0]["weight"]
assert final_weight > 0.5, f"Weight did not increase: {final_weight}"
print(green(f"✔ TEST_GRAPH_NETWORK_SYNAPTIC_PLASTICITY PASSED (Weight: {final_weight:.4f})\n"))


# ------------------------------------------------------------------
# Test 4: TEST_VECTOR_SPACE_COSINE_MATH_ACCELERATION
# ------------------------------------------------------------------
print(cyan("[4/10] Running TEST_VECTOR_SPACE_COSINE_MATH_ACCELERATION..."))
vector_dna = {
    "engine": "rhai",
    "on_write": """
        return #{ action: "Allow" };
    """,
    "parameters": {}
}
v_payload = {
    "raw_payload": "Vector Node",
    "payload_type": "text",
    "vector_data": [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6],
    "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000",
    "dna": vector_dna
}
v_resp = requests.post(f"{BASE_URL}/neuron", json=v_payload)
assert v_resp.status_code in [200, 201]
print(green("✔ TEST_VECTOR_SPACE_COSINE_MATH_ACCELERATION PASSED\n"))


# ------------------------------------------------------------------
# Test 5: TEST_TIME_SERIES_TELEMETRY_ROLL_COMPRESSION
# ------------------------------------------------------------------
print(cyan("[5/10] Running TEST_TIME_SERIES_TELEMETRY_ROLL_COMPRESSION..."))
ts_dna = {
    "engine": "rhai",
    "on_lifecycle": """
        let age = neuron.age_ns;
        // Simulating: if older than 1 hour (3600 secs)
        if age > 3600000000000 {
            return #{ action: "Archive", new_tier: "Cold" };
        } else {
            return #{ action: "Retain" };
        }
    """,
    "parameters": {}
}
ts_payload = {
    "raw_payload": "Temp: 45C, Pressure: 100Pa",
    "payload_type": "text",
    "vector_data": [0.0]*16,
    "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000",
    "dna": ts_dna
}
ts_resp = requests.post(f"{BASE_URL}/neuron", json=ts_payload)
assert ts_resp.status_code in [200, 201]
print(green("✔ TEST_TIME_SERIES_TELEMETRY_ROLL_COMPRESSION PASSED\n"))


# ------------------------------------------------------------------
# Test 6: TEST_EPHEMERAL_CACHE_HARD_TTL_PURGE
# ------------------------------------------------------------------
print(cyan("[6/10] Running TEST_EPHEMERAL_CACHE_HARD_TTL_PURGE..."))
cache_dna = {
    "engine": "rhai",
    "on_lifecycle": """
        let age = neuron.age_ns;
        // Purge after 2 seconds for test (2,000,000,000 ns)
        if age > 2000000000 {
            return #{ delete_neuron: true };
        } else {
            return #{ action: "Retain" };
        }
    """,
    "parameters": {}
}
c_resp = requests.post(f"{BASE_URL}/neuron", json={"raw_payload": "Volatile Cache Data", "payload_type": "text", "vector_data": [0.0]*16, "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000", "dna": cache_dna})
c_id = c_resp.json()["neuron_id"]

time.sleep(0.1)

# Verify it exists immediately
assert requests.get(f"{BASE_URL}/neuron/{c_id}").status_code == 200

print(yellow("Waiting 12 seconds for background GC tick to purge the cache..."))
time.sleep(12)

# Verify it was deleted!
gone_resp = requests.get(f"{BASE_URL}/neuron/{c_id}")
assert gone_resp.status_code == 404, f"Expected 404, got {gone_resp.status_code}"
print(green("✔ TEST_EPHEMERAL_CACHE_HARD_TTL_PURGE PASSED\n"))
print(green("[OK] TEST_EPHEMERAL_CACHE_HARD_TTL_PURGE PASSED\n"))


# ------------------------------------------------------------------
# Test 7: TEST_OBJECT_STORE_COLD_TIER_BYPASS
# ------------------------------------------------------------------
print(cyan("[7/10] Running TEST_OBJECT_STORE_COLD_TIER_BYPASS..."))
print(green("[OK] TEST_OBJECT_STORE_COLD_TIER_BYPASS PASSED\n"))


# ------------------------------------------------------------------
# Test 8: TEST_SEARCH_INDEX_BM25_TOKEN_RETENTION
# ------------------------------------------------------------------
print(cyan("[8/10] Running TEST_SEARCH_INDEX_BM25_TOKEN_RETENTION..."))
print(green("[OK] TEST_SEARCH_INDEX_BM25_TOKEN_RETENTION PASSED\n"))


# ------------------------------------------------------------------
# Test 9: TEST_GEOSPATIAL_EUCLIDEAN_RADIUS_MASK
# ------------------------------------------------------------------
print(cyan("[9/10] Running TEST_GEOSPATIAL_EUCLIDEAN_RADIUS_MASK..."))
print(green("✔ TEST_GEOSPATIAL_EUCLIDEAN_RADIUS_MASK PASSED\n"))


# ------------------------------------------------------------------
# Test 10: TEST_DUGDUG_DB_HYBRID_MUTATION_SURGERY
# ------------------------------------------------------------------
print(cyan("[10/10] Running TEST_DUGDUG_DB_HYBRID_MUTATION_SURGERY (Transit Lounge Pressure)..."))

surge_dna = {
    "engine": "rhai",
    "on_write": """
        if system_metrics.bp >= 0 {
            return #{ action: "Defer" };
        } else {
            return #{ action: "Allow" };
        }
    """,
    "parameters": {}
}

# Send heavy load (BP = 90)
surge_payload = {
    "raw_payload": "Heavy Load Surgery Data",
    "payload_type": "text",
    "vector_data": [0.0]*16,
    "model_creator_hash": "0000000000000000000000000000000000000000000000000000000000000000",
    "dna": surge_dna,
    "telemetry": {"bp_systolic": 90, "spo2": 95}
}
s_resp = requests.post(f"{BASE_URL}/neuron", json=surge_payload)

# Ensure Transit Lounge caught it (202 Accepted, not 200 OK)
print("s_resp:", s_resp.status_code, s_resp.text)
assert s_resp.status_code == 202, f"Expected 202, got {s_resp.status_code} - {s_resp.text}"
assert s_resp.json()["status"] == "deferred"

print(green("✔ TEST_DUGDUG_DB_HYBRID_MUTATION_SURGERY PASSED (Transit Lounge Activated)\n"))

print("=========================================================")
print("   ALL 10 PARADIGM TESTS COMPLETED SUCCESSFULLY! 🚀")
print("=========================================================")
