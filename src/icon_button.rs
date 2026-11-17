//! IconButton (`style/_buttons.scss`).

use crate::class;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct IconButtonProps {
    /// The button's other classes (`primary`, `ghost`, `danger`, `icon`
    /// for an icon alone, ...).
    #[prop_or_default]
    pub class: Classes,
    /// The icon comes after the label (`.trailing`).
    #[prop_or_default]
    pub trailing: bool,
    /// A submit button of its form (the default is a plain button).
    #[prop_or_default]
    pub submit: bool,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// Opens a dialog: whether it is open now (`aria-haspopup`, `aria-expanded`).
    #[prop_or_default]
    pub dialog: Option<bool>,
    /// Has a popup dialog it does not track (`aria-haspopup` alone).
    #[prop_or_default]
    pub haspopup: bool,
    /// A toggle: whether it is on now (`aria-pressed`).
    #[prop_or_default]
    pub pressed: Option<bool>,
    /// Discloses what it controls, not a dialog: whether that is open
    /// (`aria-expanded`).
    #[prop_or_default]
    pub expanded: Option<bool>,
    /// The id of what it discloses (`aria-controls`).
    #[prop_or_default]
    pub controls: Option<AttrValue>,
    /// The current item of a set, as `step` or `page` (`aria-current`).
    #[prop_or_default]
    pub current: Option<AttrValue>,
    /// The `<button>` element, to measure or focus it.
    #[prop_or_default]
    pub node_ref: NodeRef,
    #[prop_or_default]
    pub title: Option<AttrValue>,
    /// The accessible name, when the visible text is not the button's full
    /// name (`aria-label`).
    #[prop_or_default]
    pub label: Option<AttrValue>,
    /// The icon and the label.
    #[prop_or_default]
    pub children: Html,
}

/// The button's classes: `iconed` always, `trailing` for an icon after the label.
pub fn icon_button(classes: Classes, trailing: bool) -> Classes {
    classes!(classes, class::ICONED, trailing.then_some(class::TRAILING))
}

/// A button that leads with (or ends in) an icon, which the app supplies as
/// a child.
#[function_component]
pub fn IconButton(props: &IconButtonProps) -> Html {
    let class = icon_button(props.class.clone(), props.trailing);
    html! {
        <button
            type={if props.submit { "submit" } else { "button" }}
            {class}
            disabled={props.disabled}
            onclick={props.onclick.clone()}
            ref={props.node_ref.clone()}
            aria-haspopup={(props.haspopup || props.dialog.is_some()).then_some("dialog")}
            aria-expanded={props.dialog.or(props.expanded).map(|open| open.to_string())}
            aria-pressed={props.pressed.map(|on| on.to_string())}
            aria-controls={props.controls.clone()}
            aria-current={props.current.clone()}
            title={props.title.clone()}
            aria-label={props.label.clone()}
            data-testid={props.testid.clone()}
        >
            { props.children.clone() }
        </button>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn an_icon_button_is_iconed_and_trails_on_request() {
        assert_eq!(
            names(icon_button(classes!(class::ICON), false)),
            [class::ICON, class::ICONED]
        );
        assert_eq!(
            names(icon_button(Classes::new(), true)),
            [class::ICONED, class::TRAILING]
        );
    }

    #[test]
    fn an_icon_button_carries_its_state() {
        let html = futures::executor::block_on(
            yew::LocalServerRenderer::<IconButton>::with_props(yew::props!(IconButtonProps {
                pressed: Some(true),
                expanded: Some(false),
                controls: Some(AttrValue::from("panel")),
                current: Some(AttrValue::from("step")),
                haspopup: true,
                children: html! { "Go" }
            }))
            .hydratable(false)
            .render(),
        );
        for attr in [
            "aria-pressed=\"true\"",
            "aria-expanded=\"false\"",
            "aria-controls=\"panel\"",
            "aria-current=\"step\"",
            "aria-haspopup=\"dialog\"",
        ] {
            assert!(html.contains(attr), "{attr}: {html}");
        }
    }
}
