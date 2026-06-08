///  Cluaiz Database Query Language (CDQL)
/// 
/// A universal, Python-like query language that works uniformly
/// across ALL 9 database paradigms (Document, Graph, SQL, Vector, etc.)
///
/// # Syntax Examples
/// ```text
/// find *(name: "Aryan")                          // Document search
/// find *(name: "Aryan") -> get friends           // Graph traversal
/// find * -> filter age > 18 -> limit 10          // Filtered scan
/// find * -> similar_to([0.1, 0.2]) -> limit 5   // Vector similarity
/// ```
pub mod parser;
pub mod planner;

pub use parser::{parse, CdqlQuery};
pub use planner::{build_plan, QueryPlan};
