//! ChipToggle and ChipList (`style/_toggles.scss`).

use crate::class;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ChipToggleProps {
    #[prop_or_default]
    pub class: Classes,
    /// Whether the filter is on (`aria-pressed`): marked by its fill, never
    /// its weight.
    pub on: bool,
    /// How many items it filters to, in a field two figures wide.
    #[prop_or_default]
    pub count: Option<u32>,
    #[prop_or_default]
    pub disabled: bool,
    /// Called with the new state when clicked.
    pub on_toggle: Callback<bool>,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// The label, cut with an ellipsis when the chip is out of room.
    #[prop_or_default]
    pub children: Html,
}

/// A count's classes: `chip-count`, and `zero` for none.
pub fn chip_count(count: u32) -> Classes {
    classes!(class::CHIP_COUNT, (count == 0).then_some(class::ZERO))
}

/// A filter chip: a toggle button of one height, its label and optional
/// count, the same box on or off.
#[function_component]
pub fn ChipToggle(props: &ChipToggleProps) -> Html {
    let onclick = {
        let on_toggle = props.on_toggle.clone();
        let on = props.on;
        Callback::from(move |_: MouseEvent| on_toggle.emit(!on))
    };

    html! {
        <button type="button" class={classes!(class::CHIP, props.class.clone())}
            aria-pressed={props.on.to_string()}
            disabled={props.disabled}
            data-testid={props.testid.clone()}
            {onclick}>
            <span class={class::CHIP_LABEL}>{ props.children.clone() }</span>
            { for props.count.map(|count| html! { <span class={chip_count(count)}>{ count }</span> }) }
        </button>
    }
}

#[derive(Properties, PartialEq)]
pub struct ChipListProps {
    #[prop_or_default]
    pub class: Classes,
    /// What the chips filter (`aria-label`).
    pub label: AttrValue,
    #[prop_or_default]
    pub children: Html,
}

/// Chips that wrap, in a row at least one chip high, so an empty list
/// does not collapse and grow as chips arrive.
#[function_component]
pub fn ChipList(props: &ChipListProps) -> Html {
    html! {
        <div role="group" class={classes!(class::CHIP_LIST, props.class.clone())}
            aria-label={props.label.clone()}>
            { props.children.clone() }
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_count_of_none_is_zero() {
        let names = |c: Classes| c.into_iter().map(|c| c.to_string()).collect::<Vec<_>>();
        assert_eq!(names(chip_count(3)), [class::CHIP_COUNT]);
        assert_eq!(names(chip_count(0)), [class::CHIP_COUNT, class::ZERO]);
    }

    #[test]
    fn a_chip_reports_its_state_and_count() {
        let html = futures::executor::block_on(
            yew::LocalServerRenderer::<ChipToggle>::with_props(yew::props!(ChipToggleProps {
                on: true,
                count: Some(4),
                on_toggle: Callback::noop(),
                children: html! { "Mine" }
            }))
            .hydratable(false)
            .render(),
        );
        assert!(html.contains("aria-pressed=\"true\""), "{html}");
        assert!(html.contains(">4<"), "{html}");
    }
}
