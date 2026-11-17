//! SelectionBar and ActionRow (`style/_rows.scss`).

use crate::class;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SelectionBarProps {
    /// How many are picked; none leaves the bar's room, empty.
    pub count: usize,
    /// The count as text ("3 selected").
    pub label: AttrValue,
    /// Its sides level with a row list's rows.
    #[prop_or_default]
    pub inset: bool,
    /// Stays at the bottom of its scroll container.
    #[prop_or_default]
    pub sticky: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// The actions on the picked rows.
    #[prop_or_default]
    pub children: Html,
}

/// A selection bar's classes: `idle` while nothing is picked.
pub fn selection_bar(idle: bool, inset: bool, sticky: bool) -> Classes {
    classes!(
        class::SELECTION_BAR,
        idle.then_some(class::IDLE),
        inset.then_some(class::INSET),
        sticky.then_some(class::STICKY)
    )
}

/// The band under a list: the picked count, then the actions on them. It is
/// always in the page's flow at one height, hidden while nothing is picked,
/// so picking never moves the list.
#[function_component]
pub fn SelectionBar(props: &SelectionBarProps) -> Html {
    html! {
        <div class={classes!(selection_bar(props.count == 0, props.inset, props.sticky), props.class.clone())}
            data-testid={props.testid.clone()}>
            <span class={class::SELECTION_BAR_COUNT} role="status">{ props.label.clone() }</span>
            { props.children.clone() }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ActionRowProps {
    /// Over the end of the row it stands in, invisible and inert until the
    /// row is hovered or has focus.
    #[prop_or_default]
    pub overlay: bool,
    /// One line, never shrinking.
    #[prop_or_default]
    pub nowrap: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    #[prop_or_default]
    pub children: Html,
}

/// An action row's classes.
pub fn action_row(overlay: bool, nowrap: bool) -> Classes {
    classes!(
        class::ACTION_ROW,
        overlay.then_some(class::OVERLAY),
        nowrap.then_some(class::NOWRAP)
    )
}

/// The actions that act on what stands above or around them, wrapping when
/// crowded. As an overlay it takes no room: it lies over its row's end.
#[function_component]
pub fn ActionRow(props: &ActionRowProps) -> Html {
    html! {
        <div class={classes!(action_row(props.overlay, props.nowrap), props.class.clone())}
            data-testid={props.testid.clone()}>
            { props.children.clone() }
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    /// The declarations of the first rule whose selector is `selector`.
    fn rule(selector: &str) -> &'static str {
        let css = crate::STYLESHEET;
        let from = css
            .find(&format!("{selector} {{"))
            .expect("the rule exists");
        let body = &css[from..];
        &body[..body.find('}').expect("the rule ends")]
    }

    #[test]
    fn selection_bar_modifiers() {
        assert_eq!(
            names(selection_bar(false, false, false)),
            [class::SELECTION_BAR]
        );
        assert_eq!(
            names(selection_bar(true, true, true)),
            [
                class::SELECTION_BAR,
                class::IDLE,
                class::INSET,
                class::STICKY
            ]
        );
    }

    #[test]
    fn action_row_modifiers() {
        assert_eq!(names(action_row(false, false)), [class::ACTION_ROW]);
        assert_eq!(
            names(action_row(true, true)),
            [class::ACTION_ROW, class::OVERLAY, class::NOWRAP]
        );
    }

    #[test]
    fn an_overlay_is_inert_until_its_row_is_hovered_or_focused() {
        let overlay = rule(".sockel-action-row.sockel-overlay");
        assert!(overlay.contains("opacity: var(--overlay-opacity, 0)"));
        assert!(overlay.contains("pointer-events: var(--overlay-events, none)"));
        assert!(overlay.contains("position: absolute"));

        // Focus reaches its controls (they stay in the tab order) and shows it.
        let focused = rule(".sockel-action-row.sockel-overlay:focus-within");
        assert!(focused.contains("opacity: 1"));
        assert!(focused.contains("pointer-events: auto"));

        let row = rule(".sockel-row-list-item:hover, .sockel-row-list-item:focus-within");
        assert!(row.contains("--overlay-opacity: 1"));
        assert!(row.contains("--overlay-events: auto"));
    }
}
