//! Pure Google Sheets address helpers; transport belongs to the caller.
pub fn tab_range(title: &str) -> String {
    format!("'{}'", title.replace('\'', "''"))
}
pub fn to_csv_url(url: &str) -> String {
    let u = url.trim();
    if ["output=csv", "format=csv", "tqx=out:csv"]
        .iter()
        .any(|s| u.contains(s))
    {
        return u.into();
    }
    let gid_re = regex::Regex::new(r"[#&?]gid=(\d+)").unwrap();
    let gid = gid_re.captures(u).map(|c| c[1].to_owned());
    for (pattern, published) in [
        (r"docs\.google\.com/spreadsheets/d/e/([\w-]+)", true),
        (r"docs\.google\.com/spreadsheets/d/([\w-]+)", false),
    ] {
        if let Some(c) = regex::Regex::new(pattern).unwrap().captures(u) {
            let mut result = if published {
                format!(
                    "https://docs.google.com/spreadsheets/d/e/{}/pub?output=csv",
                    &c[1]
                )
            } else {
                format!(
                    "https://docs.google.com/spreadsheets/d/{}/export?format=csv",
                    &c[1]
                )
            };
            if let Some(gid) = gid {
                if published {
                    result.push_str("&single=true");
                }
                result.push_str(&format!("&gid={gid}"));
            }
            return result;
        }
    }
    u.into()
}
