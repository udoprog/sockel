use yew::prelude::*;

/// Sockel's compiled stylesheet: `style/` built with its defaults.
pub const STYLESHEET: &str = include_str!(concat!(env!("OUT_DIR"), "/style.css"));

/// The id of the one `<style>` element [`Styles`] fills, whichever
/// stylesheet it is given.
pub const STYLE_ID: &str = "sockel-styles";

trait StyleHost {
    fn has_style(&self, id: &str) -> bool;
    /// Adds the style element, or replaces its text.
    fn set_style(&self, id: &str, css: &str);
}

impl StyleHost for web_sys::Document {
    fn has_style(&self, id: &str) -> bool {
        self.get_element_by_id(id).is_some()
    }

    fn set_style(&self, id: &str, css: &str) {
        if let Some(style) = self.get_element_by_id(id) {
            style.set_text_content(Some(css));
            return;
        }
        let Some(head) = self.head() else { return };
        let Ok(style) = self.create_element("style") else {
            return;
        };
        style.set_id(id);
        style.set_text_content(Some(css));
        let _ = head.append_child(&style);
    }
}

/// An app's stylesheet (from sockel-build, sockel's CSS first) replaces
/// whatever is there; sockel's alone is added only when nothing is, so the
/// page has one stylesheet whichever `Styles` mounts first.
fn ensure_styles(host: &impl StyleHost, css: Option<&str>) {
    match css {
        Some(css) => host.set_style(STYLE_ID, css),
        None if !host.has_style(STYLE_ID) => host.set_style(STYLE_ID, STYLESHEET),
        None => {}
    }
}

#[derive(Properties, PartialEq)]
pub struct StylesProps {
    /// The app's own stylesheet from sockel-build, which holds sockel's
    /// CSS; without it, [`STYLESHEET`].
    #[prop_or_default]
    pub css: Option<&'static str>,
}

/// Puts the page's one stylesheet in `<head>` (`<style id="sockel-styles">`),
/// however many times it is mounted.
#[function_component]
pub fn Styles(props: &StylesProps) -> Html {
    use_effect_with(props.css, |css| {
        if let Some(document) = web_sys::window().and_then(|window| window.document()) {
            ensure_styles(&document, *css);
        }
    });
    Html::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[derive(Default)]
    struct Head(RefCell<Vec<(String, String)>>);

    impl StyleHost for Head {
        fn has_style(&self, id: &str) -> bool {
            self.0.borrow().iter().any(|(existing, _)| existing == id)
        }

        fn set_style(&self, id: &str, css: &str) {
            let mut styles = self.0.borrow_mut();
            match styles.iter_mut().find(|(existing, _)| existing == id) {
                Some(style) => style.1 = css.to_owned(),
                None => styles.push((id.to_owned(), css.to_owned())),
            }
        }
    }

    fn only(head: &Head) -> (String, String) {
        let styles = head.0.borrow();
        assert_eq!(styles.len(), 1);
        styles[0].clone()
    }

    #[test]
    fn the_stylesheet_holds_the_utilities_and_reserves_borders() {
        let compact: String = STYLESHEET.split_whitespace().collect();
        for rule in [
            ".sockel-focus-ring:focus-visible{outline:var(--focus-ring-width)solidvar(--focus-ring-color);",
            ".sockel-numeric{font-variant-numeric:tabular-nums;}",
            "scrollbar-gutter:stable;}",
            ".sockel-truncate{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;}",
            ".sockel-clamp-2{",
            ".sockel-clamp-4{",
            ".sockel-visually-hidden{position:absolute;",
            "@media(width>=40rem){.sockel-phone-only.sockel-phone-only{display:none;}}",
            "@media(width<40rem){.sockel-desktop-only.sockel-desktop-only{display:none;}}",
            ".sockel-card.sockel-link:focus-visible{outline:",
        ] {
            assert!(compact.contains(rule), "{rule}");
        }
        assert!(compact.contains(
            ".sockel-card{padding:var(--space-4);border:var(--card-border-width)solidvar(--card-border);"
        ));
        assert!(!STYLESHEET.contains("!important"));
    }

    #[test]
    fn injects_once() {
        let head = Head::default();
        ensure_styles(&head, None);
        ensure_styles(&head, None);
        assert_eq!(only(&head), (STYLE_ID.to_owned(), STYLESHEET.to_owned()));
    }

    #[test]
    fn an_apps_stylesheet_is_the_only_one_whichever_mounts_first() {
        let app = format!("{STYLESHEET}.app-sidebar {{}}");
        for app_first in [true, false] {
            let head = Head::default();
            if app_first {
                ensure_styles(&head, Some(&app));
                ensure_styles(&head, None);
            } else {
                ensure_styles(&head, None);
                ensure_styles(&head, Some(&app));
            }
            ensure_styles(&head, None);
            assert_eq!(only(&head), (STYLE_ID.to_owned(), app.clone()));
        }
    }
}
