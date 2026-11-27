//! PageHead and SectionHead (`style/_heads.scss`).

use crate::class;
use crate::testid::with_testid;
use yew::prelude::*;

#[derive(Properties, Default, PartialEq, Clone)]
pub struct HeadProps {
    /// The title's content: text, or formatted names and spans.
    pub title: Html,
    /// An icon before the title's text, inside the heading (a
    /// `sockel-head-icon` span, in the accent).
    #[prop_or_default]
    pub icon: Option<Html>,
    /// One muted line under the head's row (`sockel-head-subtitle`).
    #[prop_or_default]
    pub subtitle: Option<Html>,
    /// The pages above this one, outermost first: each a link, stacked in
    /// a list above the title's row with a chevron after it.
    #[prop_or_default]
    pub crumbs: Vec<Html>,
    /// The crumbs' accessible name.
    #[prop_or_default]
    pub crumbs_label: Option<AttrValue>,
    /// What explains the page or section (a help tip), after the heading
    /// as its sibling, so the heading's text and test id exclude it.
    #[prop_or_default]
    pub help: Option<Html>,
    /// The title's heading level, instead of the head's own (`h1` for a
    /// page, `h2` for a section).
    #[prop_or_default]
    pub level: Option<HeadLevel>,
    /// What follows the title (a count, a badge), after its help.
    #[prop_or_default]
    pub suffix: Option<Html>,
    /// The help's test id; the help then sits in an element carrying it.
    #[prop_or_default]
    pub help_testid: Option<AttrValue>,
    /// The title's own classes.
    #[prop_or_default]
    pub title_class: Classes,
    /// The title's test id, as `testid`.
    #[prop_or_default]
    pub title_testid: Option<AttrValue>,
    /// The head's test id.
    #[prop_or_default]
    pub head_testid: Option<AttrValue>,
    /// The title's test id.
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    #[prop_or_default]
    pub class: Classes,
    /// A back link (`Link` of classes `crumb-back page-head-back`, its words in a
    /// `label-text`) that turns the head into a crumb: the link, a `/`,
    /// then the title.
    #[prop_or_default]
    pub back: Option<Html>,
    /// The crumb's accessible name.
    #[prop_or_default]
    pub back_label: Option<AttrValue>,
    /// What stands between the title and the actions, such as keywords (an element of class `page-head-meta`).
    #[prop_or_default]
    pub meta: Option<Html>,
    /// The actions, at the end: full-size buttons, the primary last. A
    /// menu among them takes a full-size toggle (`toggle_class`
    /// `icon-only`), so it stands as high as the buttons beside it.
    #[prop_or_default]
    pub children: Html,
}

/// A title's heading level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeadLevel {
    H1,
    H2,
    H3,
    H4,
}

impl HeadLevel {
    pub(crate) fn tag(self) -> &'static str {
        match self {
            HeadLevel::H1 => "h1",
            HeadLevel::H2 => "h2",
            HeadLevel::H3 => "h3",
            HeadLevel::H4 => "h4",
        }
    }
}

/// Which head it is: its classes and its title's element.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Head {
    Page,
    Section,
}

impl Head {
    /// The head's classes, its title's and its actions'.
    pub fn classes(self) -> [&'static str; 3] {
        match self {
            Head::Page => [
                class::PAGE_HEAD,
                class::PAGE_HEAD_TITLE,
                class::PAGE_HEAD_ACTIONS,
            ],
            Head::Section => [
                class::SECTION_HEAD,
                class::SECTION_HEAD_TITLE,
                class::SECTION_HEAD_ACTIONS,
            ],
        }
    }

    fn tag(self) -> &'static str {
        match self {
            Head::Page => "h1",
            Head::Section => "h2",
        }
    }
}

/// A head's own classes: `crumb` when it leads back.
fn head_classes(head: &'static str, crumb: bool) -> Classes {
    classes!(head, crumb.then_some(class::CRUMB))
}

fn head(kind: Head, props: &HeadProps) -> Html {
    let [head, title, actions] = kind.classes();
    let help = props.help.clone().map(|help| match &props.help_testid {
        Some(testid) => with_testid(
            html! { <span class={class::CONTENTS}>{ help }</span> },
            Some(testid.clone()),
        ),
        None => help,
    });
    let tag = props.level.map_or(kind.tag(), HeadLevel::tag);
    let suffix = props.suffix.clone();
    let testid = props.title_testid.clone().or_else(|| props.testid.clone());
    let actions = (props.children != Html::default()).then(|| {
        html! { <div class={actions}>{ props.children.clone() }</div> }
    });

    let title = with_testid(
        html! {
            <@{tag} class={classes!(title, props.title_class.clone())}>
                { for props.icon.clone().map(|icon| html! { <span class={class::HEAD_ICON}>{ icon }</span> }) }
                { props.title.clone() }
            </@>
        },
        testid,
    );
    let meta = props.meta.clone();
    let trail = (!props.crumbs.is_empty()).then(|| {
        html! {
            <nav class={class::HEAD_TRAIL} aria-label={props.crumbs_label.clone()}>
                <ol class={class::HEAD_TRAIL_LIST}>
                    { for props.crumbs.iter().map(|crumb| html! {
                        <li class={class::HEAD_TRAIL_ITEM}>{ crumb.clone() }</li>
                    }) }
                </ol>
            </nav>
        }
    });
    let subtitle = props
        .subtitle
        .clone()
        .map(|subtitle| html! { <p class={class::HEAD_SUBTITLE}>{ subtitle }</p> });

    let Some(back) = props.back.clone() else {
        return with_testid(
            html! {
                <header class={classes!(head_classes(head, false), props.class.clone())}>
                    { for trail }
                    <div class={class::HEAD_HEADING}>
                        { title }
                        { for help }
                        { for suffix }
                    </div>
                    { for meta }
                    { for actions }
                    { for subtitle }
                </header>
            },
            props.head_testid.clone(),
        );
    };

    with_testid(
        html! {
            <header class={classes!(head_classes(head, true), props.class.clone())}>
                { for trail }
                <nav class={class::PAGE_HEAD_CRUMBS} aria-label={props.back_label.clone()}>
                    { back }
                    <span class={class::PAGE_HEAD_SEP} aria-hidden="true">{ "/" }</span>
                    { title }
                    { for help }
                    { for suffix }
                </nav>
                { for meta }
                { for actions }
                { for subtitle }
            </header>
        },
        props.head_testid.clone(),
    )
}

/// A page's head: its title (`h1`) with its help, and its actions.
#[function_component]
pub fn PageHead(props: &HeadProps) -> Html {
    head(Head::Page, props)
}

/// A section's head: its title (`h2`) with its help, and its actions.
#[function_component]
pub fn SectionHead(props: &HeadProps) -> Html {
    head(Head::Section, props)
}

#[cfg(test)]
mod tests {
    use super::*;
    use yew::LocalServerRenderer;

    fn render(props: HeadProps) -> String {
        futures::executor::block_on(
            LocalServerRenderer::<PageHead>::with_props(props)
                .hydratable(false)
                .render(),
        )
    }

    fn props() -> HeadProps {
        yew::props!(HeadProps {
            title: html! { <b>{ "Name" }</b> },
        })
    }

    #[test]
    fn the_help_follows_the_heading_as_its_sibling() {
        let html = render(HeadProps {
            help: Some(html! { <button>{ "?" }</button> }),
            ..props()
        });
        let heading = html.split("</h1>").next().expect("a heading");
        assert!(!heading.contains("<button>"), "{html}");
        assert!(html.contains("</h1><button>?</button>"), "{html}");
    }

    #[test]
    fn the_help_follows_the_title_in_a_crumb_too() {
        let html = render(HeadProps {
            back: Some(html! { <a href="#">{ "Back" }</a> }),
            help: Some(html! { <button>{ "?" }</button> }),
            ..props()
        });
        assert!(html.contains("</h1><button>?</button>"), "{html}");
    }

    #[test]
    fn crumbs_stack_above_the_title_and_the_subtitle_follows_the_row() {
        let html = render(HeadProps {
            crumbs: vec![
                html! { <a href="/a">{ "A" }</a> },
                html! { <a href="/b">{ "B" }</a> },
            ],
            crumbs_label: Some("Trail".into()),
            subtitle: Some(html! { { "Facts" } }),
            icon: Some(html! { { "*" } }),
            children: html! { <button>{ "Go" }</button> },
            ..props()
        });
        let at = |needle: &str| {
            html.find(needle)
                .unwrap_or_else(|| panic!("{needle} in {html}"))
        };
        assert!(at("<nav") < at("<h1"));
        assert!(at("<h1") < at("<button>Go"));
        assert!(at("<button>Go") < at("<p class=\"sockel-head-subtitle\">Facts"));
        assert!(html.contains("aria-label=\"Trail\""), "{html}");
        assert_eq!(html.matches("<li ").count(), 2, "{html}");
        assert!(html.contains("<h1 class=\"sockel-page-head-title\"><span class=\"sockel-head-icon\">*</span><b>Name</b></h1>"), "{html}");
    }

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn a_head_with_a_back_link_is_a_crumb() {
        assert_eq!(
            names(head_classes(class::PAGE_HEAD, false)),
            [class::PAGE_HEAD]
        );
        assert_eq!(
            names(head_classes(class::PAGE_HEAD, true)),
            [class::PAGE_HEAD, class::CRUMB]
        );
    }

    #[test]
    fn a_head_is_as_high_with_or_without_its_actions() {
        let sass = include_str!("../style/_heads.scss");
        let rule = sass
            .split(".sockel-page-head,\n.sockel-section-head {")
            .nth(1)
            .and_then(|rest| rest.split('}').next())
            .expect("the heads' shared rule");
        assert!(
            rule.contains("min-height: var(--control-height);"),
            "{rule}"
        );
    }

    #[test]
    fn a_label_head_is_its_lines_height_and_no_head_has_margins() {
        let sass = include_str!("../style/_heads.scss");
        let shared = sass
            .split(".sockel-page-head,\n.sockel-section-head {")
            .nth(1)
            .and_then(|rest| rest.split('}').next())
            .expect("the heads' shared rule");
        assert!(shared.contains("margin: 0;"), "{shared}");
        let label = sass
            .split(".sockel-section-head.sockel-label {")
            .nth(1)
            .and_then(|rest| rest.split('}').next())
            .expect("the label head's rule");
        assert!(label.contains("min-height: 0;"), "{label}");
    }

    #[test]
    fn a_compact_heads_icons_never_shrink_under_its_title() {
        let sass = include_str!("../style/_heads.scss");
        let actions = sass
            .split("%-phone-actions {")
            .nth(1)
            .and_then(|rest| rest.split('}').next())
            .expect("the phone actions' rule");
        assert!(actions.contains("flex-wrap: nowrap;"), "{actions}");
        assert!(actions.contains("flex-shrink: 0;"), "{actions}");
        let compact = sass
            .split(".sockel-page-head.sockel-compact {\n        > .sockel-head-heading")
            .nth(1)
            .and_then(|rest| rest.split("> .sockel-page-head-actions {").nth(1))
            .and_then(|rest| rest.split('}').next())
            .expect("the compact head's actions on a phone");
        assert!(compact.contains("@extend %-phone-actions;"), "{compact}");
    }

    #[test]
    fn a_one_row_head_without_crumbs_keeps_its_actions_on_their_row() {
        let sass = include_str!("../style/_heads.scss");
        let rule = sass
            .split(
                ".sockel-page-head.sockel-one-row:not(.sockel-crumb) > .sockel-page-head-actions {",
            )
            .nth(1)
            .and_then(|rest| rest.split('}').next())
            .expect("the one-row head's actions on a phone");
        assert!(rule.contains("@extend %-phone-actions;"), "{rule}");
        let heading = sass
            .split(".sockel-page-head.sockel-one-row {\n        flex-wrap: nowrap;")
            .nth(1)
            .and_then(|rest| rest.split("> .sockel-page-head-title {").nth(1))
            .and_then(|rest| rest.split('}').next())
            .expect("the one-row head's title on a phone");
        assert!(heading.contains("@extend %-phone-title;"), "{heading}");
    }

    #[test]
    fn the_help_stays_beside_a_wrapping_title() {
        let sass = include_str!("../style/_heads.scss");
        let rule = sass
            .split(".sockel-head-heading {")
            .nth(1)
            .and_then(|rest| rest.split('}').next())
            .expect("the heading group's rule");
        assert!(rule.contains("flex-wrap: nowrap;"), "{rule}");
    }

    #[test]
    fn a_heads_actions_shrink_with_its_column() {
        let sass = include_str!("../style/_heads.scss");
        let rule = sass
            .split(".sockel-page-head-actions,\n.sockel-section-head-actions {")
            .nth(1)
            .and_then(|rest| rest.split('}').next())
            .expect("the heads' actions rule");
        assert!(rule.contains("min-width: 0;"), "{rule}");
    }

    #[test]
    fn heads_name_their_parts_after_themselves() {
        assert_eq!(
            Head::Page.classes(),
            [
                class::PAGE_HEAD,
                class::PAGE_HEAD_TITLE,
                class::PAGE_HEAD_ACTIONS
            ]
        );
        assert_eq!(
            Head::Section.classes(),
            [
                class::SECTION_HEAD,
                class::SECTION_HEAD_TITLE,
                class::SECTION_HEAD_ACTIONS
            ]
        );
        assert_eq!((Head::Page.tag(), Head::Section.tag()), ("h1", "h2"));
    }

    #[test]
    fn a_level_names_its_heading() {
        let tags = [HeadLevel::H1, HeadLevel::H2, HeadLevel::H3, HeadLevel::H4].map(HeadLevel::tag);
        assert_eq!(tags, ["h1", "h2", "h3", "h4"]);
    }
}
