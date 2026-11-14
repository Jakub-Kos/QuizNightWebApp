// Remove when the show uses this module.
#![allow(dead_code)]

use leptos::prelude::*;
use quiz_core::theme::{
    Theme,
    builtin::{style_classes, theme_by_id, theme_style},
};
use std::collections::BTreeMap;

/// Optional asset resolver owned by the loaded show. Editor previews draw their own image.
#[derive(Clone, Copy)]
pub struct ThemeAssets(pub Callback<String, Option<String>>);

/// Serialize CSS variables deterministically without changing quoted font names.
fn inline_style(variables: &BTreeMap<String, String>) -> String {
    variables
        .iter()
        .map(|(key, value)| format!("{key}: {value};"))
        .collect::<Vec<_>>()
        .join(" ")
}

#[component]
pub fn ThemeScope(
    #[prop(into)] theme: Signal<Theme>,
    #[prop(optional, into)] class: String,
    children: Children,
) -> impl IntoView {
    provide_context(theme);
    // LoadedShow installs pack fonts; SceneBackdrop resolves the background through quiz media.
    view! {
        <div class=move || format!("{} {}", class, style_classes(&theme.get()))
            style=move || inline_style(&theme_style(&theme.get()))>
            {children()}
        </div>
    }
}

pub fn use_theme() -> Signal<Theme> {
    use_context::<Signal<Theme>>().unwrap_or_else(|| Signal::stored(theme_by_id("default").clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classic_has_no_inline_style() {
        assert_eq!(inline_style(&theme_style(theme_by_id("default"))), "");
    }

    #[test]
    fn themed_variables_are_sorted_and_preserve_quotes() {
        let variables = theme_style(theme_by_id("halloween"));
        assert!(!variables.is_empty());
        let style = inline_style(&variables);
        assert_eq!(style, inline_style(&variables));
        let keys: Vec<_> = variables.keys().collect();
        assert!(keys.windows(2).all(|pair| pair[0] < pair[1]));
        for (key, value) in &variables {
            assert!(style.contains(&format!("{key}: {value};")));
        }
        assert!(style.contains('"'));
    }
}
