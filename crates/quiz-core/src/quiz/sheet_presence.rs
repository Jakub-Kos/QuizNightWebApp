//! Sheet editor presence stored as Drive appProperties, compatible with the React client.
use serde_json::{Value, json};
pub const PREFIX: &str = "qn_editing_";
#[derive(Debug, Clone, PartialEq)]
pub struct Presence {
    pub key: String,
    pub at: f64,
    pub question: String,
    pub typing: bool,
    pub name: String,
}
pub fn presence_key(email: &str) -> String {
    let mut h = 2166136261u32;
    for ch in email.chars() {
        h = (h ^ ch as u32).wrapping_mul(16777619);
    }
    let mut n = h;
    let mut s = String::new();
    loop {
        s.insert(0, char::from_digit(n % 36, 36).unwrap());
        n /= 36;
        if n == 0 {
            break;
        }
    }
    format!("{PREFIX}{s}")
}
pub fn read_presence(props: &Value, now_seconds: f64) -> Vec<Presence> {
    props
        .as_object()
        .into_iter()
        .flatten()
        .filter(|(k, _)| k.starts_with(PREFIX))
        .filter_map(|(k, v)| {
            let s = v.as_str()?;
            let parts = s.split('|').collect::<Vec<_>>();
            let at = parts.first()?.parse::<f64>().ok()?;
            if !at.is_finite() || now_seconds - at >= 60. {
                return None;
            }
            Some(Presence {
                key: k.clone(),
                at,
                question: parts.get(1).unwrap_or(&"").to_string(),
                typing: parts.get(2) == Some(&"1"),
                name: parts.get(3..).unwrap_or_default().join("|"),
            })
        })
        .collect()
}
pub fn presence_properties(
    key: &str,
    entry: Option<&Presence>,
    existing: &Value,
    now_seconds: f64,
) -> Value {
    let mut props = json!({});
    props[key] = if let Some(e) = entry {
        let s = format!(
            "{}|{}|{}|{}",
            now_seconds.round(),
            e.question,
            if e.typing { 1 } else { 0 },
            e.name
        );
        let limit = 124usize.saturating_sub(key.len() + 1);
        let mut length = 0;
        json!(
            s.chars()
                .take_while(|c| {
                    length += c.len_utf16();
                    length <= limit
                })
                .collect::<String>()
        )
    } else {
        Value::Null
    };
    if entry.is_some() {
        if let Some(m) = existing.as_object() {
            for (k, v) in m {
                if k != key
                    && k.starts_with(PREFIX)
                    && v.as_str()
                        .and_then(|s| s.split('|').next())
                        .and_then(|s| s.parse::<f64>().ok())
                        .is_some_and(|at| now_seconds - at > 600.)
                {
                    props[k] = Value::Null;
                }
            }
        }
    }
    props
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hashes() {
        assert_eq!(presence_key(""), "qn_editing_ztntfp");
        assert_eq!(presence_key("test@example.com"), "qn_editing_1dblcg8");
        assert_ne!(presence_key("a@b"), presence_key("A@b"));
    }
    #[test]
    fn freshness_names_and_typing() {
        let p = read_presence(
            &json!({"qn_editing_a":"950|q1|1|A|B","qn_editing_b":"940|q2|0|Gone","qn_editing_c":"bad|q|0|bad","other":"999|q|1|other"}),
            1000.,
        );
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].name, "A|B");
        assert!(p[0].typing);
    }
    #[test]
    fn payload_expiry_and_leaving() {
        let e = Presence {
            key: "qn_editing_a".into(),
            at: 0.,
            question: "q1".into(),
            typing: true,
            name: "Name".into(),
        };
        let props = presence_properties(
            &e.key,
            Some(&e),
            &json!({"qn_editing_old":"300|q|0|Old","qn_editing_recent":"999|q|0|Recent","other":"1"}),
            1000.4,
        );
        assert_eq!(props[&e.key], "1000|q1|1|Name");
        assert_eq!(props["qn_editing_old"], Value::Null);
        assert!(props.get("qn_editing_recent").is_none());
        assert!(props.get("other").is_none());
        assert_eq!(
            presence_properties(&e.key, None, &json!({}), 1000.),
            json!({"qn_editing_a":null})
        );
        let mut e = e;
        e.name = "é".repeat(300);
        let props = presence_properties(&e.key, Some(&e), &json!({}), 1.);
        assert_eq!(
            props[&e.key].as_str().unwrap().encode_utf16().count(),
            124 - e.key.len() - 1
        );
    }
}
