use std::collections::{HashMap, HashSet};
fn words(s: &str) -> Vec<&str> {
    s.split_whitespace().collect()
}
fn letter(c: char) -> bool {
    c.is_alphabetic() || c.is_numeric()
}
pub fn team_initials(name: &str) -> String {
    let all = words(name);
    let filtered: Vec<_> = all
        .iter()
        .copied()
        .filter(|w| w.chars().any(letter))
        .collect();
    let ws = if filtered.is_empty() { all } else { filtered };
    if ws.is_empty() {
        return "?".into();
    }
    let chars = |w: &str| {
        let s = w.trim_start_matches(|c| !letter(c));
        if s.is_empty() { w } else { s }.chars().collect::<Vec<_>>()
    };
    let a = chars(ws[0]);
    let result = if ws.len() == 1 {
        a.into_iter().take(2).collect::<String>()
    } else {
        [a[0], chars(ws[1])[0]].iter().collect()
    };
    result.to_uppercase()
}
pub fn unique_initials(names: &[String]) -> Vec<String> {
    let base: Vec<_> = names.iter().map(|n| team_initials(n)).collect();
    let mut counts = HashMap::new();
    for b in &base {
        *counts.entry(b).or_insert(0) += 1;
    }
    let mut used: HashSet<String> = base.iter().filter(|b| counts[b] == 1).cloned().collect();
    names
        .iter()
        .enumerate()
        .map(|(i, n)| {
            if counts[&base[i]] == 1 {
                return base[i].clone();
            }
            let ws: Vec<Vec<char>> = words(n)
                .iter()
                .map(|w| w.chars().filter(|c| letter(*c)).collect::<Vec<_>>())
                .filter(|w| !w.is_empty())
                .collect();
            let mut candidates = vec![base[i].clone()];
            if ws.len() >= 3 {
                candidates.push(ws[..3].iter().map(|w| w[0]).collect());
            }
            if ws.len() > 1 && ws.last().unwrap().len() <= 3 {
                candidates.push(format!(
                    "{}{}",
                    ws[0][0],
                    ws.last().unwrap().iter().collect::<String>()
                ));
            }
            let all: Vec<char> = ws.concat();
            for c in all.iter().skip(1) {
                candidates.push(format!("{}{}", all[0], c));
            }
            let pick = candidates
                .into_iter()
                .map(|s| s.to_uppercase())
                .find(|s| !used.contains(s))
                .unwrap_or_else(|| format!("{}{}", base[i], i + 1));
            used.insert(pick.clone());
            pick
        })
        .collect()
}
