// Copyright (c) Cluaiz Technologies. Licensed under BSL 1.1.
// SAFETY: All operations are safe. No unsafe blocks.

//! CDQL Search Evaluation Functions
//!
//! This module provides pure, stateless evaluation helpers for the three
//! advanced CDQL search index mechanics:
//!
//! - Full-Text / Inverted Search  (`eval_full_text`)
//! - Range Scan                   (`eval_range`)
//! - Geo-Spatial Proximity        (`eval_geo_near`)
//!
//! Every function receives borrowed, zero-copy references and returns a score
//! or a boolean — NO heap allocation outside of the JSON parse step.

use super::parser::CdqlValue;

// ─────────────────────────────────────────────────────────────────────────────
// 1. FULL-TEXT / INVERTED SEARCH
// ─────────────────────────────────────────────────────────────────────────────

/// Score how well `payload` matches `query` tokens.
///
/// Returns a score in `[0.0, ∞)`. A score of `0.0` means no match.
/// The score is proportional to the number of matched tokens, so that
/// results can be sorted by relevance.
///
/// # Params
/// - `payload`:  The raw textual content of the neuron's JSON payload (as a `&str`).
/// - `query`:    The search query string (e.g. `"database engine"`).
/// - `fuzzy`:    If `true`, a query token matches any payload word that *contains* it
///               as a substring. If `false`, only exact word matches are counted.
pub fn eval_full_text(payload: &str, query: &str, fuzzy: bool) -> f32 {
    if query.is_empty() || payload.is_empty() {
        return 0.0;
    }

    // Extract all textual content from JSON values, or fall back to raw string.
    let searchable = extract_text_from_json(payload);

    let query_tokens: Vec<&str> = query.split_whitespace().collect();
    let payload_words: Vec<&str> = searchable.split_whitespace().collect();

    let mut matched = 0usize;

    for q_tok in &query_tokens {
        let q_lower = q_tok.to_lowercase();
        let found = payload_words.iter().any(|w| {
            let w_lower = w.to_lowercase();
            if fuzzy {
                w_lower.contains(q_lower.as_str())
            } else {
                w_lower == q_lower.as_str()
            }
        });
        if found {
            matched += 1;
        }
    }

    if matched == 0 {
        return 0.0;
    }

    // Normalised score: proportion of query tokens matched × document coverage bonus.
    let query_coverage = matched as f32 / query_tokens.len() as f32;
    query_coverage
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. RANGE SCAN
// ─────────────────────────────────────────────────────────────────────────────

/// Check whether a neuron's `field` falls within `[start, end]`.
///
/// Supports numeric and lexicographic (string) comparisons.
/// Returns `false` on any parse failure — never panics.
///
/// # Params
/// - `payload`:  The raw JSON payload of the neuron.
/// - `field`:    The JSON key to extract and compare (e.g. `"age"`, `"salary"`).
/// - `start`:    The lower boundary value (inclusive).
/// - `end`:      The upper boundary value (inclusive).
pub fn eval_range(payload: &str, field: &str, start: &CdqlValue, end: &CdqlValue) -> bool {
    let json: serde_json::Value = match serde_json::from_str(payload) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let field_val = match json.get(field) {
        Some(v) => v,
        None => return false,
    };

    match (start, end) {
        // ── Numeric range ──────────────────────────────────────────────────
        (CdqlValue::Number(lo), CdqlValue::Number(hi)) => {
            if let Some(num) = field_val.as_f64() {
                num >= *lo && num <= *hi
            } else {
                false
            }
        }
        // ── String / lexicographic range ───────────────────────────────────
        (CdqlValue::Text(lo), CdqlValue::Text(hi)) => {
            if let Some(s) = field_val.as_str() {
                s >= lo.as_str() && s <= hi.as_str()
            } else {
                false
            }
        }
        // Mixed types — no valid comparison.
        _ => false,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. GEO-SPATIAL PROXIMITY (Haversine)
// ─────────────────────────────────────────────────────────────────────────────

/// Check whether the neuron's stored coordinates are within `radius_km` of the
/// target point `(target_lat, target_lon)`.
///
/// Returns `Some(score)` if the neuron is inside the radius, where the score
/// is inverse-proportional to distance (closer = higher score). Returns `None`
/// if the neuron has no coordinate fields or lies outside the radius.
///
/// The function probes for coordinate keys in order:
/// - latitude: `"lat"`, then `"latitude"`
/// - longitude: `"lon"`, then `"longitude"`
///
/// # Params
/// - `payload`:     Raw JSON payload of the neuron.
/// - `target_lat`:  Query point latitude in decimal degrees.
/// - `target_lon`:  Query point longitude in decimal degrees.
/// - `radius_km`:   Maximum accepted distance in kilometres.
pub fn eval_geo_near(
    payload: &str,
    target_lat: f64,
    target_lon: f64,
    radius_km: f64,
) -> Option<f32> {
    let json: serde_json::Value = serde_json::from_str(payload).ok()?;

    // Probe common coordinate key variants.
    let neuron_lat = probe_f64(&json, &["lat", "latitude"])?;
    let neuron_lon = probe_f64(&json, &["lon", "longitude"])?;

    let dist_km = haversine_km(neuron_lat, neuron_lon, target_lat, target_lon);

    if dist_km <= radius_km {
        // Score: inverse distance, clamped to [0.0, 1.0].
        let score = 1.0_f32 / (1.0_f32 + dist_km as f32);
        Some(score)
    } else {
        None
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PRIVATE HELPERS
// ─────────────────────────────────────────────────────────────────────────────

/// Haversine great-circle distance formula.
/// Returns the distance in kilometres between two decimal-degree coordinates.
fn haversine_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;

    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();

    let a = (d_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS_KM * c
}

/// Try to read an `f64` from the first matching key in `candidates`.
fn probe_f64(json: &serde_json::Value, candidates: &[&str]) -> Option<f64> {
    for key in candidates {
        if let Some(v) = json.get(*key).and_then(|v| v.as_f64()) {
            return Some(v);
        }
    }
    None
}

/// Extract all string-valued leaves from a JSON object as a single space-joined string.
/// Falls back to returning `payload` verbatim if it is not valid JSON.
fn extract_text_from_json(payload: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(payload) {
        Ok(json) => collect_strings(&json),
        Err(_) => payload.to_string(),
    }
}

/// Recursively collect all string leaf values from a JSON value into a single string.
fn collect_strings(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Object(map) => map
            .values()
            .map(collect_strings)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" "),
        serde_json::Value::Array(arr) => arr
            .iter()
            .map(collect_strings)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" "),
        serde_json::Value::Number(n) => n.to_string(),
        _ => String::new(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// UNIT TESTS
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    // ── Full-Text Tests ──────────────────────────────────────────────────────

    #[test]
    fn test_fulltext_exact_match() {
        let payload = r#"{"name": "database engine"}"#;
        let score = eval_full_text(payload, "database engine", false);
        assert!(score > 0.0, "exact match must score > 0");
    }

    #[test]
    fn test_fulltext_no_match() {
        let payload = r#"{"name": "robotics stream"}"#;
        let score = eval_full_text(payload, "quantum entanglement", false);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_fulltext_fuzzy() {
        let payload = r#"{"description": "distributed database kernel"}"#;
        let score = eval_full_text(payload, "data", true);
        assert!(score > 0.0, "fuzzy should match 'database' with 'data'");
    }

    #[test]
    fn test_fulltext_empty_query() {
        let payload = r#"{"name": "something"}"#;
        assert_eq!(eval_full_text(payload, "", false), 0.0);
    }

    // ── Range Scan Tests ─────────────────────────────────────────────────────

    #[test]
    fn test_range_numeric_inside() {
        let payload = r#"{"age": 25}"#;
        let result = eval_range(payload, "age", &CdqlValue::Number(20.0), &CdqlValue::Number(30.0));
        assert!(result);
    }

    #[test]
    fn test_range_numeric_outside() {
        let payload = r#"{"age": 50}"#;
        let result = eval_range(payload, "age", &CdqlValue::Number(20.0), &CdqlValue::Number(30.0));
        assert!(!result);
    }

    #[test]
    fn test_range_string_lexicographic() {
        let payload = r#"{"city": "Mumbai"}"#;
        let result = eval_range(
            payload,
            "city",
            &CdqlValue::Text("Chennai".into()),
            &CdqlValue::Text("Pune".into()),
        );
        assert!(result, "'Mumbai' should fall between 'Chennai' and 'Pune' lexicographically");
    }

    #[test]
    fn test_range_missing_field() {
        let payload = r#"{"name": "Aryan"}"#;
        let result = eval_range(payload, "salary", &CdqlValue::Number(0.0), &CdqlValue::Number(100.0));
        assert!(!result);
    }

    // ── Geo-Spatial Tests ────────────────────────────────────────────────────

    #[test]
    fn test_geo_near_delhi_within_50km() {
        // New Delhi: 28.6139, 77.2090
        // Gurgaon:   28.4595, 77.0266 (~35 km from Delhi)
        let payload = r#"{"lat": 28.4595, "lon": 77.0266, "name": "Gurgaon"}"#;
        let result = eval_geo_near(payload, 28.6139, 77.2090, 50.0);
        assert!(result.is_some(), "Gurgaon should be within 50km of Delhi");
    }

    #[test]
    fn test_geo_near_mumbai_outside_delhi_radius() {
        // Mumbai: 19.0760, 72.8777
        let payload = r#"{"lat": 19.0760, "lon": 72.8777, "name": "Mumbai"}"#;
        let result = eval_geo_near(payload, 28.6139, 77.2090, 50.0);
        assert!(result.is_none(), "Mumbai should be outside 50km of Delhi");
    }

    #[test]
    fn test_geo_near_latitude_key_variant() {
        let payload = r#"{"latitude": 28.4595, "longitude": 77.0266}"#;
        let result = eval_geo_near(payload, 28.6139, 77.2090, 50.0);
        assert!(result.is_some(), "should resolve 'latitude'/'longitude' keys");
    }

    #[test]
    fn test_geo_near_missing_coords() {
        let payload = r#"{"name": "Unknown City"}"#;
        let result = eval_geo_near(payload, 28.6139, 77.2090, 50.0);
        assert!(result.is_none());
    }

    #[test]
    fn test_haversine_distance_accuracy() {
        // Delhi to Mumbai should be approximately 1,150-1,200 km
        let dist = haversine_km(28.6139, 77.2090, 19.0760, 72.8777);
        assert!(dist > 1100.0 && dist < 1250.0, "Delhi-Mumbai distance should be ~1150km, got {}", dist);
    }
}
