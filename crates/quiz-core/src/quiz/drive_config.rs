//! Version-two quiznight.json and scores-tab reconciliation. Missing JSON keys differ from null.
use super::{
    model::Quiz,
    parse::{TEAM_COLUMN, TEAM_ID_HEADER},
};
use serde_json::{Map, Value, json};
use std::collections::HashSet;
pub const CONFIG_NAME: &str = "quiznight.json";
pub fn merge3(
    base: Option<&Value>,
    local: Option<&Value>,
    remote: Option<&Value>,
) -> Option<Value> {
    if let (Some(Value::Object(l)), Some(Value::Object(r))) = (local, remote) {
        let mut out = Map::new();
        for k in l.keys().chain(r.keys()) {
            if let Some(v) = merge3(base.and_then(|b| b.get(k)), l.get(k), r.get(k)) {
                out.insert(k.clone(), v);
            }
        }
        return Some(Value::Object(out));
    }
    if local == base {
        remote.cloned()
    } else {
        local.cloned()
    }
}
pub fn overlay(local: &Value, remote: &Value) -> Value {
    if let (Some(l), Some(r)) = (local.as_object(), remote.as_object()) {
        let mut o = l.clone();
        for (k, v) in r {
            o.insert(k.clone(), overlay(l.get(k).unwrap_or(&Value::Null), v));
        }
        Value::Object(o)
    } else {
        remote.clone()
    }
}
pub fn upgrade(config: Value) -> Value {
    if config.is_null() || config["version"] == 2 {
        return config;
    }
    let mut c = config.clone();
    let mut t = Map::new();
    if let Some(m) = config["teams"].as_object() {
        for (n, v) in m {
            let mut v = v.clone();
            v["name"] = json!(n);
            t.insert(format!("name:{}", lower(n)), v);
        }
    }
    c["version"] = json!(2);
    c["teams"] = json!(t);
    c
}
fn lower(s: &str) -> String {
    s.trim().to_lowercase()
}
pub fn config_from_quiz(q: &Quiz) -> Value {
    let mut teams = Map::new();
    if let Some(a) = q.fields.get("teamArchive").and_then(Value::as_object) {
        for (id, v) in a {
            let mut v = v.clone();
            v["removed"] = json!(true);
            teams.insert(id.clone(), v);
        }
    }
    for t in &q.teams {
        if t.fields.get("draft") == Some(&json!(true)) || t.name.trim().is_empty() {
            continue;
        }
        let mut v = serde_json::to_value(t).unwrap();
        let id = match v["id"].clone() {
            Value::String(s) => s,
            v if v.is_null() => "undefined".into(),
            v => v.to_string(),
        };
        v.as_object_mut().unwrap().remove("id");
        v["name"] = json!(t.name.trim());
        teams.insert(id, v);
    }
    json!({"version":2,"title":q.title,"settings":q.settings,"rules":q.rules,"prizes":q.prizes,"customTheme":q.fields.get("customTheme").unwrap_or(&Value::Null),"teams":teams})
}
pub fn apply_config(q: &Quiz, c: &Value, order: Option<&[String]>) -> Quiz {
    let mut v = serde_json::to_value(q).unwrap();
    for k in ["title", "rules", "prizes"] {
        if !c[k].is_null() {
            v[k] = c[k].clone();
        }
    }
    if let Some(m) = c["settings"].as_object() {
        for (k, x) in m {
            v["settings"][k] = x.clone();
        }
    }
    if let Some(t) = c.get("customTheme") {
        v["customTheme"] = if t.is_null() {
            Value::Null
        } else {
            let n = crate::theme::format::normalize_theme_with_options(
                t,
                &crate::theme::format::NormalizeOptions {
                    stored: true,
                    ..Default::default()
                },
            );
            serde_json::to_value(n.theme).unwrap_or(Value::Null)
        };
    }
    let teams = c["teams"].as_object().cloned().unwrap_or_default();
    let shown = order.map(|o| o.to_vec()).unwrap_or_else(|| {
        teams
            .iter()
            .filter(|(_, v)| v["removed"] != true)
            .map(|(k, _)| k.clone())
            .collect()
    });
    let mut active = vec![];
    let mut archive = Map::new();
    for (id, entry) in &teams {
        let mut e = entry.clone();
        e.as_object_mut().unwrap().remove("removed");
        if !shown.contains(id) {
            archive.insert(id.clone(), e);
        }
    }
    for id in shown {
        if let Some(e) = teams.get(&id) {
            let mut e = e.clone();
            e.as_object_mut().unwrap().remove("removed");
            e["id"] = json!(id);
            active.push(e);
        }
    }
    active.extend(
        q.teams
            .iter()
            .filter(|t| t.fields.get("draft") == Some(&json!(true)))
            .map(|t| serde_json::to_value(t).unwrap()),
    );
    v["teams"] = json!(active);
    v["teamArchive"] = json!(archive);
    serde_json::from_value(v).expect("valid quiz configuration")
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ScoreRow {
    pub row: usize,
    pub name: String,
    pub id: Option<String>,
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoresTable {
    pub header: usize,
    pub name_col: usize,
    pub id_col: isize,
    pub width: usize,
    pub end: usize,
    pub rows: Vec<ScoreRow>,
    pub empty: Vec<usize>,
}
pub fn scores_table(values: &[Vec<String>]) -> Option<ScoresTable> {
    let re = regex::Regex::new(TEAM_COLUMN).unwrap();
    let header = values
        .iter()
        .position(|r| r.iter().any(|c| re.is_match(c)))?;
    let name_col = values[header].iter().position(|c| re.is_match(c)).unwrap();
    let id_col = values[header]
        .iter()
        .position(|c| c == TEAM_ID_HEADER)
        .map_or(-1, |i| i as isize);
    let mut rows = vec![];
    let mut empty = vec![];
    for (r, v) in values.iter().enumerate().skip(header + 1) {
        let name = v.get(name_col).map_or("", |s| s.trim());
        if name.is_empty() {
            empty.push(r)
        } else {
            rows.push(ScoreRow {
                row: r,
                name: name.into(),
                id: usize::try_from(id_col)
                    .ok()
                    .and_then(|i| v.get(i))
                    .map(|s| s.trim().to_owned())
                    .filter(|s| !s.is_empty()),
            });
        }
    }
    Some(ScoresTable {
        header,
        name_col,
        id_col,
        width: values.iter().map(Vec::len).max().unwrap_or(0),
        end: values.len(),
        rows,
        empty,
    })
}
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Ops {
    pub ids: Vec<Value>,
    pub names: Vec<Value>,
    pub clears: Vec<usize>,
    pub appends: Vec<Value>,
}
pub struct Reconciled {
    pub entries: Value,
    pub order: Vec<String>,
    pub ops: Ops,
}
pub fn reconcile(
    rows: &[ScoreRow],
    entries: &Value,
    base: Option<&Value>,
    local: &Value,
    can_write: bool,
    new_id: &mut impl FnMut() -> String,
) -> Reconciled {
    let mut entries = entries.as_object().cloned().unwrap_or_default();
    let local = local.as_object().cloned().unwrap_or_default();
    let active = |v: Option<&Value>| v.is_some_and(|e| e["removed"] != true);
    let deleted: HashSet<_> = base
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(|m| m.iter())
        .filter(|(id, e)| active(Some(e)) && !local.contains_key(*id))
        .map(|(id, _)| id.clone())
        .collect();
    let added: Vec<_> = local
        .iter()
        .filter(|(id, e)| base.is_some() && active(Some(e)) && base.unwrap().get(*id).is_none())
        .map(|(id, _)| id.clone())
        .collect();
    let row_ids: HashSet<_> = rows.iter().filter_map(|r| r.id.clone()).collect();
    let mut used = HashSet::new();
    let mut order = vec![];
    let mut ops = Ops::default();
    for r in rows {
        let id = if let Some(id) = r.id.as_ref().filter(|id| !used.contains(*id)) {
            id.clone()
        } else {
            let candidates: Vec<_> = entries
                .iter()
                .filter(|(id, e)| {
                    !used.contains(*id)
                        && !row_ids.contains(*id)
                        && !added.contains(id)
                        && lower(e["name"].as_str().unwrap_or("")) == lower(&r.name)
                })
                .map(|(id, _)| id.clone())
                .collect();
            let mut id = candidates
                .iter()
                .find(|id| entries[*id]["removed"] == true)
                .or(candidates.first())
                .cloned()
                .unwrap_or_else(|| {
                    if can_write {
                        new_id()
                    } else {
                        format!("name:{}", lower(&r.name))
                    }
                });
            if can_write && id.starts_with("name:") {
                let real = new_id();
                if let Some(e) = entries.remove(&id) {
                    entries.insert(real.clone(), e);
                }
                id = real;
            }
            if can_write {
                ops.ids.push(json!({"row":r.row,"id":id}));
            }
            id
        };
        used.insert(id.clone());
        if can_write && deleted.contains(&id) {
            ops.clears.push(r.row);
            let e = entries
                .entry(id.clone())
                .or_insert_with(|| base.unwrap()[&id].clone());
            e["removed"] = json!(true);
            continue;
        }
        let renamed = can_write
            && base.is_some_and(|b| {
                active(b.get(&id)) && active(local.get(&id)) && local[&id]["name"] != b[&id]["name"]
            });
        let name = if renamed {
            local[&id]["name"].as_str().unwrap_or(&r.name)
        } else {
            &r.name
        };
        if name != r.name {
            ops.names.push(json!({"row":r.row,"name":name}));
        }
        let e = entries.entry(id.clone()).or_insert(json!({}));
        e["name"] = json!(name);
        e.as_object_mut().unwrap().remove("removed");
        order.push(id);
    }
    if can_write {
        for id in added {
            if used.contains(&id) {
                continue;
            }
            let name = local[&id]["name"].clone();
            let archived = entries
                .iter()
                .find(|(other, e)| {
                    **other != id
                        && !used.contains(*other)
                        && e["removed"] == true
                        && lower(e["name"].as_str().unwrap_or(""))
                            == lower(name.as_str().unwrap_or(""))
                })
                .map(|(id, _)| id.clone());
            let target = archived.unwrap_or_else(|| id.clone());
            if target != id {
                let mut details = entries[&target].clone();
                details.as_object_mut().unwrap().remove("removed");
                let mut e = local[&id].clone();
                for (k, v) in details.as_object().unwrap() {
                    e[k] = v.clone();
                }
                e["name"] = name.clone();
                entries.insert(target.clone(), e);
                entries.remove(&id);
            }
            ops.appends.push(json!({"name":name,"id":target}));
            used.insert(target.clone());
            order.push(target);
        }
    }
    for (id, e) in &mut entries {
        if !used.contains(id) {
            e["removed"] = json!(true);
        }
    }
    Reconciled {
        entries: json!(entries),
        order,
        ops,
    }
}
pub fn column(mut i: usize) -> String {
    let mut s = String::new();
    loop {
        s.insert(0, (b'A' + (i % 26) as u8) as char);
        if i < 26 {
            break;
        }
        i = i / 26 - 1;
    }
    s
}
pub fn scores_writes(s: &ScoresTable, sheet_id: i64, name: &str, ops: &Ops) -> Value {
    let id_col = if s.id_col < 0 {
        s.width
    } else {
        s.id_col as usize
    };
    let mut cells = vec![];
    let mut structure = vec![];
    let mut set = |row: usize, col: usize, v: Value| {
        cells.push(json!({"range":format!("{name}!{}{}",column(col),row+1),"values":[[v]]}))
    };
    if s.id_col < 0 {
        set(s.header, id_col, json!(TEAM_ID_HEADER));
        structure.push(json!({"updateDimensionProperties":{"range":{"sheetId":sheet_id,"dimension":"COLUMNS","startIndex":id_col,"endIndex":id_col+1},"properties":{"hiddenByUser":true},"fields":"hiddenByUser"}}));
    }
    for v in &ops.ids {
        set(v["row"].as_u64().unwrap() as usize, id_col, v["id"].clone());
    }
    for v in &ops.names {
        set(
            v["row"].as_u64().unwrap() as usize,
            s.name_col,
            v["name"].clone(),
        );
    }
    for r in &ops.clears {
        set(*r, s.name_col, json!(""));
        set(*r, id_col, json!(""));
    }
    if ops.appends.len() > s.empty.len() {
        structure.push(json!({"insertDimension":{"range":{"sheetId":sheet_id,"dimension":"ROWS","startIndex":s.end,"endIndex":s.end+ops.appends.len()-s.empty.len()},"inheritFromBefore":s.end>s.header+1}}));
    }
    let mut next = s.end;
    for (i, v) in ops.appends.iter().enumerate() {
        let row = s.empty.get(i).copied().unwrap_or_else(|| {
            let r = next;
            next += 1;
            r
        });
        set(row, s.name_col, v["name"].clone());
        set(row, id_col, v["id"].clone());
    }
    json!({"structure":structure,"cells":cells})
}
