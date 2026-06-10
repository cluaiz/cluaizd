use serde_json::json;
use reqwest::StatusCode;

mod common;

#[tokio::test]
async fn test_malformed_payload_error_handling() {
    common::wait_for_server().await;
    let client = common::get_client();

    // Send malformed JSON payload (missing closing brace)
    let bad_json = r#"{"raw_payload": "broken json", "vector_data": [0.0; 16]"#;
    
    let res = client.post(&format!("{}/neuron", common::BASE_URL))
        .header("Content-Type", "application/json")
        .body(bad_json)
        .send()
        .await
        .expect("Failed to send POST request");

    // Server should reject it gracefully with a 400 Bad Request, not crash.
    assert_eq!(res.status(), StatusCode::BAD_REQUEST, "Server did not reject malformed JSON with 400");
}

#[tokio::test]
async fn test_missing_required_fields() {
    common::wait_for_server().await;
    let client = common::get_client();

    // Send valid JSON but missing required fields (e.g., model_creator_hash)
    let missing_fields = json!({
        "raw_payload": "Valid JSON, but missing required structure",
    });
    
    let res = client.post(&format!("{}/neuron", common::BASE_URL))
        .json(&missing_fields)
        .send()
        .await
        .expect("Failed to send POST request");

    // Server should reject it gracefully with a 422 Unprocessable Entity or 400 Bad Request
    assert!(res.status().is_client_error(), "Server did not reject missing required fields");
}

#[tokio::test]
async fn test_oom_crash_prevention_heavy_ingestion() {
    common::wait_for_server().await;
    let client = common::get_client();

    // Create a 50MB payload
    let huge_string = "A".repeat(50 * 1024 * 1024);
    let vector_data = vec![0.0_f32; 16];

    let huge_payload = json!({
        "raw_payload": huge_string,
        "vector_data": vector_data,
        "model_creator_hash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "payload_type": "text",
    });
    
    let res = client.post(&format!("{}/neuron", common::BASE_URL))
        .json(&huge_payload)
        .send()
        .await
        .expect("Failed to send POST request");

    // Server might accept it or reject with Payload Too Large (413).
    // The main test is that the request completes and the server didn't crash (OOM).
    // If it succeeds, we're testing the system can handle a 50MB block cleanly.
    // We just verify it returns a valid HTTP response (not a connection drop).
    assert!(res.status().is_success() || res.status() == StatusCode::PAYLOAD_TOO_LARGE, "Unexpected status code for huge payload: {}", res.status());
}
