//! PageHead and SectionHead (`style/_heads.scss`).

use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct HeadProps {
    pub title: AttrValue,
    /// What explains the page or section (a help tip), in the title's
    /// line after its text.
    #[prop_or_default]
    pub help: Option<Html>,
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

/// Which head it is: its classes and its title's element.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Head {
    Page,
    Section,
}

impl Head {
    /// The head's classes, its title's and its actions' (spelled out, so
    /// the class guard sees them).
    pub fn classes(self) -> [&'static str; 3] {
        match self {
            Head::Page => ["page-head", "page-head-title", "page-head-actions"],
            Head::Section => ["section-head", "section-head-title", "section-head-actions"],
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
    classes!(head, crumb.then_some("crumb"))
}

fn head(kind: Head, props: &HeadProps) -> Html {
    let [head, title, actions] = kind.classes();
    let help = props.help.clone();
    let actions = (props.children != Html::default()).then(|| {
        html! { <div class={actions}>{ props.children.clone() }</div> }
    });

    let title = html! {
        <@{kind.tag()} class={title} data-testid={props.testid.clone()}>
            { props.title.clone() }
            { for help }
        </@>
    };
    let meta = props.meta.clone();

    let Some(back) = props.back.clone() else {
        return html! {
            <header class={classes!(head_classes(head, false), props.class.clone())} data-testid={props.head_testid.clone()}>
                { title }
                { for meta }
                { for actions }
            </header>
        };
    };

    html! {
        <header class={classes!(head_classes(head, true), props.class.clone())} data-testid={props.head_testid.clone()}>
            <nav class="page-head-crumbs" aria-label={props.back_label.clone()}>
                { back }
                <span class="page-head-sep" aria-hidden="true">{ "/" }</span>
                { title }
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
        assert_eq!(names(head_classes("page-head", false)), ["page-head"]);
        assert_eq!(
            names(head_classes("page-head", true)),
            ["page-head", "crumb"]
        );
    }

    #[test]
    fn heads_name_their_parts_after_themselves() {
        assert_eq!(
            Head::Page.classes(),
            ["page-head", "page-head-title", "page-head-actions"]
        );
        assert_eq!(
            Head::Section.classes(),
            ["section-head", "section-head-title", "section-head-actions"]
        );
        assert_eq!((Head::Page.tag(), Head::Section.tag()), ("h1", "h2"));
    }
}
