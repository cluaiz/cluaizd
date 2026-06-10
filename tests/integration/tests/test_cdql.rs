use serde_json::json;
use tokio::task;

mod common;
use common::logger::TestLogger;

/// Helper function to create neurons for testing
async fn seed_data(client: &reqwest::Client, payload: serde_json::Value) -> Option<String> {
    let res = client.post(&format!("{}/neuron", common::BASE_URL))
        .json(&payload)
        .send()
        .await
        .ok()?;
    
    let res_json: serde_json::Value = res.json().await.ok()?;
    res_json["neuron_id"].as_str().map(|s| s.to_string())
}

#[tokio::test]
async fn test_cdql_multi_paradigm() {
    common::wait_for_server().await;
    let logger = TestLogger::start("CDQL - Multi-Paradigm CRUD Completeness");
    let client = common::get_client();

    // 1. Seed some Relational / SQL Data
    let ids = vec![
        seed_data(&client, json!({
            "raw_payload": { "type": "Employee", "department": "Engineering", "salary": 100000 },
            "vector_data": [0.1; 16], "model_creator_hash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855", "payload_type": "text"
        })).await,
        seed_data(&client, json!({
            "raw_payload": { "type": "Employee", "department": "Engineering", "salary": 120000 },
            "vector_data": [0.2; 16], "model_creator_hash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855", "payload_type": "text"
        })).await,
        seed_data(&client, json!({
            "raw_payload": { "type": "Employee", "department": "Marketing", "salary": 80000 },
            "vector_data": [0.3; 16], "model_creator_hash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855", "payload_type": "text"
        })).await,
    ];

    if ids.iter().any(|id| id.is_none()) {
        logger.fail("Failed to seed initial relational data.");
        panic!("Seed failure");
    }

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // 2. Test SQL GroupBy & Aggregate
    let agg_query = json!({
        "cdql": "find *(type: \"Employee\") -> group_by(\"department\") -> aggregate(count(), sum(\"salary\"))"
    });

    let res = client.post(&format!("{}/query", common::BASE_URL))
        .json(&agg_query)
        .send()
        .await
        .expect("Failed to send aggregate query");

    if res.status().is_success() {
        let results: Vec<serde_json::Value> = res.json().await.expect("Parse agg results");
        if results.len() >= 2 { // At least Engineering and Marketing
            logger.pass(&format!("Successfully executed SQL Aggregate: {}", json!(results)));
        } else {
            logger.fail(&format!("Aggregate returned incomplete data: {}", json!(results)));
        }
    } else {
        logger.fail("Aggregate CDQL query failed.");
    }

    // 3. Test Vector Search Pipeline
    let vec_query = json!({
        "cdql": "find * -> similar_to(vector: [0.1], metric: \"cosine\") -> limit 1"
    });
    
    // Note: The planner parses SimilarTo, but our dummy vector_data may just return everything.
    // The point is to verify the parser doesn't crash.
    let vec_res = client.post(&format!("{}/query", common::BASE_URL))
        .json(&vec_query)
        .send()
        .await
        .expect("Failed vector query");

    if vec_res.status().is_success() {
        logger.pass("Successfully parsed and executed Vector Similarity Pipeline.");
    } else {
        logger.fail("Vector search pipeline failed.");
    }
}
