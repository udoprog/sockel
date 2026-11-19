//! Chip, ChipToggle and ChipList (`style/_chip.scss`).

use crate::phrasebookcatalog01::{L10N, label};
use crate::{Glyph, Icon, class};
use phrasebook02::yew023::use_phrases_with;
use yew::prelude::*;

/// A [`Chip`]'s classes: `chip`, `static` for one that is neither a link
/// nor a button, `lead-media` and `lead-icon` for one led by an avatar or an
/// icon.
pub fn chip(still: bool, media: bool, icon: bool) -> Classes {
    classes!(
        class::CHIP,
        still.then_some(class::STATIC),
        media.then_some(class::LEAD_MEDIA),
        (icon && !media).then_some(class::LEAD_ICON)
    )
}

#[derive(Properties, PartialEq)]
pub struct ChipProps {
    #[prop_or_default]
    pub class: Classes,
    /// Makes the chip a link.
    #[prop_or_default]
    pub href: Option<AttrValue>,
    /// Makes the chip a `<button>`; with `href`, called when the link is
    /// clicked (an app's router).
    #[prop_or_default]
    pub on_click: Option<Callback<MouseEvent>>,
    /// Only for a button.
    #[prop_or_default]
    pub disabled: bool,
    /// Leads the chip in its round start (`lead-media`): an avatar or a
    /// portrait, `$chip-media-inset` from the edge.
    #[prop_or_default]
    pub media: Option<Html>,
    /// Leads the label (`lead-icon`): an icon in the muted ink,
    /// `$chip-icon-inset` from the edge when set.
    #[prop_or_default]
    pub icon: Option<Html>,
    /// After the label and never cut with it: a number or a badge.
    #[prop_or_default]
    pub after: Html,
    /// Gives a chip that is neither a link nor a button a remove cross,
    /// called when it is pressed.
    #[prop_or_default]
    pub on_remove: Option<Callback<MouseEvent>>,
    /// The cross's name and tooltip (`sockel-chip-remove-label`: "Remove").
    #[prop_or_default]
    pub remove_label: Option<AttrValue>,
    #[prop_or_default]
    pub remove_testid: Option<AttrValue>,
    /// A `data-tip`.
    #[prop_or_default]
    pub tip: Option<AttrValue>,
    /// Custom properties the app's class reads (a colour).
    #[prop_or_default]
    pub style: Option<AttrValue>,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// More `data-*` attributes (`("data-group", ..)`) the app's code reads.
    #[prop_or_default]
    pub data: Vec<(&'static str, AttrValue)>,
    /// The label, cut with an ellipsis when the chip is out of room.
    #[prop_or_default]
    pub children: Html,
}

/// A chip that is not a toggle: a link, a button or an item, led by an
/// avatar or an icon, an item removable by a cross. One height whatever it
/// holds, as a [`ChipToggle`]'s.
#[function_component]
pub fn Chip(props: &ChipProps) -> Html {
    let phrases = use_phrases_with(&L10N);
    let still = props.href.is_none() && props.on_click.is_none();
    let class = classes!(
        chip(still, props.media.is_some(), props.icon.is_some()),
        props.class.clone()
    );

    let remove = props.on_remove.as_ref().filter(|_| still).map(|on_remove| {
        let name = label(&phrases, &props.remove_label, "sockel-chip-remove-label");

        html! {
            <button type="button" class={class::CHIP_REMOVE} aria-label={name.clone()} data-tip={name}
                data-testid={props.remove_testid.clone()} onclick={on_remove.clone()}>
                <Icon glyph={Glyph::Close} />
            </button>
        }
    });

    let content = html! {
        <>
            { for props.media.clone().map(|media| html! { <span class={class::CHIP_MEDIA}>{ media }</span> }) }
            { for props.icon.clone().map(|icon| html! { <span class={class::CHIP_ICON} aria-hidden="true">{ icon }</span> }) }
            <span class={class::CHIP_LABEL}>{ props.children.clone() }</span>
            { props.after.clone() }
            { for remove }
        </>
    };

    let (testid, tip, style) = (props.testid.clone(), props.tip.clone(), props.style.clone());
    let mut node = match (&props.href, &props.on_click) {
        (Some(href), onclick) => html! {
            <a href={href.clone()} {class} {style} data-tip={tip} data-testid={testid} onclick={onclick.clone()}>{ content }</a>
        },
        (None, Some(onclick)) => html! {
            <button type="button" {class} {style} data-tip={tip} data-testid={testid}
                disabled={props.disabled} onclick={onclick.clone()}>{ content }</button>
        },
        (None, None) => html! {
            <span {class} {style} data-tip={tip} data-testid={testid}>{ content }</span>
        },
    };

    if let Html::VTag(tag) = &mut node {
        for (name, value) in &props.data {
            std::rc::Rc::make_mut(tag).add_attribute(name, value.clone());
        }
    }

    node
}

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

    fn render(props: ChipProps) -> String {
        futures::executor::block_on(
            yew::LocalServerRenderer::<Chip>::with_props(props)
                .hydratable(false)
                .render(),
        )
    }

    #[test]
    fn an_item_chip_is_static_and_a_link_is_not() {
        let names = |c: Classes| c.into_iter().map(|c| c.to_string()).collect::<Vec<_>>();
        assert_eq!(
            names(chip(true, false, false)),
            [class::CHIP, class::STATIC]
        );
        assert_eq!(names(chip(false, false, false)), [class::CHIP]);
        assert_eq!(
            names(chip(false, true, true)),
            [class::CHIP, class::LEAD_MEDIA]
        );

        let link = render(yew::props!(ChipProps {
            href: Some(AttrValue::from("/x")),
            media: Some(html! { <img alt="" /> }),
            children: html! { "Ada" }
        }));
        assert!(link.starts_with("<a href=\"/x\""), "{link}");
        assert!(!link.contains(class::STATIC), "{link}");
        assert!(link.contains(class::LEAD_MEDIA), "{link}");
        assert!(link.contains(class::CHIP_MEDIA), "{link}");
    }

    #[test]
    fn an_item_chip_has_a_named_cross() {
        let item = render(yew::props!(ChipProps {
            on_remove: Some(Callback::noop()),
            remove_label: Some(AttrValue::from("Remove Ada")),
            remove_testid: Some(AttrValue::from("x")),
            data: vec![("data-group", AttrValue::from("g"))],
            children: html! { "Ada" }
        }));
        assert!(item.starts_with("<span"), "{item}");
        assert!(item.contains("data-group=\"g\""), "{item}");
        assert!(item.contains(class::CHIP_REMOVE), "{item}");
        assert!(item.contains("aria-label=\"Remove Ada\""), "{item}");

        let link = render(yew::props!(ChipProps {
            href: Some(AttrValue::from("/x")),
            on_remove: Some(Callback::noop()),
            children: html! { "Ada" }
        }));
        assert!(!link.contains(class::CHIP_REMOVE), "{link}");
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
