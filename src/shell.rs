//! AppShell and BottomNav: an app's frame, a top bar over a scrolling main,
//! and its section navigation, which is in the bar on a wide screen and a
//! tab bar at the bottom on a narrow one (`style/_shell.scss`).

use crate::testid::with_testid;
use yew::prelude::*;

use crate::{Count, class};

#[derive(Properties, Default, PartialEq)]
pub struct AppShellProps {
    /// What begins the top bar: a back button, the title, a switcher.
    #[prop_or_default]
    pub start: Html,
    /// What ends the top bar: status, an actions menu.
    #[prop_or_default]
    pub end: Html,
    /// The section navigation, usually a [`BottomNav`]: between `start` and
    /// `end` from `$app-shell-breakpoint` up, a tab bar at the bottom below.
    #[prop_or_default]
    pub nav: Html,
    /// The page under the bar, which scrolls.
    #[prop_or_default]
    pub children: Html,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// An app shell's classes.
pub fn app_shell() -> Classes {
    classes!(class::APP_SHELL)
}

/// An app's frame, `--app-shell-height` (the window's, `100dvh`) high: a
/// top bar of `start`, `nav` and `end` over the page, which scrolls on its
/// own, so the bar and the navigation never move. Below
/// `$app-shell-breakpoint` (`phone`) of its own width, a container query,
/// the bar holds `start` and `end` alone and `nav` is a bar at the bottom.
/// One `nav` element moves by the grid alone, so nothing re-renders and its
/// focus stays where it was.
#[function_component]
pub fn AppShell(props: &AppShellProps) -> Html {
    with_testid(
        html! {
            <div class={classes!(app_shell(), props.class.clone())}>
                <div class={class::APP_SHELL_FRAME}>
                    <header class={class::APP_SHELL_BAR}>
                        <div class={class::APP_SHELL_START}>{ props.start.clone() }</div>
                        <div class={class::APP_SHELL_END}>{ props.end.clone() }</div>
                    </header>
                    <div class={class::APP_SHELL_NAV}>{ props.nav.clone() }</div>
                    <main class={classes!(class::APP_SHELL_MAIN, class::SCROLL_AREA)}>
                        { props.children.clone() }
                    </main>
                </div>
            </div>
        },
        props.testid.clone(),
    )
}

/// One section of a [`BottomNav`].
#[derive(Clone, Debug, PartialEq)]
pub struct BottomNavItem {
    pub value: AttrValue,
    pub label: AttrValue,
    /// An icon the app supplies, over the label in the bottom bar and
    /// before it in the top bar.
    pub icon: Option<Html>,
    /// A link to this address; without one the section is a button.
    pub href: Option<AttrValue>,
    /// What waits in the section, a [`Count`] on it; none is hidden but
    /// keeps its box.
    pub count: Option<u32>,
}

impl BottomNavItem {
    pub fn new(value: impl Into<AttrValue>, label: impl Into<AttrValue>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            icon: None,
            href: None,
            count: None,
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

    pub fn count(mut self, count: u32) -> Self {
        self.count = Some(count);
        self
    }
}

#[derive(Properties, PartialEq)]
pub struct BottomNavProps {
    pub items: Vec<BottomNavItem>,
    /// The current section's value (`aria-current="page"`).
    #[prop_or_default]
    pub value: Option<AttrValue>,
    /// Called with a section's value when it is pressed (a link's too, so
    /// an app's router can take it).
    #[prop_or_default]
    pub on_select: Callback<AttrValue>,
    /// What the navigation is for (`aria-label` of the `nav`).
    pub label: AttrValue,
    /// The largest count shown whole, above it `max+`.
    #[prop_or(99)]
    pub max: u32,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

impl Default for BottomNavProps {
    fn default() -> Self {
        Self {
            items: Default::default(),
            value: Default::default(),
            on_select: Default::default(),
            label: Default::default(),
            max: ::yew::html::IntoPropValue::into_prop_value(99),
            class: Default::default(),
            testid: Default::default(),
        }
    }
}

/// A bottom navigation's classes.
pub fn bottom_nav() -> Classes {
    classes!(class::BOTTOM_NAV)
}

/// An app's sections: a row of links or buttons, the current one marked by
/// `aria-current` and by colour, never weight. In an [`AppShell`]'s top bar
/// they sit in a line; below its breakpoint they share the bottom bar
/// evenly, each `--touch-height` at least, icon over label, the count on
/// the icon's corner, clear of the window's safe area.
#[function_component]
pub fn BottomNav(props: &BottomNavProps) -> Html {
    with_testid(
        html! {
            <nav class={classes!(bottom_nav(), props.class.clone())}
                aria-label={props.label.clone()}>
                <ul class={class::BOTTOM_NAV_ITEMS}>
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
                                <span class={class::BOTTOM_NAV_ICON} aria-hidden="true">
                                    { item.icon.clone().unwrap_or_default() }
                                </span>
                                <span class={class::BOTTOM_NAV_LABEL}>{ item.label.clone() }</span>
                                if let Some(count) = item.count {
                                    <Count class={class::BOTTOM_NAV_COUNT} value={count} max={props.max} hide_zero=true />
                                }
                            </>
                        };
                        html! {
                            <li class={class::BOTTOM_NAV_ENTRY}>
                                if let Some(href) = item.href.clone() {
                                    <a class={class::BOTTOM_NAV_ITEM} {href} aria-current={aria_current} {onclick}>
                                        { content }
                                    </a>
                                } else {
                                    <button type="button" class={class::BOTTOM_NAV_ITEM}
                                        aria-current={aria_current} {onclick}>
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

    fn render_nav(value: Option<&str>, items: Vec<BottomNavItem>) -> String {
        futures::executor::block_on(
            yew::LocalServerRenderer::<BottomNav>::with_props(yew::props!(BottomNavProps {
                items: items,
                value: value.map(AttrValue::from),
                label: AttrValue::from("Sections"),
            }))
            .render(),
        )
    }

    #[test]
    fn the_current_section_is_the_page() {
        let html = render_nav(
            Some("b"),
            vec![
                BottomNavItem::new("a", "A").href("/a"),
                BottomNavItem::new("b", "B"),
            ],
        );
        let link = html.split("<a ").nth(1).and_then(|a| a.split('>').next());
        assert!(link.is_some_and(|a| !a.contains("aria-current")), "{html}");
        let button = html
            .split("<button ")
            .nth(1)
            .and_then(|b| b.split('>').next());
        assert!(
            button.is_some_and(|b| b.contains(r#"aria-current="page""#)),
            "{html}"
        );
    }

    #[test]
    fn a_count_shows_only_where_one_is_given() {
        let html = render_nav(
            None,
            vec![
                BottomNavItem::new("a", "A").count(3),
                BottomNavItem::new("b", "B"),
            ],
        );
        assert_eq!(html.matches(class::BOTTOM_NAV_COUNT).count(), 1, "{html}");
        assert!(html.contains(">3<"), "{html}");
    }

    #[test]
    fn a_shell_puts_its_nav_beside_the_bar_and_the_page_under_it() {
        let html = futures::executor::block_on(
            yew::LocalServerRenderer::<AppShell>::with_props(yew::props!(AppShellProps {
                start: html! { "Title" },
                nav: html! { "Nav" },
                children: html! { "Page" },
            }))
            .render(),
        );
        let bar = html.find(class::APP_SHELL_BAR).expect("a bar");
        let nav = html.find(class::APP_SHELL_NAV).expect("a nav");
        let main = html.find("<main").expect("a main");
        assert!(bar < nav && nav < main, "{html}");
    }
}
