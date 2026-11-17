//! Notice (`style/_notices.scss`).

use crate::{Button, ButtonSize, ButtonVariant, Glyph, Icon, class};
use yew::prelude::*;

/// A notice's tone: `Info` unless set.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NoticeTone {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Properties, PartialEq)]
pub struct NoticeProps {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub tone: NoticeTone,
    /// The icon, which the app supplies, before the text.
    #[prop_or_default]
    pub icon: Option<Html>,
    /// Shows a close button, which calls this. The notice does not remove
    /// itself: the app stops rendering it.
    #[prop_or_default]
    pub onclose: Option<Callback<MouseEvent>>,
    /// The close button's accessible name.
    #[prop_or(AttrValue::Static("Dismiss"))]
    pub close_label: AttrValue,
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
pub fn notice(tone: NoticeTone) -> Classes {
    let tone = match tone {
        NoticeTone::Info => None,
        NoticeTone::Success => Some(class::SUCCESS),
        NoticeTone::Warning => Some(class::WARNING),
        NoticeTone::Error => Some(class::ERROR),
    };
    classes!(class::NOTICE, tone)
}

/// An inline message: an optional icon, the text, an optional action and
/// close button. One line high at least, so closing the button's room is
/// not a change of height; closing the notice itself collapses its space.
/// A warning or an error is announced as an alert, the others as a status.
#[function_component]
pub fn Notice(props: &NoticeProps) -> Html {
    let role = match props.tone {
        NoticeTone::Warning | NoticeTone::Error => "alert",
        NoticeTone::Info | NoticeTone::Success => "status",
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
                        label={props.close_label.clone()} onclick={onclose.clone()} />
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
        assert_eq!(names(notice(NoticeTone::Info)), [class::NOTICE]);
        assert_eq!(
            names(notice(NoticeTone::Success)),
            [class::NOTICE, class::SUCCESS]
        );
        assert_eq!(
            names(notice(NoticeTone::Warning)),
            [class::NOTICE, class::WARNING]
        );
        assert_eq!(
            names(notice(NoticeTone::Error)),
            [class::NOTICE, class::ERROR]
        );
    }
}
