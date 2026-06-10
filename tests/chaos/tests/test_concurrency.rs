use serde_json::json;
use tokio::task;

mod common;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_100_concurrent_writers_deadlock_check() {
    common::wait_for_server().await;
    let client = common::get_client();
    
    let mut handles = vec![];
    for i in 0..100 {
        let client_clone = client.clone();
        handles.push(tokio::spawn(async move {
            let raw_payload = format!("Concurrent Payload {}", i);
            let payload = json!({
                "raw_payload": raw_payload,
                "vector_data": vec![0.0_f32; 16],
                "model_creator_hash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                "payload_type": "text",
            });
            
            let res = client_clone.post(&format!("{}/neuron", common::BASE_URL))
                .json(&payload)
                .send()
                .await
                .expect("Failed POST request");
                
            assert_eq!(res.status(), 201, "Failed to insert in concurrent loop");
        }));
    }
    
    // Await all tasks
    for handle in handles {
        handle.await.expect("Task panicked");
    }
}


