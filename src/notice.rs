//! Notice (`style/_notices.scss`).

use crate::testid::with_testid;
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
    /// The element's id, for `aria-describedby` and the like.
    #[prop_or_default]
    pub id: Option<AttrValue>,
    /// Announce it as an alert whatever its tone (a failure drawn in the
    /// accent); a warning or an error always is.
    #[prop_or_default]
    pub alert: bool,
    /// Announce it as a status though its tone is a warning or an error (a
    /// caution that does not stop anything). Over `alert`.
    #[prop_or_default]
    pub status: bool,
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
/// A warning or an error (or any with `alert`) is announced as an alert, the others as a status.
#[function_component]
pub fn Notice(props: &NoticeProps) -> Html {
    let phrases = use_phrases_with(&L10N);
    let role = match props.tone {
        _ if props.status => "status",
        Tone::Warning | Tone::Danger => "alert",
        _ if props.alert => "alert",
        _ => "status",
    };
    let close_label = label(&phrases, &props.close_label, "sockel-notice-dismiss");

    with_testid(
        html! {
            <div class={classes!(notice(props.tone), props.class.clone())} {role}
                id={props.id.clone()}>
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
                            label={close_label.clone()} tip={close_label} onclick={onclose.clone()} />
                    </span>
                }
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

    fn render(props: NoticeProps) -> String {
        futures::executor::block_on(
            yew::LocalServerRenderer::<Notice>::with_props(props)
                .hydratable(false)
                .render(),
        )
    }

    fn props() -> NoticeProps {
        NoticeProps {
            class: Classes::new(),
            tone: Tone::Accent,
            icon: None,
            onclose: None,
            close_label: None,
            action: None,
            testid: None,
            id: None,
            alert: false,
            status: false,
            children: html! { "Saved" },
        }
    }

    #[test]
    fn alert_and_id_are_opt_in() {
        let html = render(props());
        assert!(html.contains("role=\"status\""), "{html}");
        assert!(!html.contains("id="), "{html}");
        let html = render(NoticeProps {
            alert: true,
            id: Some("why".into()),
            ..props()
        });
        assert!(html.contains("role=\"alert\""), "{html}");
        assert!(html.contains("id=\"why\""), "{html}");
        let html = render(NoticeProps {
            tone: Tone::Warning,
            status: true,
            ..props()
        });
        assert!(html.contains("role=\"status\""), "{html}");
    }
}
