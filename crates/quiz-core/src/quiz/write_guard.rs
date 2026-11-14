//! Google write classification and browser-local authorization. Never authorizes deletion.
use serde_json::Value;
use std::{collections::HashSet, future::Future};
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Edit,
    Create,
    Folder,
    Trash,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub kind: Kind,
    pub ids: Vec<String>,
}
pub fn body_parents(body: &str) -> Vec<String> {
    regex::Regex::new(r#""parents"\s*:\s*\[([^\]]*)\]"#)
        .unwrap()
        .captures(body)
        .map(|c| {
            regex::Regex::new(r#""([A-Za-z0-9_-]+)""#)
                .unwrap()
                .captures_iter(&c[1])
                .map(|m| m[1].into())
                .collect()
        })
        .unwrap_or_default()
}
pub fn write_targets(method: &str, url: &str, body: &str) -> Result<Option<Target>, String> {
    let method = method.to_uppercase();
    if method == "GET" || method == "HEAD" {
        return Ok(None);
    }
    let target = |kind, ids| Ok(Some(Target { kind, ids }));
    if let Some(c) = regex::Regex::new(
        r"^https://sheets\.googleapis\.com/v4/spreadsheets/([A-Za-z0-9_-]+)(?:[/:?]|$)",
    )
    .unwrap()
    .captures(url)
    {
        return if method == "POST" || method == "PUT" {
            target(Kind::Edit, vec![c[1].into()])
        } else {
            Err("sheet_method".into())
        };
    }
    let re=regex::Regex::new(r"^https://www\.googleapis\.com/(upload/)?drive/v3/files(?:/([A-Za-z0-9_-]+))?(/permissions(?:/[A-Za-z0-9_-]+)?|/copy)?(?:[?#]|$)").unwrap();
    let c = re.captures(url).ok_or("unknown_url")?;
    let sub = c.get(3).map_or("", |m| m.as_str());
    if method == "DELETE" && !sub.starts_with("/permissions") {
        return Err("delete".into());
    }
    let parents = body_parents(body);
    if c.get(2).is_none() {
        if method != "POST" {
            return Err("unknown_method".into());
        }
        if c.get(1).is_none()
            && body.contains("\"application/vnd.google-apps.folder\"")
            && parents.len() <= 1
        {
            return target(Kind::Folder, parents);
        }
        return if parents.is_empty() {
            Err("no_parent".into())
        } else {
            target(Kind::Create, parents)
        };
    }
    if sub == "/copy" {
        return if parents.is_empty() {
            Err("no_parent".into())
        } else {
            target(Kind::Create, parents)
        };
    }
    let ids = vec![c[2].into()];
    if !sub.is_empty() {
        return target(Kind::Edit, ids);
    }
    if method != "PATCH" {
        return Err("unknown_method".into());
    }
    target(
        if regex::Regex::new(r#""trashed"\s*:\s*true"#)
            .unwrap()
            .is_match(body)
        {
            Kind::Trash
        } else {
            Kind::Edit
        },
        ids,
    )
}
pub fn quiz_file_ids(quiz: &Value) -> Vec<String> {
    ["folderId", "spreadsheetId", "mediaFolderId", "configFileId"]
        .iter()
        .filter_map(|k| {
            quiz["drive"][k]
                .as_str()
                .filter(|v| !v.is_empty())
                .map(str::to_owned)
        })
        .collect()
}
#[derive(Default)]
pub struct WriteGuard {
    created: HashSet<String>,
    inside: HashSet<String>,
    known: HashSet<String>,
    my_drive: Option<String>,
}
impl WriteGuard {
    pub fn created(&mut self, id: &str) {
        if !id.is_empty() {
            self.created.insert(id.into());
        }
    }
    pub async fn check<R, RF, P, PF, M, MF>(
        &mut self,
        method: &str,
        url: &str,
        body: &str,
        roots: R,
        parents: P,
        root: M,
    ) -> Result<Option<Target>, String>
    where
        R: FnOnce() -> RF,
        RF: Future<Output = Vec<String>>,
        P: Fn(String) -> PF,
        PF: Future<Output = Vec<String>>,
        M: FnOnce() -> MF,
        MF: Future<Output = Option<String>>,
    {
        let Some(t) = write_targets(method, url, body)? else {
            return Ok(None);
        };
        if t.kind == Kind::Folder {
            return Ok(Some(t));
        }
        if t.kind == Kind::Trash {
            return if self.created.contains(&t.ids[0]) {
                Ok(Some(t))
            } else {
                Err("trash".into())
            };
        }
        if t.ids.iter().any(|id| !self.known.contains(id)) {
            self.known.extend(roots().await);
        }
        if self.my_drive.is_none() {
            self.my_drive = root().await;
        }
        for id in &t.ids {
            if self.created.contains(id) || self.inside.contains(id) {
                continue;
            }
            let is_root = |p: &String| self.my_drive.as_ref() == Some(p);
            if !is_root(id) && self.known.contains(id) {
                continue;
            }
            if is_root(id) {
                return Err(format!("outside:{id}"));
            }
            let mut level = vec![id.clone()];
            let mut seen = HashSet::from([id.clone()]);
            let mut allowed = false;
            for _ in 0..8 {
                let mut next = vec![];
                for p in level {
                    for p in parents(p).await {
                        if is_root(&p) {
                            continue;
                        }
                        if self.known.contains(&p)
                            || self.created.contains(&p)
                            || self.inside.contains(&p)
                        {
                            allowed = true;
                        }
                        if seen.insert(p.clone()) {
                            next.push(p);
                        }
                    }
                }
                if allowed || next.is_empty() {
                    break;
                }
                level = next;
            }
            if !allowed {
                return Err(format!("outside:{id}"));
            }
            self.inside.insert(id.clone());
        }
        Ok(Some(t))
    }
}
