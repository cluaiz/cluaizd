import urllib.request
import json
import time

BASE_URL = "http://localhost:7331"

def test_write_neuron():
    print("Testing POST /neuron...")
    payload = {
        "raw_payload": "Hello from API Tests!",
        "vector_data": [0.1] * 16,
        "model_creator_hash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "payload_type": "text",
        "dna": None,
        "adjacency": None
    }
    
    req = urllib.request.Request(f"{BASE_URL}/neuron", method="POST")
    req.add_header("Content-Type", "application/json")
    
    try:
        data = json.dumps(payload).encode("utf-8")
        with urllib.request.urlopen(req, data=data) as response:
            res_data = json.loads(response.read().decode())
            print(f"SUCCESS: {res_data}")
            return res_data["neuron_id"]
    except Exception as e:
        print(f"FAILED: {e}")
        return None

def test_query_neuron(neuron_id):
    print("Testing POST /query (CNQL)...")
    payload = {
        "cnql": "find *"
    }
    
    req = urllib.request.Request(f"{BASE_URL}/query", method="POST")
    req.add_header("Content-Type", "application/json")
    
    try:
        data = json.dumps(payload).encode("utf-8")
        with urllib.request.urlopen(req, data=data) as response:
            res_data = json.loads(response.read().decode())
            found = any(n["neuron"]["id"] == neuron_id for n in res_data)
            if found:
                print(f"SUCCESS: Neuron {neuron_id} found in query results!")
            else:
                print(f"FAILED: Neuron {neuron_id} not found in query results.")
    except Exception as e:
        print(f"FAILED: {e}")

if __name__ == "__main__":
    print("Starting API Tests...")
    nid = test_write_neuron()
    if nid:
        time.sleep(1) # wait for WAL flush if any
        test_query_neuron(nid)
