//! Tabs and SubTabs (`style/_tabs.scss`).

use crate::class;
use crate::roving::{self, Arrows};
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
}

#[derive(Properties, PartialEq)]
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
}

/// A tab list's classes: `tabs`, and `small` for the smaller variant.
pub fn tabs(small: bool) -> Classes {
    classes!(class::TABS, small.then_some(class::SMALL))
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

    html! {
        <div role="tablist" class={classes!(tabs(small), props.class.clone())}
            aria-label={props.label.clone()}
            data-testid={props.testid.clone()}
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
                    <button type="button" role="tab" class={classes!(class::TAB, tab.class.clone())}
                        aria-selected={selected.to_string()}
                        aria-label={tab.tip.clone().or_else(|| tab.content.as_ref().map(|_| tab.label.clone()))}
                        data-tip={tab.tip.clone()}
                        data-testid={tab.testid.clone()}
                        tabindex={if stop == Some(i) { "0" } else { "-1" }}
                        disabled={tab.disabled}
                        {onclick}>
                        { for tab.icon.clone().map(|icon| html! {
                            <span class={class::TAB_ICON} aria-hidden="true">{ icon }</span>
                        }) }
                        <span class={class::TAB_LABEL}>
                            { match &tab.content {
                                Some(content) => content.clone(),
                                None => html! { { tab.label.clone() } },
                            } }
                        </span>
                    </button>
                    { for tab.after.clone() }
                    </>
                }
            }) }
        </div>
    }
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
                .after(html! { <i class="cross" /> }),
        ];
        let html = html_of::<Tabs>(props(Some("a"), t));
        assert!(html.contains("data-testid=\"tab-a\""), "{html}");
        assert!(html.contains("data-tip=\"Tip A\""), "{html}");
        assert!(html.contains("aria-label=\"Tip A\""), "{html}");
        assert!(html.contains("mine"), "{html}");
        assert!(html.contains("<b>Rich</b>"), "{html}");
        assert!(html.find("cross") > html.find("</button>"), "{html}");
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
