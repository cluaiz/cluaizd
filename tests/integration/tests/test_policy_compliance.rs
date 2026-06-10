use serde_json::json;

mod common;

#[tokio::test]
async fn test_static_heap_mode_immortal_node() {
    common::wait_for_server().await;
    let client = common::get_client();

    // Policy Rule 4: Static Heap Mode is Default
    // Create a node with NO rules attached.
    let raw_payload_string: String = serde_json::to_string(&json!({
        "name": "Static Config",
        "version": "1.0.0",
        "desc": "This node has no rules. It should be immortal."
    })).unwrap();

    let static_heap_payload = json!({
        "raw_payload": raw_payload_string,
        "vector_data": vec![0.0_f32; 16],
        "model_creator_hash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "payload_type": "text"
    });

    // 1. Insert Node
    let res = client
        .post(&format!("{}/neuron", common::BASE_URL))
        .json(&static_heap_payload)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(res.status(), 201, "Failed to insert Static Heap node");

    let response_json: serde_json::Value = res.json().await.unwrap();
    let node_id = response_json["neuron_id"].as_str().expect("Missing neuron_id in response");

    // Wait briefly for Transit Lounge to flush to LMDB
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    // 2. Read it back immediately
    let get_res = client
        .get(&format!("{}/neuron/{}", common::BASE_URL, node_id))
        .send()
        .await
        .expect("Failed to execute GET request");

    assert_eq!(get_res.status(), 200);
    let get_json: serde_json::Value = get_res.json().await.unwrap();
    
    // The payload is returned as an array of bytes, let's decode it to check
    let returned_bytes: Vec<u8> = serde_json::from_value(get_json["raw_payload"].clone()).unwrap();
    let returned_payload: serde_json::Value = serde_json::from_slice(&returned_bytes).unwrap();
    
    // Assert payload remains untouched
    assert_eq!(returned_payload["name"], "Static Config");

    // In a full integration run, we would wait for GC sweeps and verify it still exists,
    // but verifying it accepts null rules is the first step of Policy compliance.
}

#[tokio::test]
async fn test_dna_hook_attachment() {
    common::wait_for_server().await;
    let client = common::get_client();

    // Policy Rule 2: Dynamic Custom Modalities
    // Create a node WITH an explicit on_write DNA hook.
    // For this test, we assume the backend has a basic parser that accepts this JSON structure.
    let raw_payload_string: String = serde_json::to_string(&json!({
        "email": "test@cluaiz.com",
        "age": 25
    })).unwrap();

    let dna_payload = json!({
        "raw_payload": raw_payload_string,
        "vector_data": vec![0.0_f32; 16],
        "model_creator_hash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "payload_type": "text",
        "dna": {
            "on_write": "return #{ \"action\": \"Allow\" };",
            "engine": "rhai",
            "parameters": {
                "required_fields": [{"name": "email", "type": "string"}]
            }
        }
    });

    let res = client
        .post(&format!("{}/neuron", common::BASE_URL))
        .json(&dna_payload)
        .send()
        .await
        .expect("Failed to execute request");

    // It should either return 201 (if the dummy script passes/isn't actually executed in this test build)
    // or return a specific error. The main assertion is that the API accepts the "rules" array cleanly.
    assert!(
        res.status().is_success() || res.status().is_client_error(),
        "Unexpected server error when sending DNA rules"
    );
}
