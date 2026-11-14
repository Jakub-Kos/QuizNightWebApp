use super::model::{Prizes, Quiz};
pub const DEFAULT_RULE_KEYS: [&str; 4] = ["rule_1", "rule_2", "rule_3", "rule_4"];
pub const MAX_RULES: usize = 8;
pub fn rules_for(quiz: &Quiz, default_rules: &[String]) -> Option<Vec<String>> {
    if !quiz.rules.enabled {
        return None;
    }
    let items = quiz
        .rules
        .items
        .as_deref()
        .unwrap_or(default_rules)
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    (!items.is_empty()).then_some(items)
}
pub fn prizes_for(quiz: &Quiz) -> Option<Prizes> {
    let p = &quiz.prizes;
    p.enabled.then(|| Prizes {
        enabled: true,
        first: p.first.trim().into(),
        second: p.second.trim().into(),
        third: p.third.trim().into(),
        extra: p.extra.trim().into(),
    })
}
