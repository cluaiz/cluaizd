import urllib.request
import json
import time

BASE_URL = "http://localhost:7331"
HEADERS = {
    "Content-Type": "application/json",
    "x-tenant-id": "speculative_test_tenant"
}

def post_json(path, payload):
    req = urllib.request.Request(f"{BASE_URL}{path}", method="POST", headers=HEADERS)
    data = json.dumps(payload).encode("utf-8")
    try:
        with urllib.request.urlopen(req, data=data) as response:
            return json.loads(response.read().decode())
    except urllib.error.HTTPError as e:
        err_msg = e.read().decode()
        print(f"HTTP Error {e.code}: {err_msg}")
        raise

def get_json(path):
    req = urllib.request.Request(f"{BASE_URL}{path}", method="GET", headers=HEADERS)
    try:
        with urllib.request.urlopen(req) as response:
            return json.loads(response.read().decode())
    except urllib.error.HTTPError as e:
        err_msg = e.read().decode()
        print(f"HTTP Error {e.code}: {err_msg}")
        raise

def main():
    print("--- STARTING SPECULATIVE GRAPH ROUTING E2E TEST ---")
    
    # 1. Insert Node D (Target)
    print("\nInserting Node D...")
    res_d = post_json("/neuron", {
        "raw_payload": "Node D (Target)",
        "vector_data": [0.0] * 16,
        "model_creator_hash": "00" * 32,
        "payload_type": "text",
        "dna": None,
        "adjacency": []
    })
    node_d = res_d["neuron_id"]
    print(f"Node D Created: {node_d}")

    # 2. Insert Node B (Pruning path to D)
    print("\nInserting Node B (with Prune DNA)...")
    res_b = post_json("/neuron", {
        "raw_payload": "Node B (Pruning)",
        "vector_data": [0.0] * 16,
        "model_creator_hash": "00" * 32,
        "payload_type": "text",
        "dna": {
            "on_write": None,
            "on_read": None,
            "on_index": None,
            "on_traverse": None,
            "on_dream": None,
            "on_lifecycle": None,
            "on_path_step": "return false; // prune this path",
            "on_path_resolve": None,
            "parameters": {},
            "engine": "rhai"
        },
        "adjacency": [
            {
                "target_id": node_d,
                "weight": 0.9,
                "last_accessed_ns": 0
            }
        ]
    })
    node_b = res_b["neuron_id"]
    print(f"Node B Created: {node_b}")

    # 3. Insert Node C (Winning path to D)
    print("\nInserting Node C (with Reinforcement DNA)...")
    resolve_script = """
    let current_idx = -1;
    for i in 0..winning_path.len() {
        if winning_path[i] == neuron.id {
            current_idx = i;
        }
    }
    if current_idx >= 0 && current_idx < winning_path.len() - 1 {
        let next_node = winning_path[current_idx + 1];
        ctx.strengthen_edge(neuron.id, next_node, 0.25);
    }
    """
    res_c = post_json("/neuron", {
        "raw_payload": "Node C (Winning Route)",
        "vector_data": [0.0] * 16,
        "model_creator_hash": "00" * 32,
        "payload_type": "text",
        "dna": {
            "on_write": None,
            "on_read": None,
            "on_index": None,
            "on_traverse": None,
            "on_dream": None,
            "on_lifecycle": None,
            "on_path_step": "return true;",
            "on_path_resolve": resolve_script,
            "parameters": {},
            "engine": "rhai"
        },
        "adjacency": [
            {
                "target_id": node_d,
                "weight": 0.5, # Initial weight 0.5, will be reinforced to 0.75
                "last_accessed_ns": 0
            }
        ]
    })
    node_c = res_c["neuron_id"]
    print(f"Node C Created: {node_c}")

    # 4. Insert Node A (Start node branching to B and C)
    print("\nInserting Node A...")
    res_a = post_json("/neuron", {
        "raw_payload": "Node A (Start)",
        "vector_data": [0.0] * 16,
        "model_creator_hash": "00" * 32,
        "payload_type": "text",
        "dna": None,
        "adjacency": [
            {
                "target_id": node_b,
                "weight": 0.9, # Initially prefers B because of higher weight
                "last_accessed_ns": 0
            },
            {
                "target_id": node_c,
                "weight": 0.8,
                "last_accessed_ns": 0
            }
        ]
    })
    node_a = res_a["neuron_id"]
    print(f"Node A Created: {node_a}")

    # Wait for the flusher to drain Transit Lounge to LMDB
    print("\nWaiting 1 second for nodes to be flushed to LMDB...")
    time.sleep(1)

    # 5. Execute Speculative Search from A to D
    print("\nExecuting speculative search from A to D...")
    search_res = post_json("/graph/search/speculative", {
        "start_node": node_a,
        "target_node": node_d,
        "max_parallel_paths": 4,
        "max_depth": 5
    })
    
    print(f"Search result: {json.dumps(search_res, indent=2)}")
    
    assert search_res["success"] == True, "Search should succeed!"
    path = search_res["path"]
    
    # Path should be A -> C -> D, NOT A -> B -> D because B was pruned
    assert node_b not in path, "Path should NOT contain Node B because it was pruned!"
    assert node_c in path, "Path should contain Node C!"
    print("\nSUCCESS: Speculative routing bypassed the pruned path B and chose path C!")

    # 6. Verify reinforcement on Node C
    print("\nVerifying connection reinforcement on Node C...")
    time.sleep(0.5) # allow writing transaction to commit
    neuron_c = get_json(f"/neuron/{node_c}")
    
    edge_to_d = next((e for e in neuron_c["adjacency"] if e["target_id"] == node_d), None)
    assert edge_to_d is not None, "Edge from C to D must exist"
    print(f"Reinforced edge weight from C to D: {edge_to_d['weight']} (Expected: 0.75)")
    assert abs(edge_to_d["weight"] - 0.75) < 0.01, "Edge weight should have been reinforced to 0.75!"

    print("\n--- ALL SPECULATIVE ROUTING TESTS PASSED ---")

if __name__ == "__main__":
    main()
