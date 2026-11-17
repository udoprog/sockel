use yew::prelude::*;

use plinth::{Glyph, Icon, NavItem, NavList, SubTabs, TabItem, Tabs, nav_list, tabs};

use crate::class;
use crate::code::{El, snippet};
use crate::controls::{COUNTS, Demo, Select, Text, Toggle, bind};

const SELECTED: &[(&str, usize)] = &[("1st", 0), ("2nd", 1), ("3rd", 2)];

const GLYPHS: [Glyph; 3] = [Glyph::Plus, Glyph::Search, Glyph::Close];

fn label(text: Text, prefix: &str, n: usize) -> String {
    text.of(&format!("{prefix} {}", n + 1))
}

fn glyph(n: usize) -> Glyph {
    GLYPHS[n % GLYPHS.len()]
}

#[function_component]
pub fn TabsPage() -> Html {
    let count = use_state(|| 3usize);
    let text = use_state(|| Text::Short);
    let selected = use_state(|| 0usize);
    let icons = use_state(|| false);
    let disabled = use_state(|| false);
    let small = use_state(|| false);

    let controls = html! {
        <>
            <Select<usize> label="Tabs" options={COUNTS} value={*count} on_change={bind(&count)} />
            <Select<Text> label="Text" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <Select<usize> label="Selected" options={SELECTED} value={*selected} on_change={bind(&selected)} />
            <Toggle label="icons" value={*icons} on_change={bind(&icons)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
            <Toggle label="small" value={*small} on_change={bind(&small)} />
        </>
    };

    let items: Vec<TabItem> = (0..*count)
        .map(|n| {
            let item = TabItem::new(format!("tab-{n}"), label(*text, "Tab", n))
                .disabled(*disabled && n == 1);
            if *icons {
                item.icon(html! { <Icon glyph={glyph(n)} /> })
            } else {
                item
            }
        })
        .collect();
    let current = (*selected).min(count.saturating_sub(1));
    let on_change = {
        let selected = selected.clone();
        Callback::from(move |value: AttrValue| {
            if let Some(n) = value
                .strip_prefix("tab-")
                .and_then(|n| n.parse::<usize>().ok())
            {
                selected.set(n);
            }
        })
    };

    let mut lets = vec!["let tabs = vec![".to_owned()];
    lets.extend((0..(*count).min(3)).map(|n| {
        let then = match (*icons, *disabled && n == 1) {
            (true, true) => {
                format!(
                    ".icon(html! {{ <Icon glyph={{Glyph::{:?}}} /> }}).disabled(true)",
                    glyph(n)
                )
            }
            (true, false) => format!(
                ".icon(html! {{ <Icon glyph={{Glyph::{:?}}} /> }})",
                glyph(n)
            ),
            (false, true) => ".disabled(true)".to_owned(),
            (false, false) => String::new(),
        };
        format!(
            "    TabItem::new(\"tab-{n}\", {:?}){then},",
            label(*text, "Tab", n)
        )
    }));
    if *count > 3 {
        lets.push(format!("    // … {} more", *count - 3));
    }
    lets.push("];".to_owned());
    let code = snippet(
        &lets,
        El::new(if *small { "SubTabs" } else { "Tabs" })
            .expr("tabs", "tabs")
            .expr("value", format!("Some(\"tab-{current}\".into())"))
            .text("label", "Sections")
            .expr("on_change", "on_change"),
    );

    let value = (*count > 0).then(|| AttrValue::from(format!("tab-{current}")));
    html! {
        <Demo {controls} {code}>
            <code class={class::CLASS_CALL}>
                { format!("tabs(small) → \"{}\"", tabs(*small).to_string()) }
            </code>
            if *small {
                <SubTabs tabs={items} {value} label="Sections" {on_change} />
            } else {
                <Tabs tabs={items} {value} label="Sections" {on_change} />
            }
            <span class={class::CONTROL_LABEL}>{ "Tab to it, then the arrow keys, Home and End." }</span>
        </Demo>
    }
}

#[function_component]
pub fn NavListPage() -> Html {
    let count = use_state(|| 3usize);
    let text = use_state(|| Text::Short);
    let selected = use_state(|| 0usize);
    let icons = use_state(|| false);
    let disabled = use_state(|| false);
    let links = use_state(|| false);

    let controls = html! {
        <>
            <Select<usize> label="Entries" options={COUNTS} value={*count} on_change={bind(&count)} />
            <Select<Text> label="Text" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <Select<usize> label="Current" options={SELECTED} value={*selected} on_change={bind(&selected)} />
            <Toggle label="icons" value={*icons} on_change={bind(&icons)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
            <Toggle label="links" value={*links} on_change={bind(&links)} />
        </>
    };

    let items: Vec<NavItem> = (0..*count)
        .map(|n| {
            let mut item = NavItem::new(format!("entry-{n}"), label(*text, "Entry", n))
                .disabled(*disabled && n == 1);
            if *links {
                item = item.href("#nav-list");
            }
            if *icons {
                item = item.icon(html! { <Icon glyph={glyph(n)} /> });
            }
            item
        })
        .collect();
    let on_select = {
        let selected = selected.clone();
        Callback::from(move |value: AttrValue| {
            if let Some(n) = value
                .strip_prefix("entry-")
                .and_then(|n| n.parse::<usize>().ok())
            {
                selected.set(n);
            }
        })
    };

    let mut lets = vec!["let items = vec![".to_owned()];
    lets.extend((0..(*count).min(3)).map(|n| {
        let mut then = String::new();
        if *icons {
            then += &format!(
                ".icon(html! {{ <Icon glyph={{Glyph::{:?}}} /> }})",
                glyph(n)
            );
        }
        if *links {
            then += ".href(\"#nav-list\")";
        }
        if *disabled && n == 1 {
            then += ".disabled(true)";
        }
        format!(
            "    NavItem::new(\"entry-{n}\", {:?}){then},",
            label(*text, "Entry", n)
        )
    }));
    if *count > 3 {
        lets.push(format!("    // … {} more", *count - 3));
    }
    lets.push("];".to_owned());
    let code = snippet(
        &lets,
        El::new("NavList")
            .expr("items", "items")
            .expr(
                "value",
                format!(
                    "Some(\"entry-{}\".into())",
                    (*selected).min(count.saturating_sub(1))
                ),
            )
            .text("label", "Settings")
            .expr("on_select", "on_select"),
    );

    let value =
        (*count > 0).then(|| AttrValue::from(format!("entry-{}", (*selected).min(*count - 1))));
    html! {
        <Demo {controls} {code}>
            <code class={class::CLASS_CALL}>
                { format!("nav_list() → \"{}\"", nav_list().to_string()) }
            </code>
            <NavList {items} {value} label="Settings" {on_select} />
        </Demo>
    }
}
