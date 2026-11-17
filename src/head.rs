//! PageHead and SectionHead (`style/_heads.scss`).

use crate::class;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct HeadProps {
    pub title: AttrValue,
    /// What explains the page or section (a help tip), in the title's
    /// line after its text.
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
    fn tag(self) -> &'static str {
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
        Some(testid) => {
            html! { <span class={class::CONTENTS} data-testid={testid.clone()}>{ help }</span> }
        }
        None => help,
    });
    let tag = props.level.map_or(kind.tag(), HeadLevel::tag);
    let suffix = props.suffix.clone();
    let testid = props.title_testid.clone().or_else(|| props.testid.clone());
    let actions = (props.children != Html::default()).then(|| {
        html! { <div class={actions}>{ props.children.clone() }</div> }
    });

    let title = html! {
        <@{tag} class={classes!(title, props.title_class.clone())} data-testid={testid}>
            { props.title.clone() }
            { for help }
        </@>
    };
    let meta = props.meta.clone();

    let Some(back) = props.back.clone() else {
        return html! {
            <header class={classes!(head_classes(head, false), props.class.clone())} data-testid={props.head_testid.clone()}>
                { title }
                { for suffix }
                { for meta }
                { for actions }
            </header>
        };
    };

    html! {
        <header class={classes!(head_classes(head, true), props.class.clone())} data-testid={props.head_testid.clone()}>
            <nav class={class::PAGE_HEAD_CRUMBS} aria-label={props.back_label.clone()}>
                { back }
                <span class={class::PAGE_HEAD_SEP} aria-hidden="true">{ "/" }</span>
                { title }
                { for suffix }
            </nav>
            { for meta }
            { for actions }
        </header>
    }
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
            .split(".plinth-page-head,\n.plinth-section-head {")
            .nth(1)
            .and_then(|rest| rest.split('}').next())
            .expect("the heads' shared rule");
        assert!(
            rule.contains("min-height: var(--control-height);"),
            "{rule}"
        );
    }

    #[test]
    fn a_heads_actions_shrink_with_its_column() {
        let sass = include_str!("../style/_heads.scss");
        let rule = sass
            .split(".plinth-page-head-actions,\n.plinth-section-head-actions {")
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
