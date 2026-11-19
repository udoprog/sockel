//! The attribute the components write their test ids to.
//!
//! `data-testid` unless the app builds with `SOCKEL_TEST_ID_ATTR` set (see the
//! crate docs). `html!` takes no attribute name from an expression, so a
//! component builds its element and [`with_testid`] adds the attribute to it
//! before it is returned, so the attribute is on the element when it mounts.

use std::rc::Rc;
use yew::virtual_dom::{AttributeOrProperty, VNode};
use yew::{AttrValue, Html};

/// The attribute name test ids are written to.
pub(crate) const ATTR: &str = match option_env!("SOCKEL_TEST_ID_ATTR") {
    Some(name) => name,
    None => "data-testid",
};

/// Whether `name` is usable as an HTML attribute name written to the DOM:
/// non-empty, ASCII, starting with a letter, `_` or `:`, then letters,
/// digits, `_`, `:`, `.` or `-`.
const fn valid(name: &str) -> bool {
    let bytes = name.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        let head = b.is_ascii_alphabetic() || b == b'_' || b == b':';
        let rest = b.is_ascii_digit() || b == b'.' || b == b'-';
        if !(head || (i > 0 && rest)) {
            return false;
        }
        i += 1;
    }
    true
}

const _: () = assert!(
    valid(ATTR),
    "SOCKEL_TEST_ID_ATTR is not a valid attribute name (letters, digits, `_`, `:`, `.`, `-`; not starting with a digit, `.` or `-`)"
);

/// `node` with the test id `id` on it, when it is one element and there is an
/// id. A node that is not one element is returned as it is.
pub(crate) fn with_testid(mut node: Html, id: Option<impl Into<AttrValue>>) -> Html {
    if let (Some(id), VNode::VTag(tag)) = (id, &mut node) {
        // The element was built by the caller just now, so it is not shared.
        if let Some(tag) = Rc::get_mut(tag) {
            // Yew drops a literal attribute (`role="a"`) of an element with
            // only literals when its attributes become a map, so they become
            // plain attributes first.
            for value in tag.attributes.get_mut_index_map().values_mut() {
                if let AttributeOrProperty::Static(literal) = *value {
                    *value = AttributeOrProperty::Attribute(AttrValue::Static(literal));
                }
            }
            tag.add_attribute(ATTR, id);
        } else {
            debug_assert!(false, "a test id on an element that is shared");
        }
    }
    node
}

#[cfg(test)]
mod tests {
    use super::*;
    use yew::prelude::*;

    #[test]
    fn valid_names() {
        assert!(valid("data-testid"));
        assert!(valid("data-test"));
        assert!(valid("x:y.z_1"));
        assert!(!valid(""));
        assert!(!valid("1a"));
        assert!(!valid("-a"));
        assert!(!valid("a b"));
        assert!(!valid("a=b"));
        assert!(!valid("a\"b"));
        assert!(!valid("é"));
    }

    #[derive(Properties, PartialEq)]
    struct Props {
        id: Option<AttrValue>,
    }

    #[function_component]
    fn Probe(props: &Props) -> Html {
        with_testid(html! { <div role="a" /> }, props.id.clone())
    }

    fn render(id: Option<&str>) -> String {
        futures::executor::block_on(
            yew::LocalServerRenderer::<Probe>::with_props(Props {
                id: id.map(AttrValue::from),
            })
            .hydratable(false)
            .render(),
        )
    }

    #[test]
    fn an_element_gets_the_attribute() {
        // `data-testid` unless the build sets SOCKEL_TEST_ID_ATTR.
        assert_eq!(
            render(Some("x")),
            format!("<div role=\"a\" {ATTR}=\"x\"></div>")
        );
    }

    #[test]
    fn default_renders_data_testid() {
        if option_env!("SOCKEL_TEST_ID_ATTR").is_none() {
            assert!(render(Some("x")).contains("data-testid=\"x\""));
        }
    }

    #[test]
    fn no_id_writes_no_attribute() {
        assert_eq!(render(None), "<div role=\"a\"></div>");
    }
}
