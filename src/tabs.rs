//! Tabs, SubTabs and LinkTabs (`style/_tabs.scss`).

use crate::class;
use crate::roving::{self, Arrows};
use crate::testid::with_testid;
use yew::prelude::*;

/// One tab of [`Tabs`].
#[derive(Clone, Debug, PartialEq)]
pub struct TabItem {
    pub value: AttrValue,
    pub label: AttrValue,
    /// An icon the app supplies, before the label; hidden on a phone.
    pub icon: Option<Html>,
    pub disabled: bool,
    /// Extra classes on the tab.
    pub class: Classes,
    /// The tab's `data-testid`.
    pub testid: Option<AttrValue>,
    /// A tip (`data-tip`) that also names the tab (`aria-label`): for a tab
    /// whose label is an icon.
    pub tip: Option<AttrValue>,
    /// Markup for the label where text is not enough (a flag, a mark); the
    /// `label` is then the tab's accessible name. Not hidden on a phone.
    pub content: Option<Html>,
    /// Markup right after the tab, in the tab list: a control that belongs
    /// to the tab (a remove cross), not itself a tab.
    pub after: Option<Html>,
    /// Below the phone breakpoint, `after` shows only while its tab is
    /// selected (a remove cross for the open tab alone).
    pub after_selected_only: bool,
    /// The tab's content is refused (`aria-invalid`), and the tab shows it
    /// in `--danger`.
    pub invalid: bool,
    /// The address a [`LinkTabs`] tab links to.
    pub href: Option<AttrValue>,
    /// Called on a click of a [`LinkTabs`] tab, before the browser follows
    /// it (an app's router prevents that and shows the page in place).
    pub onclick: Option<Callback<MouseEvent>>,
}

impl TabItem {
    pub fn new(value: impl Into<AttrValue>, label: impl Into<AttrValue>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            icon: None,
            disabled: false,
            class: Classes::new(),
            testid: None,
            tip: None,
            content: None,
            after: None,
            after_selected_only: false,
            invalid: false,
            href: None,
            onclick: None,
        }
    }

    pub fn icon(mut self, icon: Html) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn class(mut self, class: impl Into<Classes>) -> Self {
        self.class = class.into();
        self
    }

    pub fn testid(mut self, testid: impl Into<AttrValue>) -> Self {
        self.testid = Some(testid.into());
        self
    }

    pub fn tip(mut self, tip: impl Into<AttrValue>) -> Self {
        self.tip = Some(tip.into());
        self
    }

    pub fn content(mut self, content: Html) -> Self {
        self.content = Some(content);
        self
    }

    pub fn after(mut self, after: Html) -> Self {
        self.after = Some(after);
        self
    }

    pub fn after_selected_only(mut self, only: bool) -> Self {
        self.after_selected_only = only;
        self
    }

    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    pub fn href(mut self, href: impl Into<AttrValue>) -> Self {
        self.href = Some(href.into());
        self
    }

    pub fn onclick(mut self, onclick: Callback<MouseEvent>) -> Self {
        self.onclick = Some(onclick);
        self
    }
}

#[derive(Properties, Default, PartialEq)]
pub struct TabsProps {
    #[prop_or_default]
    pub class: Classes,
    pub tabs: Vec<TabItem>,
    /// The selected tab's value; none selected when it matches no tab.
    #[prop_or_default]
    pub value: Option<AttrValue>,
    /// Called with the newly selected value, by click or arrow key.
    pub on_change: Callback<AttrValue>,
    /// What the tabs switch between (`aria-label` of the tablist).
    pub label: AttrValue,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// Segmented: the tabs sit in a tinted pill and the selected one is a
    /// raised segment, instead of an underline.
    #[prop_or_default]
    pub segmented: bool,
    /// The list spans its container and the tabs share its width equally.
    #[prop_or_default]
    pub fill: bool,
    /// Stacked, a bar: each tab's icon over its one-line label, every tab the
    /// same width and the bar spanning its container (`STACKED`).
    #[prop_or_default]
    pub stacked: bool,
}

/// The props of [`LinkTabs`].
#[derive(Properties, Default, PartialEq)]
pub struct LinkTabsProps {
    #[prop_or_default]
    pub class: Classes,
    /// The tabs, each with its `href`.
    pub tabs: Vec<TabItem>,
    /// The current tab's value (`aria-current="page"`).
    #[prop_or_default]
    pub value: Option<AttrValue>,
    /// Where the tabs lead (`aria-label` of the `nav`).
    pub label: AttrValue,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    #[prop_or_default]
    pub segmented: bool,
    #[prop_or_default]
    pub fill: bool,
    /// Stacked, as for [`TabsProps`].
    #[prop_or_default]
    pub stacked: bool,
    /// At the small control height, as [`SubTabs`].
    #[prop_or_default]
    pub small: bool,
}

/// A tab list's classes: `tabs`, and `small` for the smaller variant.
pub fn tabs(small: bool) -> Classes {
    classes!(class::TABS, small.then_some(class::SMALL))
}

fn list(small: bool, segmented: bool, fill: bool, stacked: bool, class: &Classes) -> Classes {
    classes!(
        tabs(small),
        segmented.then_some(class::SEGMENTED),
        fill.then_some(class::FILL),
        stacked.then_some(class::STACKED),
        class.clone()
    )
}

fn face(tab: &TabItem, stacked: bool) -> Html {
    let stacked = stacked.then_some(class::STACKED);
    html! {
        <>
            { for tab.icon.clone().map(|icon| html! {
                <span class={classes!(class::TAB_ICON, stacked)} aria-hidden="true">{ icon }</span>
            }) }
            <span class={classes!(class::TAB_LABEL, stacked)}>
                { match &tab.content {
                    Some(content) => content.clone(),
                    None => html! { { tab.label.clone() } },
                } }
            </span>
        </>
    }
}

/// The markup after a tab: bare, or when `after_selected_only` in a
/// `tab-after` span that a phone hides unless the tab is `selected`.
fn after(tab: &TabItem, selected: bool) -> Html {
    match (&tab.after, tab.after_selected_only) {
        (Some(after), true) => html! {
            <span class={class::TAB_AFTER} data-selected={selected.to_string()}>{ after.clone() }</span>
        },
        (after, _) => after.clone().unwrap_or_default(),
    }
}

fn name(tab: &TabItem) -> Option<AttrValue> {
    tab.tip
        .clone()
        .or_else(|| tab.content.as_ref().map(|_| tab.label.clone()))
}

fn disabled(tabs: &[TabItem]) -> Vec<bool> {
    tabs.iter().map(|t| t.disabled).collect()
}

fn selected(tabs: &[TabItem], value: Option<&AttrValue>) -> Option<usize> {
    tabs.iter().position(|t| Some(&t.value) == value)
}

fn render(props: &TabsProps, small: bool) -> Html {
    let off = disabled(&props.tabs);
    let stop = roving::stop(&off, selected(&props.tabs, props.value.as_ref()));

    let onkeydown = {
        let on_change = props.on_change.clone();
        let tabs = props.tabs.clone();
        Callback::from(move |e: KeyboardEvent| {
            let off = disabled(&tabs);
            if let Some(next) = roving::keydown(&e, "tab", &off, stop, Arrows::Horizontal) {
                on_change.emit(tabs[next].value.clone());
            }
        })
    };

    with_testid(
        html! {
            <div role="tablist" class={list(small, props.segmented, props.fill, props.stacked, &props.class)}
                aria-label={props.label.clone()}
                {onkeydown}>
                { for props.tabs.iter().enumerate().map(|(i, tab)| {
                    let selected = props.value.as_ref() == Some(&tab.value);
                    let onclick = {
                        let on_change = props.on_change.clone();
                        let value = tab.value.clone();
                        Callback::from(move |_: MouseEvent| on_change.emit(value.clone()))
                    };
                    html! {
                        <>
                        { with_testid(html! { <button type="button" role="tab" class={classes!(class::TAB, props.stacked.then_some(class::STACKED), tab.invalid.then_some(class::INVALID), tab.class.clone())}
                            aria-selected={selected.to_string()}
                            aria-invalid={tab.invalid.then_some("true")}
                            aria-label={name(tab)}
                            data-tip={tab.tip.clone()}
                            data-value={tab.value.clone()}
                            tabindex={if stop == Some(i) { "0" } else { "-1" }}
                            disabled={tab.disabled}
                            {onclick}>
                            { face(tab, props.stacked) }
                        </button> }, tab.testid.clone()) }
                        { after(tab, selected) }
                        </>
                    }
                }) }
            </div>
        },
        props.testid.clone(),
    )
}

/// A row of tabs (`role="tablist"`): the selected one marked by its colour
/// and an underline drawn inside its box, so none changes size when selected.
/// One tab stop; Left, Right, Home and End select and focus a tab. Tabs past the
/// width scroll sideways, none is hidden.
#[function_component]
pub fn Tabs(props: &TabsProps) -> Html {
    render(props, false)
}

/// [`Tabs`] at the small control height, for tabs inside a panel.
#[function_component]
pub fn SubTabs(props: &TabsProps) -> Html {
    render(props, true)
}

/// Tabs that are links to pages, drawn as [`Tabs`]: a `nav` of links, the
/// current one `aria-current="page"`. Each link is a tab stop of its own and
/// the arrows are the page's, as for any links. A disabled tab has no
/// `href` (`aria-disabled`).
#[function_component]
pub fn LinkTabs(props: &LinkTabsProps) -> Html {
    with_testid(
        html! {
            <nav class={list(props.small, props.segmented, props.fill, props.stacked, &props.class)}
                aria-label={props.label.clone()}>
                { for props.tabs.iter().map(|tab| {
                    let current = props.value.as_ref() == Some(&tab.value);
                    let onclick = tab.onclick.clone().filter(|_| !tab.disabled);
                    html! {
                        <>
                        { with_testid(html! { <a class={classes!(class::TAB, props.stacked.then_some(class::STACKED), tab.invalid.then_some(class::INVALID), tab.class.clone())}
                            href={tab.href.clone().filter(|_| !tab.disabled)}
                            aria-current={current.then_some("page")}
                            aria-disabled={tab.disabled.then_some("true")}
                            aria-invalid={tab.invalid.then_some("true")}
                            aria-label={name(tab)}
                            data-tip={tab.tip.clone()}
                            data-value={tab.value.clone()}
                            {onclick}>
                            { face(tab, props.stacked) }
                        </a> }, tab.testid.clone()) }
                        { after(tab, current) }
                        </>
                    }
                }) }
            </nav>
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

    fn items() -> Vec<TabItem> {
        ["a", "b", "c"]
            .iter()
            .map(|v| TabItem::new(*v, v.to_uppercase()))
            .collect()
    }

    fn props(value: Option<&str>, tabs: Vec<TabItem>) -> TabsProps {
        yew::props!(TabsProps {
            tabs: tabs,
            value: value.map(AttrValue::from),
            on_change: Callback::noop(),
            label: "Sections"
        })
    }

    fn html_of<C: BaseComponent<Properties = TabsProps>>(p: TabsProps) -> String {
        futures::executor::block_on(
            yew::LocalServerRenderer::<C>::with_props(p)
                .hydratable(false)
                .render(),
        )
    }

    #[test]
    fn each_modifier_is_its_class() {
        assert_eq!(names(tabs(false)), [class::TABS]);
        assert_eq!(names(tabs(true)), [class::TABS, class::SMALL]);
    }

    #[test]
    fn sub_tabs_are_the_small_variant() {
        let html = html_of::<SubTabs>(props(Some("a"), items()));
        assert!(html.contains(class::SMALL), "{html}");
        let html = html_of::<Tabs>(props(Some("a"), items()));
        assert!(!html.contains(class::SMALL), "{html}");
    }

    fn step(off: &[bool], from: usize, key: &str) -> Option<usize> {
        roving::step(off, Some(from), key, Arrows::Horizontal)
    }

    #[test]
    fn arrows_home_and_end_move_and_wrap() {
        let off = disabled(&items());
        assert_eq!(step(&off, 0, "ArrowRight"), Some(1));
        assert_eq!(step(&off, 2, "ArrowRight"), Some(0));
        assert_eq!(step(&off, 0, "ArrowLeft"), Some(2));
        assert_eq!(step(&off, 1, "Home"), Some(0));
        assert_eq!(step(&off, 0, "End"), Some(2));
        assert_eq!(step(&off, 0, "Tab"), None);
        // Up and Down are the page's.
        assert_eq!(step(&off, 0, "ArrowDown"), None);
        assert_eq!(step(&off, 1, "ArrowUp"), None);
    }

    #[test]
    fn arrows_skip_a_disabled_tab() {
        let mut t = items();
        t[1] = t[1].clone().disabled(true);
        assert_eq!(step(&disabled(&t), 0, "ArrowRight"), Some(2));
        t[0] = t[0].clone().disabled(true);
        assert_eq!(step(&disabled(&t), 2, "Home"), Some(2));
    }

    #[test]
    fn the_selected_tab_alone_is_selected_and_the_tab_stop() {
        let html = html_of::<Tabs>(props(Some("b"), items()));
        assert_eq!(html.matches("aria-selected=\"true\"").count(), 1, "{html}");
        assert_eq!(html.matches("aria-selected=\"false\"").count(), 2, "{html}");
        assert_eq!(html.matches("tabindex=\"0\"").count(), 1, "{html}");
        assert!(html.find("tabindex=\"0\"") > html.find("aria-selected=\"false\""));
        assert!(html.contains("role=\"tablist\""), "{html}");
    }

    #[test]
    fn a_tab_takes_a_testid_tip_class_and_markup_after() {
        let t = vec![
            TabItem::new("a", "A")
                .testid("tab-a")
                .tip("Tip A")
                .class("mine")
                .content(html! { <b>{ "Rich" }</b> })
                .after(html! { <i data-after="" /> }),
        ];
        let html = html_of::<Tabs>(props(Some("a"), t));
        assert!(
            html.contains(&format!("{}=\"tab-a\"", crate::testid::ATTR)),
            "{html}"
        );
        assert!(html.contains("data-tip=\"Tip A\""), "{html}");
        assert!(html.contains("aria-label=\"Tip A\""), "{html}");
        assert!(html.contains("mine"), "{html}");
        assert!(html.contains("<b>Rich</b>"), "{html}");
        assert!(html.find("data-after") > html.find("</button>"), "{html}");
    }

    #[test]
    fn after_selected_only_wraps_after_with_the_tab_state() {
        let t = vec![
            TabItem::new("a", "A")
                .after(html! { <i data-after="a" /> })
                .after_selected_only(true),
            TabItem::new("b", "B").after(html! { <i data-after="b" /> }),
        ];
        let html = html_of::<Tabs>(props(Some("a"), t));
        assert!(html.contains("data-selected=\"true\""), "{html}");
        assert_eq!(html.matches("data-selected").count(), 1, "{html}");
    }

    #[test]
    fn an_invalid_tab_is_aria_invalid_and_segmented_is_a_class() {
        let t = vec![TabItem::new("a", "A").invalid(true), TabItem::new("b", "B")];
        let html = html_of::<Tabs>(props(Some("b"), t));
        assert_eq!(html.matches("aria-invalid=\"true\"").count(), 1, "{html}");
        assert!(html.contains(class::INVALID), "{html}");
        assert!(!html.contains(class::SEGMENTED), "{html}");
        let mut p = props(Some("a"), items());
        p.segmented = true;
        let html = html_of::<Tabs>(p);
        assert!(html.contains(class::SEGMENTED), "{html}");
    }

    #[test]
    fn a_tab_names_its_value_and_fill_is_a_class() {
        let html = html_of::<Tabs>(props(Some("a"), items()));
        assert!(html.contains("data-value=\"b\""), "{html}");
        assert!(!html.contains(class::FILL), "{html}");
        let mut p = props(Some("a"), items());
        p.fill = true;
        assert!(html_of::<Tabs>(p).contains(class::FILL));
    }

    #[test]
    fn stacked_is_a_class_on_the_list_the_tabs_and_their_parts() {
        let t = vec![TabItem::new("a", "A").icon(html! { <i /> })];
        let off = html_of::<Tabs>(props(Some("a"), t.clone()));
        assert!(!off.contains(class::STACKED), "{off}");
        let mut p = props(Some("a"), t);
        p.stacked = true;
        let on = html_of::<Tabs>(p);
        // The list, the tab, its icon and its label.
        assert_eq!(on.matches(class::STACKED).count(), 4, "{on}");
    }

    #[test]
    fn link_tabs_stack_too() {
        let html = futures::executor::block_on(
            yew::LocalServerRenderer::<LinkTabs>::with_props(yew::props!(LinkTabsProps {
                tabs: vec![TabItem::new("a", "A").href("/a")],
                label: "Pages",
                stacked: true
            }))
            .hydratable(false)
            .render(),
        );
        assert!(html.contains(class::STACKED), "{html}");
    }

    fn links(value: Option<&str>) -> String {
        let tabs = vec![
            TabItem::new("a", "A").href("/a").testid("tab-a"),
            TabItem::new("b", "B").href("/b"),
            TabItem::new("c", "C").href("/c").disabled(true),
        ];
        futures::executor::block_on(
            yew::LocalServerRenderer::<LinkTabs>::with_props(yew::props!(LinkTabsProps {
                tabs: tabs,
                value: value.map(AttrValue::from),
                label: "Pages",
                segmented: true
            }))
            .hydratable(false)
            .render(),
        )
    }

    #[test]
    fn link_tabs_are_links_the_current_one_aria_current() {
        let html = links(Some("b"));
        assert!(html.starts_with("<nav"), "{html}");
        assert!(!html.contains("role=\"tablist\""), "{html}");
        assert!(!html.contains("tabindex"), "{html}");
        assert!(html.contains(class::SEGMENTED), "{html}");
        assert_eq!(html.matches("aria-current=\"page\"").count(), 1, "{html}");
        assert!(
            html.find("aria-current") > html.find("href=\"/b\""),
            "{html}"
        );
        assert!(
            html.contains(&format!("{}=\"tab-a\"", crate::testid::ATTR)),
            "{html}"
        );
        assert_eq!(links(None).matches("aria-current").count(), 0);
    }

    #[test]
    fn a_disabled_link_tab_has_no_href() {
        let html = links(None);
        assert_eq!(html.matches("href=").count(), 2, "{html}");
        assert!(!html.contains("/c"), "{html}");
        assert_eq!(html.matches("aria-disabled=\"true\"").count(), 1, "{html}");
    }

    #[test]
    fn with_none_selected_the_first_enabled_is_the_tab_stop() {
        let mut t = items();
        t[0] = t[0].clone().disabled(true);
        let html = html_of::<Tabs>(props(None, t));
        assert_eq!(html.matches("aria-selected=\"true\"").count(), 0, "{html}");
        assert_eq!(html.matches("tabindex=\"0\"").count(), 1, "{html}");
        assert!(html.find("tabindex=\"0\"") > html.find("disabled"));
    }
}
