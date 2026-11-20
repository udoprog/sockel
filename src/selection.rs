//! SelectionBar and ActionRow (`style/_rows.scss`).

use crate::class;
use crate::testid::with_testid;
use yew::prelude::*;

#[derive(Properties, Default, PartialEq)]
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
    /// The count's test id, on its span.
    #[prop_or_default]
    pub count_testid: Option<AttrValue>,
    /// Whether nothing is picked, when `count` cannot say (a count shown as
    /// a phrase); `None` takes it from `count == 0`.
    #[prop_or_default]
    pub idle: Option<bool>,
    #[prop_or_default]
    pub onkeydown: Callback<KeyboardEvent>,
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
    let idle = props.idle.unwrap_or(props.count == 0);
    let count = with_testid(
        html! {
            <span class={class::SELECTION_BAR_COUNT} role="status">{ props.label.clone() }</span>
        },
        props.count_testid.clone(),
    );
    with_testid(
        html! {
            <div class={classes!(selection_bar(idle, props.inset, props.sticky), props.class.clone())}
                onkeydown={props.onkeydown.clone()}>
                { count }
                { props.children.clone() }
            </div>
        },
        props.testid.clone(),
    )
}

#[derive(Properties, Default, PartialEq)]
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
    with_testid(
        html! {
            <div class={classes!(action_row(props.overlay, props.nowrap), props.class.clone())}>
                { props.children.clone() }
            </div>
        },
        props.testid.clone(),
    )
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

    fn render(props: SelectionBarProps) -> String {
        futures::executor::block_on(
            yew::LocalServerRenderer::<SelectionBar>::with_props(props)
                .hydratable(false)
                .render(),
        )
    }

    #[test]
    fn idle_comes_from_the_count_unless_given() {
        let idle = |count, idle| {
            render(SelectionBarProps {
                count,
                idle,
                label: "Some picked".into(),
                ..Default::default()
            })
            .contains(class::IDLE)
        };
        assert!(idle(0, None));
        assert!(!idle(2, None));
        assert!(!idle(0, Some(false)));
        assert!(idle(2, Some(true)));
    }

    #[test]
    fn the_count_takes_a_test_id() {
        let attr = crate::testid::ATTR;
        let html = render(SelectionBarProps {
            count: 1,
            label: "1 selected".into(),
            count_testid: Some("picked".into()),
            ..Default::default()
        });
        assert!(
            html.contains(&format!("{attr}=\"picked\">1 selected</span>")),
            "{html}"
        );
        let html = render(SelectionBarProps {
            count: 1,
            label: "1 selected".into(),
            ..Default::default()
        });
        assert!(!html.contains(attr), "{html}");
        // Unchanged from before the new props.
        assert_eq!(
            html,
            format!(
                "<div class=\"{}\"><span role=\"status\" class=\"{}\">1 selected</span></div>",
                class::SELECTION_BAR,
                class::SELECTION_BAR_COUNT
            )
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
