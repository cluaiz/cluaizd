// Copyright (c) Cluaiz Technologies. Licensed under BSL 1.1.
//
//! CDQL Search Mechanics — End-to-End Integration Tests (Rust)
//!
//! Tests the complete pipeline:
//!   CDQL string → parse() → build_plan() → eval functions
//!
//! Covers all three newly implemented search index mechanics:
//!   1. Full-Text / Inverted Search  (`eval_full_text`)
//!   2. Range Scan                   (`eval_range`)
//!   3. Geo-Spatial Proximity        (`eval_geo_near`)
//!
//! Run with:
//!   cargo test --package genome cdql_search --nocapture

#[cfg(test)]
mod cdql_search_mechanics {
    use genome::cdql::{
        eval::{eval_full_text, eval_geo_near, eval_range},
        parser::CdqlValue,
        parse,
        planner::{build_plan, PlanStep},
    };

    // ─────────────────────────────────────────────────────────────────────────
    // HELPERS
    // ─────────────────────────────────────────────────────────────────────────

    /// Extract the first `FullTextSearch` step from a CDQL plan.
    fn extract_fulltext_step(cdql: &str) -> (String, bool) {
        let query = parse(cdql).expect("CDQL parse failed");
        let plan = build_plan(&query).expect("Plan build failed");
        for step in plan.steps {
            if let PlanStep::FullTextSearch { query, fuzzy } = step {
                return (query, fuzzy);
            }
        }
        panic!("No FullTextSearch step found in plan")
    }

    /// Extract the first `RangeScan` step from a CDQL plan.
    fn extract_range_step(cdql: &str) -> (String, CdqlValue, CdqlValue) {
        let query = parse(cdql).expect("CDQL parse failed");
        let plan = build_plan(&query).expect("Plan build failed");
        for step in plan.steps {
            if let PlanStep::RangeScan { field, start, end } = step {
                return (field, start, end);
            }
        }
        panic!("No RangeScan step found in plan")
    }

    /// Extract the first `GeoNear` step from a CDQL plan.
    fn extract_geo_step(cdql: &str) -> (f64, f64, f64) {
        let query = parse(cdql).expect("CDQL parse failed");
        let plan = build_plan(&query).expect("Plan build failed");
        for step in plan.steps {
            if let PlanStep::GeoNear { lat, lon, radius_km } = step {
                return (lat, lon, radius_km);
            }
        }
        panic!("No GeoNear step found in plan")
    }

    // ─────────────────────────────────────────────────────────────────────────
    // 1. FULL-TEXT SEARCH — PIPELINE TESTS
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn pipeline_fulltext_plan_parses_correctly() {
        let (query, fuzzy) = extract_fulltext_step(
            r#"find * -> search(query: "database engine", fuzzy: false)"#
        );
        assert_eq!(query, "database engine");
        assert!(!fuzzy);
    }

    #[test]
    fn pipeline_fulltext_fuzzy_plan_parses_correctly() {
        let (query, fuzzy) = extract_fulltext_step(
            r#"find * -> search(query: "kernel", fuzzy: true)"#
        );
        assert_eq!(query, "kernel");
        assert!(fuzzy);
    }

    #[test]
    fn pipeline_fulltext_exact_matches_relevant_payload() {
        let (query, fuzzy) = extract_fulltext_step(
            r#"find * -> search(query: "database engine", fuzzy: false)"#
        );
        let rust_payload = r#"{"description": "A high-performance database engine written in Rust"}"#;
        let python_payload = r#"{"description": "Python tutorial for beginners covering loops"}"#;

        assert!(
            eval_full_text(rust_payload, &query, fuzzy) > 0.0,
            "rust payload must match 'database engine'"
        );
        assert_eq!(
            eval_full_text(python_payload, &query, fuzzy),
            0.0,
            "python tutorial must NOT match 'database engine'"
        );
    }

    #[test]
    fn pipeline_fulltext_fuzzy_matches_substring() {
        let (query, fuzzy) = extract_fulltext_step(
            r#"find * -> search(query: "data", fuzzy: true)"#
        );
        let payload = r#"{"name": "database", "category": "storage engine"}"#;
        assert!(
            eval_full_text(payload, &query, fuzzy) > 0.0,
            "fuzzy 'data' should match payload containing 'database'"
        );
    }

    #[test]
    fn pipeline_fulltext_score_orders_by_relevance() {
        // "high relevance" payload has both tokens matched; "low relevance" only one.
        let query = "database engine";
        let high = r#"{"text": "distributed database engine kernel"}"#;
        let low  = r#"{"text": "just a database"}"#;
        let none = r#"{"text": "robotics lidar stream"}"#;

        let score_high = eval_full_text(high, query, false);
        let score_low  = eval_full_text(low,  query, false);
        let score_none = eval_full_text(none, query, false);

        assert!(score_high > score_low,  "2-token match must score higher than 1-token");
        assert!(score_low  > score_none, "1-token match must score higher than 0-token");
        assert_eq!(score_none, 0.0);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // 2. RANGE SCAN — PIPELINE TESTS
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn pipeline_range_plan_parses_numeric_correctly() {
        let (field, start, end) = extract_range_step(
            r#"find * -> range(field: "age", start: 20, end: 30)"#
        );
        assert_eq!(field, "age");
        assert!(matches!(start, CdqlValue::Number(n) if (n - 20.0).abs() < 1e-9));
        assert!(matches!(end,   CdqlValue::Number(n) if (n - 30.0).abs() < 1e-9));
    }

    #[test]
    fn pipeline_range_numeric_alice_in_bounds() {
        let (field, start, end) = extract_range_step(
            r#"find * -> range(field: "age", start: 20, end: 30)"#
        );
        let alice = r#"{"name": "alice", "age": 25}"#;
        assert!(eval_range(alice, &field, &start, &end), "alice (age 25) must be in [20, 30]");
    }

    #[test]
    fn pipeline_range_numeric_bob_out_of_bounds() {
        let (field, start, end) = extract_range_step(
            r#"find * -> range(field: "age", start: 20, end: 30)"#
        );
        let bob = r#"{"name": "bob", "age": 35}"#;
        assert!(!eval_range(bob, &field, &start, &end), "bob (age 35) must NOT be in [20, 30]");
    }

    #[test]
    fn pipeline_range_boundary_values_inclusive() {
        let (field, start, end) = extract_range_step(
            r#"find * -> range(field: "age", start: 20, end: 30)"#
        );
        let at_lower = r#"{"age": 20}"#;
        let at_upper = r#"{"age": 30}"#;
        assert!(eval_range(at_lower, &field, &start, &end), "lower boundary 20 must be inclusive");
        assert!(eval_range(at_upper, &field, &start, &end), "upper boundary 30 must be inclusive");
    }

    #[test]
    fn pipeline_range_missing_field_returns_false() {
        let (field, start, end) = extract_range_step(
            r#"find * -> range(field: "salary", start: 50000, end: 90000)"#
        );
        let no_salary = r#"{"name": "carol"}"#;
        assert!(!eval_range(no_salary, &field, &start, &end), "missing field must return false");
    }

    #[test]
    fn pipeline_range_salary_filters_correctly() {
        let (field, start, end) = extract_range_step(
            r#"find * -> range(field: "salary", start: 60000, end: 90000)"#
        );
        let alice = r#"{"name": "alice", "salary": 70000}"#;
        let bob   = r#"{"name": "bob",   "salary": 120000}"#;
        let carol = r#"{"name": "carol", "salary": 85000}"#;

        assert!( eval_range(alice, &field, &start, &end), "alice 70k in [60k-90k]");
        assert!(!eval_range(bob,   &field, &start, &end), "bob 120k NOT in [60k-90k]");
        assert!( eval_range(carol, &field, &start, &end), "carol 85k in [60k-90k]");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // 3. GEO-SPATIAL SEARCH — PIPELINE TESTS
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn pipeline_geo_plan_parses_correctly() {
        let (lat, lon, radius_km) = extract_geo_step(
            r#"find * -> geo_near(lat: 28.6139, lon: 77.2090, radius_km: 50.0)"#
        );
        assert!((lat - 28.6139).abs() < 1e-4, "lat must parse to ~28.6139");
        assert!((lon - 77.2090).abs() < 1e-4, "lon must parse to ~77.2090");
        assert!((radius_km - 50.0).abs() < 1e-4, "radius_km must parse to 50.0");
    }

    #[test]
    fn pipeline_geo_gurgaon_within_50km_of_delhi() {
        // Gurgaon is ~35 km from New Delhi.
        let (lat, lon, radius_km) = extract_geo_step(
            r#"find * -> geo_near(lat: 28.6139, lon: 77.2090, radius_km: 50.0)"#
        );
        let gurgaon = r#"{"city": "Gurgaon", "lat": 28.4595, "lon": 77.0266}"#;
        assert!(
            eval_geo_near(gurgaon, lat, lon, radius_km).is_some(),
            "Gurgaon (~35 km) must be within 50 km radius of Delhi"
        );
    }

    #[test]
    fn pipeline_geo_mumbai_outside_50km_of_delhi() {
        // Mumbai is ~1,150 km from New Delhi.
        let (lat, lon, radius_km) = extract_geo_step(
            r#"find * -> geo_near(lat: 28.6139, lon: 77.2090, radius_km: 50.0)"#
        );
        let mumbai = r#"{"city": "Mumbai", "lat": 19.0760, "lon": 72.8777}"#;
        assert!(
            eval_geo_near(mumbai, lat, lon, radius_km).is_none(),
            "Mumbai (~1150 km) must NOT be within 50 km radius of Delhi"
        );
    }

    #[test]
    fn pipeline_geo_alternate_key_variants_latitude_longitude() {
        let (lat, lon, radius_km) = extract_geo_step(
            r#"find * -> geo_near(lat: 28.6139, lon: 77.2090, radius_km: 50.0)"#
        );
        // Same position as Gurgaon but using full key names.
        let gurgaon_alt = r#"{"city": "Gurgaon", "latitude": 28.4595, "longitude": 77.0266}"#;
        assert!(
            eval_geo_near(gurgaon_alt, lat, lon, radius_km).is_some(),
            "Must resolve 'latitude'/'longitude' key names as well"
        );
    }

    #[test]
    fn pipeline_geo_missing_coords_returns_none() {
        let (lat, lon, radius_km) = extract_geo_step(
            r#"find * -> geo_near(lat: 28.6139, lon: 77.2090, radius_km: 50.0)"#
        );
        let no_coords = r#"{"city": "Unknown"}"#;
        assert!(
            eval_geo_near(no_coords, lat, lon, radius_km).is_none(),
            "Payload with no coordinate fields must return None"
        );
    }

    #[test]
    fn pipeline_geo_closer_city_scores_higher() {
        // Gurgaon (~35 km) should score higher than Faridabad (~25 km from Delhi).
        // We compare inverse-distance scores.
        let delhi_lat = 28.6139_f64;
        let delhi_lon = 77.2090_f64;

        let gurgaon  = r#"{"lat": 28.4595, "lon": 77.0266}"#; // ~35 km
        let noida    = r#"{"lat": 28.5355, "lon": 77.3910}"#; // ~13 km

        let score_gurgaon = eval_geo_near(gurgaon, delhi_lat, delhi_lon, 100.0).unwrap();
        let score_noida   = eval_geo_near(noida,   delhi_lat, delhi_lon, 100.0).unwrap();

        assert!(
            score_noida > score_gurgaon,
            "Noida (~13 km) must score higher than Gurgaon (~35 km)"
        );
    }
}
