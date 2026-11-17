//! Button (`style/_buttons.scss`).

use crate::{Glyph, Icon, Spinner, class};
use yew::prelude::*;

/// A button's colour: `Secondary` (the plain button) unless set.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    #[default]
    Secondary,
    /// The one thing to do here: filled with `--accent`.
    Primary,
    /// No fill, its border kept.
    Ghost,
    /// No fill and no border until hovered: a toolbar's or a row's tools.
    Quiet,
    Danger,
    Success,
    /// No fill, edge, padding or height of its own: its content is its face,
    /// such as a swatch.
    Bare,
}

/// A button's height: `--control-height`, `--control-height-small` or
/// `--touch-height`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonSize {
    #[default]
    Regular,
    Small,
    Touch,
}

#[derive(Properties, PartialEq)]
pub struct ButtonProps {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub variant: ButtonVariant,
    #[prop_or_default]
    pub size: ButtonSize,
    /// The icon, which the app supplies; with no label, the button is the
    /// icon alone and square (`.sockel-icon`).
    #[prop_or_default]
    pub icon: Option<Html>,
    /// The icon comes after the label.
    #[prop_or_default]
    pub trailing: bool,
    /// Ends in a chevron after the label, as a menu's opener does, beside
    /// a leading icon too.
    #[prop_or_default]
    pub chevron: bool,
    /// The icon alone below the phone breakpoint, the label kept for a
    /// screen reader (`.sockel-fold`).
    #[prop_or_default]
    pub fold: bool,
    /// The icon alone from the phone breakpoint up, the label kept for a
    /// screen reader (`.sockel-fold-wide`): `fold` the other way round.
    #[prop_or_default]
    pub fold_wide: bool,
    /// A toggle and whether it is on (`aria-pressed`).
    #[prop_or_default]
    pub pressed: Option<bool>,
    /// Working on what it started: a spinner over the label, which keeps
    /// its room (`aria-busy`). A busy button ignores clicks.
    #[prop_or_default]
    pub busy: bool,
    #[prop_or_default]
    pub disabled: bool,
    /// Cannot act now but stays focusable and hoverable, so it can say why
    /// (`aria-disabled`, drawn as disabled): a click does nothing.
    #[prop_or_default]
    pub unavailable: bool,
    /// The button's element, for what anchors beside it.
    #[prop_or_default]
    pub node_ref: NodeRef,
    /// Whether what it opens is open (`aria-expanded`).
    #[prop_or_default]
    pub expanded: Option<bool>,
    /// What it opens (`aria-haspopup`: `"menu"`, `"listbox"`, `"dialog"`).
    #[prop_or_default]
    pub haspopup: Option<AttrValue>,
    /// A link styled as a button (`<a>`); disabled, it loses its `href`.
    #[prop_or_default]
    pub href: Option<AttrValue>,
    /// A submit button of its form (the default is a plain button).
    #[prop_or_default]
    pub submit: bool,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
    #[prop_or_default]
    pub onkeydown: Callback<KeyboardEvent>,
    #[prop_or_default]
    pub title: Option<AttrValue>,
    /// Its tooltip, shown by [`DataTips`](crate::DataTips) (`data-tip`).
    #[prop_or_default]
    pub tip: Option<AttrValue>,
    /// The accessible name, when the label is not the full name or there
    /// is none (`aria-label`).
    #[prop_or_default]
    pub label: Option<AttrValue>,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// A role other than a button's, such as `menuitemradio` or `switch`.
    #[prop_or_default]
    pub role: Option<AttrValue>,
    /// Whether a `menuitemradio`, `menuitemcheckbox` or `switch` is on
    /// (`aria-checked`).
    #[prop_or_default]
    pub checked: Option<bool>,
    /// The current item of a set, as `page` or `step` (`aria-current`).
    #[prop_or_default]
    pub current: Option<AttrValue>,
    /// The label.
    #[prop_or_default]
    pub children: Html,
}

/// A button's classes: `button`, its variant and size, `icon` for an icon
/// alone and `fold` for one that folds to its icon on a phone.
pub fn button(variant: ButtonVariant, size: ButtonSize, icon_only: bool, fold: bool) -> Classes {
    let variant = match variant {
        ButtonVariant::Secondary => None,
        ButtonVariant::Primary => Some(class::PRIMARY),
        ButtonVariant::Ghost => Some(class::GHOST),
        ButtonVariant::Quiet => Some(class::QUIET),
        ButtonVariant::Danger => Some(class::DANGER),
        ButtonVariant::Success => Some(class::SUCCESS),
        ButtonVariant::Bare => Some(class::BARE),
    };
    let size = match size {
        ButtonSize::Regular => None,
        ButtonSize::Small => Some(class::SMALL),
        ButtonSize::Touch => Some(class::TOUCH),
    };
    classes!(
        class::BUTTON,
        variant,
        size,
        icon_only.then_some(class::ICON),
        fold.then_some(class::FOLD)
    )
}

pub(crate) fn is_empty(html: &Html) -> bool {
    match html {
        Html::VList(list) => list.iter().all(is_empty),
        Html::VText(text) => text.text.is_empty(),
        _ => false,
    }
}

/// A button, or a link that looks like one: an optional icon and a label,
/// one height per size, and the same box whether it is pressed, busy,
/// hovered or focused.
#[function_component]
pub fn Button(props: &ButtonProps) -> Html {
    let has_label = !is_empty(&props.children);
    let icon_only = props.icon.is_some() && !has_label;
    let fold = props.fold && props.icon.is_some() && has_label;
    let fold_wide = props.fold_wide && props.icon.is_some() && has_label;
    let class = classes!(
        button(props.variant, props.size, icon_only, fold),
        fold_wide.then_some(class::FOLD_WIDE),
        props.class.clone()
    );

    let icon = props
        .icon
        .clone()
        .map(|icon| html! { <span class={class::BUTTON_ICON} aria-hidden="true">{ icon }</span> });
    let label = has_label.then(|| {
        html! {
            <span class={classes!(
                class::BUTTON_LABEL,
                fold.then_some(class::FOLD),
                fold_wide.then_some(class::FOLD_WIDE)
            )}>
                { props.children.clone() }
            </span>
        }
    });
    let chevron = props.chevron.then(|| {
        html! {
            <span class={class::BUTTON_ICON} aria-hidden="true"><Icon glyph={Glyph::ChevronDown} /></span>
        }
    });
    let content = html! {
        <span class={classes!(class::BUTTON_CONTENT, props.busy.then_some(class::BUSY))}>
            if props.trailing { { for label } { for icon } } else { { for icon } { for label } }
            { for chevron }
        </span>
    };
    let spinner = props.busy.then(|| {
        html! { <span class={class::BUTTON_SPINNER}><Spinner /></span> }
    });

    let onclick = {
        let onclick = props.onclick.clone();
        let blocked = props.busy || props.disabled || props.unavailable;
        Callback::from(move |e: MouseEvent| {
            if blocked {
                e.prevent_default();
            } else {
                onclick.emit(e);
            }
        })
    };
    let busy = props.busy.then_some("true");
    let unavailable = props.unavailable.then_some("true");
    let pressed = props.pressed.map(|on| on.to_string());
    let expanded = props.expanded.map(|on| on.to_string());
    let checked = props.checked.map(|on| on.to_string());

    match &props.href {
        Some(href) => html! {
            <a {class} ref={props.node_ref.clone()}
                aria-expanded={expanded.clone()}
                aria-haspopup={props.haspopup.clone()}
                href={(!props.disabled).then(|| href.clone())}
                aria-disabled={(props.disabled || props.unavailable).then_some("true")}
                aria-busy={busy}
                aria-pressed={pressed}
                title={props.title.clone()}
                data-tip={props.tip.clone()}
                aria-label={props.label.clone()}
                data-testid={props.testid.clone()}
                role={props.role.clone()}
                aria-checked={checked.clone()}
                aria-current={props.current.clone()}
                {onclick}
                onkeydown={props.onkeydown.clone()}
            >
                { content }{ for spinner }
            </a>
        },
        None => html! {
            <button ref={props.node_ref.clone()}
                aria-expanded={expanded.clone()}
                aria-haspopup={props.haspopup.clone()}
                type={if props.submit { "submit" } else { "button" }}
                {class}
                disabled={props.disabled}
                aria-disabled={unavailable}
                aria-busy={busy}
                aria-pressed={pressed}
                title={props.title.clone()}
                data-tip={props.tip.clone()}
                aria-label={props.label.clone()}
                data-testid={props.testid.clone()}
                role={props.role.clone()}
                aria-checked={checked.clone()}
                aria-current={props.current.clone()}
                {onclick}
                onkeydown={props.onkeydown.clone()}
            >
                { content }{ for spinner }
            </button>
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    fn render(props: ButtonProps) -> String {
        futures::executor::block_on(
            yew::LocalServerRenderer::<Button>::with_props(props)
                .hydratable(false)
                .render(),
        )
    }

    fn props() -> ButtonProps {
        ButtonProps {
            class: Classes::new(),
            variant: ButtonVariant::default(),
            size: ButtonSize::default(),
            icon: None,
            trailing: false,
            chevron: false,
            fold: false,
            fold_wide: false,
            pressed: None,
            busy: false,
            disabled: false,
            unavailable: false,
            node_ref: NodeRef::default(),
            expanded: None,
            haspopup: None,
            href: None,
            submit: false,
            onclick: Callback::noop(),
            onkeydown: Callback::noop(),
            title: None,
            tip: None,
            label: None,
            testid: None,
            role: None,
            checked: None,
            current: None,
            children: html! { "Save" },
        }
    }

    #[test]
    fn each_variant_is_its_modifier() {
        let of = |v| names(button(v, ButtonSize::Regular, false, false));
        assert_eq!(of(ButtonVariant::Secondary), [class::BUTTON]);
        assert_eq!(of(ButtonVariant::Primary), [class::BUTTON, class::PRIMARY]);
        assert_eq!(of(ButtonVariant::Ghost), [class::BUTTON, class::GHOST]);
        assert_eq!(of(ButtonVariant::Quiet), [class::BUTTON, class::QUIET]);
        assert_eq!(of(ButtonVariant::Danger), [class::BUTTON, class::DANGER]);
        assert_eq!(of(ButtonVariant::Success), [class::BUTTON, class::SUCCESS]);
    }

    #[test]
    fn each_size_is_its_modifier() {
        let of = |s| names(button(ButtonVariant::Secondary, s, false, false));
        assert_eq!(of(ButtonSize::Regular), [class::BUTTON]);
        assert_eq!(of(ButtonSize::Small), [class::BUTTON, class::SMALL]);
        assert_eq!(of(ButtonSize::Touch), [class::BUTTON, class::TOUCH]);
    }

    #[test]
    fn icon_only_and_fold_are_modifiers() {
        let v = ButtonVariant::Secondary;
        let s = ButtonSize::Regular;
        assert_eq!(
            names(button(v, s, true, false)),
            [class::BUTTON, class::ICON]
        );
        assert_eq!(
            names(button(v, s, false, true)),
            [class::BUTTON, class::FOLD]
        );
    }

    #[test]
    fn a_chevron_ends_it_after_the_icon_and_label() {
        let html = render(ButtonProps {
            icon: Some(html! { <svg /> }),
            chevron: true,
            children: html! { "Presets" },
            ..props()
        });
        let icon = format!("class=\"{}\"", class::BUTTON_ICON);
        assert_eq!(html.matches(&icon).count(), 2, "{html}");
        assert!(html.find("Presets") < html.rfind(&icon), "{html}");
    }

    #[test]
    fn an_icon_without_a_label_is_icon_only() {
        let html = render(ButtonProps {
            icon: Some(html! { <svg /> }),
            children: Html::default(),
            ..props()
        });
        assert!(
            html.contains(&format!("{} {}", class::BUTTON, class::ICON)),
            "{html}"
        );
        assert!(!html.contains(class::BUTTON_LABEL), "{html}");
    }

    #[test]
    fn an_empty_text_is_no_label() {
        let html = render(ButtonProps {
            icon: Some(html! { <svg /> }),
            children: html! { <>{ String::new() }if false { { "x" } }</> },
            ..props()
        });
        assert!(html.contains(class::ICON), "{html}");
        assert!(!html.contains(class::BUTTON_LABEL), "{html}");
    }

    #[test]
    fn fold_wide_marks_the_button_and_its_label() {
        let html = render(ButtonProps {
            icon: Some(html! { <svg /> }),
            fold_wide: true,
            ..props()
        });
        assert!(
            html.contains(&format!("{} {}", class::BUTTON, class::FOLD_WIDE)),
            "{html}"
        );
        assert!(
            html.contains(&format!("{} {}\"", class::BUTTON_LABEL, class::FOLD_WIDE)),
            "{html}"
        );
        let html = render(ButtonProps {
            fold_wide: true,
            ..props()
        });
        assert!(!html.contains(class::FOLD_WIDE), "{html}");
    }

    #[test]
    fn a_role_says_whether_it_is_checked_and_current() {
        let html = render(ButtonProps {
            role: Some(AttrValue::from("menuitemradio")),
            checked: Some(true),
            current: Some(AttrValue::from("page")),
            ..props()
        });
        for attr in [
            "role=\"menuitemradio\"",
            "aria-checked=\"true\"",
            "aria-current=\"page\"",
        ] {
            assert!(html.contains(attr), "{attr}: {html}");
        }
    }

    #[test]
    fn fold_marks_the_button_and_its_label() {
        let html = render(ButtonProps {
            icon: Some(html! { <svg /> }),
            fold: true,
            ..props()
        });
        assert!(
            html.contains(&format!("{} {}\"", class::BUTTON, class::FOLD)),
            "{html}"
        );
        assert!(
            html.contains(&format!("{} {}\"", class::BUTTON_LABEL, class::FOLD)),
            "{html}"
        );
        // Without an icon there is nothing to fold to.
        let html = render(ButtonProps {
            fold: true,
            ..props()
        });
        assert!(!html.contains(class::FOLD), "{html}");
    }

    #[test]
    fn busy_sets_aria_busy_and_keeps_the_label() {
        let html = render(ButtonProps {
            busy: true,
            ..props()
        });
        assert!(html.contains("aria-busy=\"true\""), "{html}");
        assert!(html.contains(">Save</span>"), "{html}");
        assert!(html.contains(class::BUTTON_SPINNER), "{html}");
        assert!(
            html.contains(&format!("{} {}", class::BUTTON_CONTENT, class::BUSY)),
            "{html}"
        );

        let html = render(props());
        assert!(!html.contains("aria-busy"), "{html}");
        assert!(!html.contains(class::BUTTON_SPINNER), "{html}");
    }

    #[test]
    fn pressed_sets_aria_pressed() {
        let html = render(ButtonProps {
            pressed: Some(false),
            ..props()
        });
        assert!(html.contains("aria-pressed=\"false\""), "{html}");
        assert!(!render(props()).contains("aria-pressed"));
    }

    #[test]
    fn a_disabled_link_is_aria_disabled_without_its_href() {
        let link = || ButtonProps {
            href: Some("/next".into()),
            ..props()
        };
        let html = render(link());
        assert!(html.starts_with("<a"), "{html}");
        assert!(html.contains("href=\"/next\""), "{html}");
        assert!(!html.contains("aria-disabled"), "{html}");

        let html = render(ButtonProps {
            disabled: true,
            ..link()
        });
        assert!(html.contains("aria-disabled=\"true\""), "{html}");
        assert!(!html.contains("href"), "{html}");
    }

    #[test]
    fn an_unavailable_button_is_aria_disabled_and_stays_enabled() {
        let html = render(ButtonProps {
            unavailable: true,
            ..props()
        });
        assert!(html.contains("aria-disabled=\"true\""), "{html}");
        assert!(!html.contains(" disabled"), "{html}");
    }

    #[test]
    fn an_opener_says_what_it_opens_and_whether_it_is_open() {
        let html = render(ButtonProps {
            expanded: Some(true),
            haspopup: Some("menu".into()),
            ..props()
        });
        assert!(html.contains("aria-expanded=\"true\""), "{html}");
        assert!(html.contains("aria-haspopup=\"menu\""), "{html}");
        assert!(!render(props()).contains("aria-expanded"));
    }

    #[test]
    fn a_tip_is_its_data_tip() {
        let html = render(ButtonProps {
            tip: Some("Saves the draft".into()),
            ..props()
        });
        assert!(html.contains("data-tip=\"Saves the draft\""), "{html}");
        assert!(!render(props()).contains("data-tip"));
    }

    #[test]
    fn a_disabled_button_is_disabled() {
        let html = render(ButtonProps {
            disabled: true,
            ..props()
        });
        assert!(html.starts_with("<button"), "{html}");
        assert!(html.contains("disabled"), "{html}");
    }
}
