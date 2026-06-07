/// CNQL Query Planner (10-Database Unified Architecture)
///
/// Converts a `CnqlQuery` AST into an ordered `QueryPlan` that the
/// server's execution engine can run against LMDB + WASM DNA.

use super::parser::{CnqlOp, CnqlQuery, CompareOp, CnqlValue, Filter, JoinType, AggFunc};

/// A resolved, executable step in the query execution pipeline
#[derive(Debug, Clone)]
pub enum PlanStep {
    // ---------------------------------------------------------
    // 1. FAST PATH (Key-Value)
    // ---------------------------------------------------------
    /// Bypass WASM completely, direct LMDB fetch
    FastPathIdLookup {
        id: String,
    },

    // ---------------------------------------------------------
    // 2. CORE / DOCUMENT
    // ---------------------------------------------------------
    ScanAll {
        label_filter: Option<String>,
        filters: Vec<Filter>,
    },
    FilterResults {
        field: String,
        op: CompareOp,
        value: CnqlValue,
    },
    Unwind {
        field: String,
    },
    Project {
        keep: Vec<String>,
    },

    // ---------------------------------------------------------
    // 3. GRAPH
    // ---------------------------------------------------------
    GraphTraverse {
        edge: String,
        min_hops: usize,
        max_hops: usize,
        min_weight: f64,
    },
    ShortestPath {
        to_node: String,
    },

    // ---------------------------------------------------------
    // 4. RELATIONAL SQL
    // ---------------------------------------------------------
    RelationalJoin {
        target: String,
        on_left: String,
        on_right: String,
        join_type: JoinType,
    },
    GroupBy {
        fields: Vec<String>,
    },
    Aggregate {
        functions: Vec<AggFunc>,
    },

    // ---------------------------------------------------------
    // 5. TIME-SERIES
    // ---------------------------------------------------------
    TimeWindow {
        size: String,
    },

    // ---------------------------------------------------------
    // 6. VECTOR / AI
    // ---------------------------------------------------------
    VectorScan {
        vector: Vec<f32>,
        metric: String,
    },

    // ---------------------------------------------------------
    // 7. FULL TEXT SEARCH
    // ---------------------------------------------------------
    FullTextSearch {
        query: String,
        fuzzy: bool,
    },

    // ---------------------------------------------------------
    // 8. GEO-SPATIAL
    // ---------------------------------------------------------
    GeoNear {
        lat: f64,
        lon: f64,
        radius_km: f64,
    },

    // ---------------------------------------------------------
    // 9. WIDE-COLUMN
    // ---------------------------------------------------------
    RangeScan {
        field: String,
        start: CnqlValue,
        end: CnqlValue,
    },

    // ---------------------------------------------------------
    // 10. BLOB / OBJECT
    // ---------------------------------------------------------
    ByteStream {
        start_byte: usize,
        end_byte: usize,
    },

    // ---------------------------------------------------------
    // UTILITY
    // ---------------------------------------------------------
    Limit(usize),
    SortBy {
        field: String,
        ascending: bool,
    },
}

/// The full execution plan for a CNQL query
#[derive(Debug, Clone)]
pub struct QueryPlan {
    pub steps: Vec<PlanStep>,
    /// If true, the execution engine bypasses WASM and hits LMDB directly
    pub is_fast_path: bool,
    /// Maximum number of results to return (default: 100)
    pub limit: usize,
}

impl QueryPlan {
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            is_fast_path: false,
            limit: 100,
        }
    }
}

/// Build an executable `QueryPlan` from a parsed `CnqlQuery`
pub fn build_plan(query: &CnqlQuery) -> Result<QueryPlan, String> {
    let mut plan = QueryPlan::new();

    for op in &query.ops {
        match op {
            CnqlOp::FindById { id } => {
                plan.is_fast_path = true;
                plan.steps.push(PlanStep::FastPathIdLookup { id: id.clone() });
            }
            CnqlOp::Find { label, filters } => {
                let label_filter = if label == "*" { None } else { Some(label.clone()) };
                plan.steps.push(PlanStep::ScanAll {
                    label_filter,
                    filters: filters.clone(),
                });
            }
            CnqlOp::Filter { field, op, value } => {
                plan.steps.push(PlanStep::FilterResults {
                    field: field.clone(),
                    op: op.clone(),
                    value: value.clone(),
                });
            }
            CnqlOp::Unwind { field } => plan.steps.push(PlanStep::Unwind { field: field.clone() }),
            CnqlOp::Project { keep } => plan.steps.push(PlanStep::Project { keep: keep.clone() }),
            
            CnqlOp::Traverse { edge, min_hops, max_hops, min_weight } => {
                plan.steps.push(PlanStep::GraphTraverse {
                    edge: edge.clone(),
                    min_hops: *min_hops,
                    max_hops: *max_hops,
                    min_weight: *min_weight,
                });
            }
            CnqlOp::ShortestPath { to_node } => plan.steps.push(PlanStep::ShortestPath { to_node: to_node.clone() }),
            
            CnqlOp::Join { target, on_left, on_right, join_type } => {
                plan.steps.push(PlanStep::RelationalJoin {
                    target: target.clone(),
                    on_left: on_left.clone(),
                    on_right: on_right.clone(),
                    join_type: join_type.clone(),
                });
            }
            CnqlOp::GroupBy { fields } => plan.steps.push(PlanStep::GroupBy { fields: fields.clone() }),
            CnqlOp::Aggregate { functions } => plan.steps.push(PlanStep::Aggregate { functions: functions.clone() }),
            
            CnqlOp::TimeWindow { size } => plan.steps.push(PlanStep::TimeWindow { size: size.clone() }),
            
            CnqlOp::SimilarTo { vector, metric } => plan.steps.push(PlanStep::VectorScan { vector: vector.clone(), metric: metric.clone() }),
            
            CnqlOp::Search { query, fuzzy } => plan.steps.push(PlanStep::FullTextSearch { query: query.clone(), fuzzy: *fuzzy }),
            
            CnqlOp::GeoNear { lat, lon, radius_km } => plan.steps.push(PlanStep::GeoNear { lat: *lat, lon: *lon, radius_km: *radius_km }),
            
            CnqlOp::RangeScan { field, start, end } => plan.steps.push(PlanStep::RangeScan { field: field.clone(), start: start.clone(), end: end.clone() }),
            
            CnqlOp::Stream { start_byte, end_byte } => plan.steps.push(PlanStep::ByteStream { start_byte: *start_byte, end_byte: *end_byte }),
            
            CnqlOp::Limit(n) => {
                plan.limit = *n;
                plan.steps.push(PlanStep::Limit(*n));
            }
            CnqlOp::SortBy { field, ascending } => {
                plan.steps.push(PlanStep::SortBy {
                    field: field.clone(),
                    ascending: *ascending,
                });
            }
        }
    }

    if plan.steps.is_empty() {
        return Err("CNQL plan has no executable steps".to_string());
    }

    Ok(plan)
}

// ============================================================
// Unit Tests
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cnql::parser::parse;

    #[test]
    fn test_plan_fast_path() {
        let q = parse(r#"find id("123")"#).unwrap();
        let plan = build_plan(&q).unwrap();
        assert!(plan.is_fast_path);
        assert!(matches!(plan.steps[0], PlanStep::FastPathIdLookup { .. }));
    }

    #[test]
    fn test_plan_graph_traversal() {
        let q = parse(r#"find User -> traverse(edge: "friends")"#).unwrap();
        let plan = build_plan(&q).unwrap();
        assert_eq!(plan.steps.len(), 2);
        assert!(matches!(plan.steps[1], PlanStep::GraphTraverse { .. }));
    }

    #[test]
    fn test_plan_time_window() {
        let q = parse(r#"find * -> time_window(size: "5m")"#).unwrap();
        let plan = build_plan(&q).unwrap();
        assert!(matches!(plan.steps[1], PlanStep::TimeWindow { .. }));
    }
}
