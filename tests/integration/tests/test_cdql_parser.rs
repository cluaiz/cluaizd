mod common;
use serde_json::json;

#[tokio::test]
async fn test_cdql_complex_parser_stability() {
    common::wait_for_server().await;
    let client = common::get_client();

    // Policy Rule 2: Dynamic Custom Modalities
    // We send a complex CDQL string to the /api/v1/cdql endpoint.
    // Even if the payload format or table doesn't exist, the PARSER must not panic.
    // It should translate the string into an AST or return a Graceful Error.
    
    let cdql_payload = json!({
        "query": "GROUP_BY time_bucket(1 HOURS) -> AGGREGATE avg(payload.temperature) as avg_temp WHERE timestamp < SYSTEM.NOW - 7 DAYS"
    });

    let res = client
        .post(&format!("{}/query", common::BASE_URL))
        .json(&cdql_payload)
        .send()
        .await
        .expect("Failed to send CDQL request");

    // The parser shouldn't panic (which would drop the connection or return 500).
    // It might return 200 (Parsed and executed) or 400 (Bad Syntax/Schema), 
    // but 500 means the parser crashed, which is a violation of engine stability.
    assert_ne!(res.status(), 500, "CDQL Parser panicked and returned 500 Internal Server Error");
}
