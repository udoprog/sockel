//! Notice (`style/_notices.scss`).

use phrasebook02::yew023::use_phrases_with;
use yew::prelude::*;

use crate::phrasebookcatalog01::{L10N, label};
use crate::{Button, ButtonSize, ButtonVariant, Glyph, Icon, Tone, class};

#[derive(Properties, PartialEq)]
pub struct NoticeProps {
    #[prop_or_default]
    pub class: Classes,
    /// The accent unless set.
    #[prop_or(Tone::Accent)]
    pub tone: Tone,
    /// The icon, which the app supplies, before the text.
    #[prop_or_default]
    pub icon: Option<Html>,
    /// Shows a close button, which calls this. The notice does not remove
    /// itself: the app stops rendering it.
    #[prop_or_default]
    pub onclose: Option<Callback<MouseEvent>>,
    /// The close button's accessible name, else the catalog's.
    #[prop_or_default]
    pub close_label: Option<AttrValue>,
    /// An action (a link or a button) after the text.
    #[prop_or_default]
    pub action: Option<Html>,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// The text.
    #[prop_or_default]
    pub children: Html,
}

/// A notice's classes: `notice` and its tone.
pub fn notice(tone: Tone) -> Classes {
    classes!(class::NOTICE, tone.class())
}

/// An inline message: an optional icon, the text, an optional action and
/// close button. One line high at least, so closing the button's room is
/// not a change of height; closing the notice itself collapses its space.
/// A warning or an error is announced as an alert, the others as a status.
#[function_component]
pub fn Notice(props: &NoticeProps) -> Html {
    let phrases = use_phrases_with(&L10N);
    let role = match props.tone {
        Tone::Warning | Tone::Danger => "alert",
        _ => "status",
    };

    html! {
        <div class={classes!(notice(props.tone), props.class.clone())} {role}
            data-testid={props.testid.clone()}>
            if let Some(icon) = &props.icon {
                <span class={class::NOTICE_ICON}>{ icon.clone() }</span>
            }
            <div class={class::NOTICE_BODY}>{ props.children.clone() }</div>
            if let Some(action) = &props.action {
                <span class={class::NOTICE_ACTION}>{ action.clone() }</span>
            }
            if let Some(onclose) = &props.onclose {
                <span class={class::NOTICE_CLOSE}>
                    <Button variant={ButtonVariant::Quiet} size={ButtonSize::Small}
                        icon={html! { <Icon glyph={Glyph::Close} /> }}
                        label={label(&phrases, &props.close_label, "sockel-notice-dismiss")} onclick={onclose.clone()} />
                </span>
            }
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn a_tone_is_a_modifier() {
        assert_eq!(names(notice(Tone::Neutral)), [class::NOTICE]);
        assert_eq!(names(notice(Tone::Accent)), [class::NOTICE, class::ACCENT]);
        assert_eq!(
            names(notice(Tone::Success)),
            [class::NOTICE, class::SUCCESS]
        );
        assert_eq!(
            names(notice(Tone::Warning)),
            [class::NOTICE, class::WARNING]
        );
        assert_eq!(names(notice(Tone::Danger)), [class::NOTICE, class::DANGER]);
    }
}
