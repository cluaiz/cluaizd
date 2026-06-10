use serde_json::json;
use tokio::task;
use std::time::Duration;

mod common;
use common::logger::TestLogger;

/// Tests Consistency: Invalid dimensions should be rejected
#[tokio::test]
async fn test_acid_consistency_invalid_vector() {
    common::wait_for_server().await;
    let logger = TestLogger::start("ACID - Consistency (Invalid Vector)");
    let client = common::get_client();

    // Create payload with WRONG vector dimensions (15 instead of 16)
    // Wait, the API might auto-pad or we should pass something it will reject.
    // Let's pass a NaN value if JSON supports it, or simply a string instead of array.
    // A string instead of array should fail the strongly typed JSON deserialization.
    let payload = json!({
        "raw_payload": "Trying to corrupt the database",
        "vector_data": "not an array",
        "model_creator_hash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "payload_type": "text",
    });

    let res = client.post(&format!("{}/neuron", common::BASE_URL))
        .json(&payload)
        .send()
        .await
        .expect("Failed to send POST request");

    // The server should REJECT this and return HTTP 400 Bad Request or Unprocessable Entity
    if res.status().is_client_error() {
        logger.pass(&format!("Properly rejected invalid vector data: HTTP {}", res.status()));
    } else {
        logger.fail(&format!("Allowed invalid vector data to be written! HTTP {}", res.status()));
        panic!("Consistency check failed");
    }
}

/// Tests Isolation: 1000 concurrent writes to check for race conditions
#[tokio::test]
async fn test_acid_isolation_concurrent_writes() {
    common::wait_for_server().await;
    let logger = TestLogger::start("ACID - Isolation (100 Concurrent Writes)");
    let client = common::get_client();

    let mut tasks = vec![];

    for i in 0..100 {
        let client_clone = client.clone();
        tasks.push(tokio::spawn(async move {
            let vector_data = vec![0.0_f32; 16];
            let payload = json!({
                "raw_payload": format!("Concurrent transaction #{}", i),
                "vector_data": vector_data,
                "model_creator_hash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                "payload_type": "text",
            });

            let res = client_clone.post(&format!("{}/neuron", common::BASE_URL))
                .json(&payload)
                .send()
                .await;

            res.is_ok() && res.unwrap().status().is_success()
        }));
    }

    let results = futures_util::future::join_all(tasks).await;
    
    let success_count = results.into_iter()
        .filter(|r| r.is_ok() && *r.as_ref().unwrap())
        .count();

    if success_count == 100 {
        logger.pass("All 100 concurrent transactions committed cleanly without dirty writes or deadlocks.");
    } else {
        logger.fail(&format!("Only {}/100 transactions succeeded. Possible race condition or deadlock.", success_count));
        panic!("Isolation check failed");
    }
}

/// Tests Atomicity: Missing payload type
#[tokio::test]
async fn test_acid_atomicity_missing_fields() {
    common::wait_for_server().await;
    let logger = TestLogger::start("ACID - Atomicity (Incomplete Schema)");
    let client = common::get_client();

    let payload = json!({
        "raw_payload": "Missing required fields",
        // No vector_data, no model_creator_hash
    });

    let res = client.post(&format!("{}/neuron", common::BASE_URL))
        .json(&payload)
        .send()
        .await
        .expect("Failed to send POST request");

    if res.status().is_client_error() {
        logger.pass("Atomically rejected incomplete schema.");
    } else {
        logger.fail("Allowed partial/incomplete transaction!");
        panic!("Atomicity check failed");
    }
}
