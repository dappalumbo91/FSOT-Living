//! Interactive query — language organ front door (not CLI-only)

use crate::language::{self, LanguageResult};
use crate::organism::Organism;

#[derive(Clone, Debug)]
pub struct QueryResult {
    pub kind: String,
    pub answer: String,
    pub intent: String,
    pub plan: String,
    pub grounded: bool,
}

/// Mutable query: dialogue, remember, forge ownership, language path.
pub fn query(org: &mut Organism, q: &str) -> QueryResult {
    let q = q.trim();
    if q.is_empty() {
        return QueryResult {
            kind: "empty".into(),
            answer: "Say something — I listen in natural language.".into(),
            intent: "empty".into(),
            plan: String::new(),
            grounded: true,
        };
    }

    let r: LanguageResult = language::respond(org, q);
    QueryResult {
        kind: r.kind,
        answer: format!(
            "{}\n\n[intent={} · plan: {}]",
            r.answer, r.intent, r.plan_summary
        ),
        intent: r.intent,
        plan: r.plan_summary,
        grounded: r.grounded,
    }
}
