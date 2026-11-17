use yew::prelude::*;

/// Plinth's compiled stylesheet: `style/` built with its defaults.
pub const STYLESHEET: &str = include_str!(concat!(env!("OUT_DIR"), "/plinth.css"));

const STYLE_ID: &str = "plinth-styles";

trait StyleHost {
    fn has_style(&self, id: &str) -> bool;
    fn add_style(&self, id: &str, css: &str);
}

impl StyleHost for web_sys::Document {
    fn has_style(&self, id: &str) -> bool {
        self.get_element_by_id(id).is_some()
    }

    fn add_style(&self, id: &str, css: &str) {
        let Some(head) = self.head() else { return };
        let Ok(style) = self.create_element("style") else {
            return;
        };
        style.set_id(id);
        style.set_text_content(Some(css));
        let _ = head.append_child(&style);
    }
}

fn ensure_styles(host: &impl StyleHost) {
    if !host.has_style(STYLE_ID) {
        host.add_style(STYLE_ID, STYLESHEET);
    }
}

/// Adds [`STYLESHEET`] to `<head>` once, however many times it is mounted.
#[function_component]
pub fn Styles() -> Html {
    use_effect_with((), |_| {
        if let Some(document) = web_sys::window().and_then(|window| window.document()) {
            ensure_styles(&document);
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

        fn add_style(&self, id: &str, css: &str) {
            self.0.borrow_mut().push((id.to_owned(), css.to_owned()));
        }
    }

    #[test]
    fn injects_once() {
        let head = Head::default();
        ensure_styles(&head);
        ensure_styles(&head);
        let styles = head.0.borrow();
        assert_eq!(styles.len(), 1);
        assert_eq!(styles[0], (STYLE_ID.to_owned(), STYLESHEET.to_owned()));
    }
}
