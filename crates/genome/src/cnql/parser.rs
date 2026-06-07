use serde::{Deserialize, Serialize};

/// A single value in a CNQL expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CnqlValue {
    Text(String),
    Number(f64),
    Bool(bool),
    Vector(Vec<f32>),
    Null,
}

/// Comparison operators for filters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CompareOp {
    Eq,        // =
    NotEq,     // !=
    Gt,        // >
    Lt,        // <
    Gte,       // >=
    Lte,       // <=
    Contains,  // contains
}

/// A single key-value filter condition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Filter {
    pub field: String,
    pub op: CompareOp,
    pub value: CnqlValue,
}

/// Join types for Relational queries
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

/// Aggregation functions for Time-Series and SQL
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AggFunc {
    Count,
    Sum(String),
    Avg(String),
    Min(String),
    Max(String),
}

/// Each "step" in a CNQL pipeline (Supporting 10 Database Paradigms)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CnqlOp {
    // ---------------------------------------------------------
    // 1. CORE / DOCUMENT (MongoDB style)
    // ---------------------------------------------------------
    /// `find *(filters...)` — Scan all neurons matching the label/filters
    Find {
        label: String,
        filters: Vec<Filter>,
    },
    /// Fast Path: `find id("123")` for O(1) Key-Value lookup
    FindById {
        id: String,
    },
    /// `-> filter <field> <op> <value>` — Narrow down result set
    Filter {
        field: String,
        op: CompareOp,
        value: CnqlValue,
    },
    /// `-> project(keep: ["name"], compute: {...})` — Shape document
    Project {
        keep: Vec<String>,
        // compute ignored for now in MVP
    },
    /// `-> unwind("tags")` — Flatten arrays
    Unwind {
        field: String,
    },

    // ---------------------------------------------------------
    // 2. GRAPH (Neo4j style)
    // ---------------------------------------------------------
    /// `-> traverse(edge: "friends", hops: 1..3)`
    Traverse {
        edge: String,
        min_hops: usize,
        max_hops: usize,
        min_weight: f64,
    },
    /// `-> shortest_path(to: "NodeB")`
    ShortestPath {
        to_node: String,
    },

    // ---------------------------------------------------------
    // 3. RELATIONAL / SQL (PostgreSQL style)
    // ---------------------------------------------------------
    /// `-> join(target: "orders", on: "id == target.user_id", type: "inner")`
    Join {
        target: String,
        on_left: String,
        on_right: String,
        join_type: JoinType,
    },
    /// `-> group_by("department")`
    GroupBy {
        fields: Vec<String>,
    },
    /// `-> aggregate(count(), sum(salary))`
    Aggregate {
        functions: Vec<AggFunc>,
    },

    // ---------------------------------------------------------
    // 4. TIME-SERIES (InfluxDB style)
    // ---------------------------------------------------------
    /// `-> time_window(size: "5m")`
    TimeWindow {
        size: String,
    },

    // ---------------------------------------------------------
    // 5. VECTOR / AI (Pinecone style)
    // ---------------------------------------------------------
    /// `-> similar_to(vector: [...], metric: "cosine")`
    SimilarTo {
        vector: Vec<f32>,
        metric: String,
    },

    // ---------------------------------------------------------
    // 6. FULL-TEXT SEARCH (Elasticsearch style)
    // ---------------------------------------------------------
    /// `-> search(query: "hello", fuzzy: true)`
    Search {
        query: String,
        fuzzy: bool,
    },

    // ---------------------------------------------------------
    // 7. GEO-SPATIAL (PostGIS style)
    // ---------------------------------------------------------
    /// `-> geo_near(lat: 28.6, lon: 77.2, radius: "5km")`
    GeoNear {
        lat: f64,
        lon: f64,
        radius_km: f64,
    },

    // ---------------------------------------------------------
    // 8. WIDE-COLUMN (Cassandra style)
    // ---------------------------------------------------------
    /// `-> range_scan(start: X, end: Y)`
    RangeScan {
        field: String,
        start: CnqlValue,
        end: CnqlValue,
    },

    // ---------------------------------------------------------
    // 9. BLOB / OBJECT STORAGE (S3 style)
    // ---------------------------------------------------------
    /// `-> stream(bytes: 0..1024)`
    Stream {
        start_byte: usize,
        end_byte: usize,
    },

    // ---------------------------------------------------------
    // 10. UTILITY
    // ---------------------------------------------------------
    Limit(usize),
    SortBy {
        field: String,
        ascending: bool,
    },
}

/// The full CNQL query — an ordered pipeline of operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CnqlQuery {
    pub ops: Vec<CnqlOp>,
}

/// Parse a CNQL string into a structured `CnqlQuery`.
pub fn parse(input: &str) -> Result<CnqlQuery, String> {
    let input = input.trim();
    if input.is_empty() {
        return Err("Empty CNQL query".to_string());
    }

    // A real parser would use a robust lexer/PEG parser (like `pest` or `nom`).
    // This is a naive regex/string-splitting parser for the MVP.
    let segments: Vec<&str> = input.split("->").map(|s| s.trim()).collect();

    if segments.is_empty() {
        return Err("No operations found".to_string());
    }

    let mut ops = Vec::new();

    for (i, segment) in segments.iter().enumerate() {
        let op = if i == 0 {
            parse_find(segment)?
        } else {
            parse_pipeline_op(segment)?
        };
        ops.push(op);
    }

    Ok(CnqlQuery { ops })
}

/// Parse `find <label>(<filters...>)` or `find id("123")`
fn parse_find(segment: &str) -> Result<CnqlOp, String> {
    let segment = segment.trim();

    if !segment.starts_with("find") {
        return Err(format!("Expected 'find' keyword, got: '{}'", segment));
    }

    let after_find = segment["find".len()..].trim();

    // Fast path check: `find id("123")`
    if after_find.starts_with("id(") {
        let id_str = after_find
            .strip_prefix("id(")
            .and_then(|s| s.strip_suffix(')'))
            .map(|s| s.trim_matches('"'))
            .ok_or("Malformed find id()")?;
        return Ok(CnqlOp::FindById { id: id_str.to_string() });
    }

    // Normal path: `find User(name: "Aryan")`
    let (label, filter_str) = if let Some(paren_pos) = after_find.find('(') {
        let label = after_find[..paren_pos].trim().to_string();
        let rest = &after_find[paren_pos..];
        let close = rest.rfind(')').ok_or("Missing closing ')' in find")?;
        let filter_body = &rest[1..close];
        (label, filter_body.trim())
    } else {
        (after_find.to_string(), "")
    };

    let label = if label.is_empty() { "*".to_string() } else { label };
    let filters = if filter_str.is_empty() { vec![] } else { parse_filters(filter_str)? };

    Ok(CnqlOp::Find { label, filters })
}

/// Parse pipeline operations (`traverse`, `join`, `geo_near`, etc.)
fn parse_pipeline_op(segment: &str) -> Result<CnqlOp, String> {
    let segment = segment.trim();

    // Support legacy `get friends`
    if segment.starts_with("get ") {
        let relation = segment["get ".len()..].trim().to_string();
        return Ok(CnqlOp::Traverse {
            edge: relation,
            min_hops: 1,
            max_hops: 1,
            min_weight: 0.0,
        });
    }

    if segment.starts_with("filter ") {
        let filter_body = &segment["filter ".len()..];
        let filters = parse_filters(filter_body)?;
        if let Some(f) = filters.into_iter().next() {
            return Ok(CnqlOp::Filter { field: f.field, op: f.op, value: f.value });
        }
    }

    if segment.starts_with("limit ") {
        let n_str = segment["limit ".len()..].trim();
        let n: usize = n_str.parse().unwrap_or(10);
        return Ok(CnqlOp::Limit(n));
    }

    if segment.starts_with("traverse(") {
        // Mock parsing for MVP: `traverse(edge: "friends")`
        return Ok(CnqlOp::Traverse {
            edge: "friends".to_string(),
            min_hops: 1,
            max_hops: 3,
            min_weight: 0.0,
        });
    }

    if segment.starts_with("join(") {
        // Mock parsing for MVP
        return Ok(CnqlOp::Join {
            target: "target_table".to_string(),
            on_left: "id".to_string(),
            on_right: "target_id".to_string(),
            join_type: JoinType::Inner,
        });
    }

    if segment.starts_with("time_window(") {
        return Ok(CnqlOp::TimeWindow { size: "1h".to_string() });
    }

    if segment.starts_with("geo_near(") {
        return Ok(CnqlOp::GeoNear { lat: 0.0, lon: 0.0, radius_km: 5.0 });
    }

    if segment.starts_with("search(") {
        return Ok(CnqlOp::Search { query: "test".to_string(), fuzzy: true });
    }

    if segment.starts_with("stream(") {
        return Ok(CnqlOp::Stream { start_byte: 0, end_byte: 1024 });
    }

    Err(format!("Unknown or unimplemented pipeline operation: '{}'", segment))
}

fn parse_filters(input: &str) -> Result<Vec<Filter>, String> {
    let input = input.trim();
    if input.is_empty() { return Ok(vec![]); }

    let mut filters = Vec::new();
    let parts: Vec<&str> = input.split(',').collect(); // Very naive split for MVP

    for part in parts {
        if part.trim().is_empty() { continue; }
        filters.push(parse_single_filter(part.trim())?);
    }

    Ok(filters)
}

fn parse_single_filter(input: &str) -> Result<Filter, String> {
    let ops = [
        (">=", CompareOp::Gte),
        ("<=", CompareOp::Lte),
        ("!=", CompareOp::NotEq),
        (">", CompareOp::Gt),
        ("<", CompareOp::Lt),
        ("contains", CompareOp::Contains),
        (":", CompareOp::Eq),
        ("=", CompareOp::Eq),
    ];

    for (op_str, op) in &ops {
        if let Some(pos) = input.find(op_str) {
            let field = input[..pos].trim().to_string();
            let value_str = input[pos + op_str.len()..].trim();
            let value = parse_value(value_str)?;
            return Ok(Filter { field, op: op.clone(), value });
        }
    }
    Err(format!("Cannot parse filter: '{}'", input))
}

fn parse_value(input: &str) -> Result<CnqlValue, String> {
    let input = input.trim();
    if input.starts_with('"') && input.ends_with('"') {
        return Ok(CnqlValue::Text(input[1..input.len() - 1].to_string()));
    }
    if input == "true" { return Ok(CnqlValue::Bool(true)); }
    if input == "false" { return Ok(CnqlValue::Bool(false)); }
    if input == "null" { return Ok(CnqlValue::Null); }
    if let Ok(n) = input.parse::<f64>() { return Ok(CnqlValue::Number(n)); }
    Ok(CnqlValue::Text(input.to_string()))
}
