//! NavList (`style/_tabs.scss`).

use crate::class;
use crate::testid::with_testid;
use yew::prelude::*;

/// One entry of a [`NavList`].
#[derive(Clone, Debug, PartialEq)]
pub struct NavItem {
    pub value: AttrValue,
    pub label: AttrValue,
    /// An icon the app supplies, before the label.
    pub icon: Option<Html>,
    /// A link to this address; without one the entry is a button.
    pub href: Option<AttrValue>,
    pub disabled: bool,
}

impl NavItem {
    pub fn new(value: impl Into<AttrValue>, label: impl Into<AttrValue>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            icon: None,
            href: None,
            disabled: false,
        }
    }

    pub fn icon(mut self, icon: Html) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn href(mut self, href: impl Into<AttrValue>) -> Self {
        self.href = Some(href.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Properties, Default, PartialEq)]
pub struct NavListProps {
    #[prop_or_default]
    pub class: Classes,
    pub items: Vec<NavItem>,
    /// The current entry's value (`aria-current="page"`).
    #[prop_or_default]
    pub value: Option<AttrValue>,
    /// Called with an entry's value when it is pressed.
    #[prop_or_default]
    pub on_select: Callback<AttrValue>,
    /// What the navigation is for (`aria-label` of the `nav`).
    pub label: AttrValue,
    /// Below the phone breakpoint, the column folds into one row that
    /// scrolls sideways ([`class::FOLD`]).
    #[prop_or_default]
    pub fold: bool,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// A navigation list's classes.
pub fn nav_list() -> Classes {
    classes!(class::NAV_LIST)
}

/// A side or settings navigation: a column of links or buttons, the current
/// one marked by `aria-current` and by colour and background, never weight.
#[function_component]
pub fn NavList(props: &NavListProps) -> Html {
    with_testid(
        html! {
            <nav class={classes!(nav_list(), props.fold.then_some(class::FOLD), props.class.clone())}
                aria-label={props.label.clone()}>
                <ul class={class::NAV_LIST_ITEMS}>
                    { for props.items.iter().map(|item| {
                        let current = props.value.as_ref() == Some(&item.value);
                        let aria_current = current.then_some("page");
                        let onclick = {
                            let on_select = props.on_select.clone();
                            let value = item.value.clone();
                            Callback::from(move |_: MouseEvent| on_select.emit(value.clone()))
                        };
                        let content = html! {
                            <>
                                { for item.icon.clone().map(|icon| html! {
                                    <span class={class::NAV_ICON} aria-hidden="true">{ icon }</span>
                                }) }
                                <span class={class::NAV_LABEL}>{ item.label.clone() }</span>
                            </>
                        };
                        html! {
                            <li>
                                if let Some(href) = item.href.clone().filter(|_| !item.disabled) {
                                    <a class={class::NAV_ITEM} {href} aria-current={aria_current} {onclick}>
                                        { content }
                                    </a>
                                } else {
                                    <button type="button" class={class::NAV_ITEM}
                                        aria-current={aria_current}
                                        disabled={item.disabled} {onclick}>
                                        { content }
                                    </button>
                                }
                            </li>
                        }
                    }) }
                </ul>
            </nav>
        },
        props.testid.clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(value: Option<&str>, items: Vec<NavItem>) -> String {
        render_with(value, items, false)
    }

    fn render_with(value: Option<&str>, items: Vec<NavItem>, fold: bool) -> String {
        futures::executor::block_on(
            yew::LocalServerRenderer::<NavList>::with_props(yew::props!(NavListProps {
                items: items,
                value: value.map(AttrValue::from),
                label: "Settings",
                fold: fold
            }))
            .hydratable(false)
            .render(),
        )
    }

    fn items() -> Vec<NavItem> {
        vec![
            NavItem::new("a", "Profile").href("#a"),
            NavItem::new("b", "Account"),
            NavItem::new("c", "Billing").href("#c").disabled(true),
        ]
    }

    #[test]
    fn the_nav_list_is_its_class() {
        let names: Vec<String> = nav_list().into_iter().map(|c| c.to_string()).collect();
        assert_eq!(names, [class::NAV_LIST]);
    }

    #[test]
    fn the_current_entry_alone_has_aria_current() {
        let html = render(Some("b"), items());
        assert_eq!(html.matches("aria-current=\"page\"").count(), 1, "{html}");
        assert!(html.find("aria-current").unwrap() > html.find("Profile").unwrap());
        assert_eq!(render(None, items()).matches("aria-current").count(), 0);
    }

    #[test]
    fn a_link_has_an_href_and_a_disabled_one_is_a_disabled_button() {
        let html = render(None, items());
        assert_eq!(html.matches("<a href").count(), 1, "{html}");
        assert!(html.contains("href=\"#a\""), "{html}");
        assert!(!html.contains("#c"), "{html}");
        assert_eq!(html.matches("disabled=").count(), 1, "{html}");
    }

    #[test]
    fn a_folding_list_has_the_fold_modifier() {
        assert!(!render(None, items()).contains(class::FOLD));
        let html = render_with(None, items(), true);
        assert!(
            html.contains(&format!("{} {}", class::NAV_LIST, class::FOLD)),
            "{html}"
        );
    }
}
