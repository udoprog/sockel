use yew::prelude::*;

use sockel::{AppShell, BottomNav, BottomNavItem, Glyph, Icon, PickRow, SplitView};

use crate::class;
use crate::code::{El, snippet};
use crate::controls::{Demo, Select, Text, Toggle, bind};

const SECTIONS: [(&str, &str, Glyph); 4] = [
    ("boards", "Boards", Glyph::Check),
    ("sessions", "Sessions", Glyph::Ellipsis),
    ("inbox", "Inbox", Glyph::Search),
    ("settings", "Settings", Glyph::Plus),
];

const CURRENT: &[(&str, &str)] = &[
    ("Boards", "boards"),
    ("Sessions", "sessions"),
    ("Inbox", "inbox"),
    ("Settings", "settings"),
];

const WAITING: &[(&str, u32)] = &[("3", 3), ("0 (hidden, box kept)", 0), ("120 (99+)", 120)];

fn items(text: Text, icons: bool, waiting: u32) -> Vec<BottomNavItem> {
    SECTIONS
        .iter()
        .map(|&(value, label, glyph)| {
            let mut item = BottomNavItem::new(value, text.of(label));
            if icons {
                item = item.icon(html! { <Icon glyph={glyph} /> });
            }
            if value == "inbox" {
                item = item.count(waiting);
            }
            item
        })
        .collect()
}

#[function_component]
pub fn AppShellPage() -> Html {
    let current = use_state(|| "boards");
    let waiting = use_state(|| 3u32);
    let icons = use_state(|| true);
    let text = use_state(|| Text::Short);

    let controls = html! {
        <>
            <Select<&'static str> label="current" options={CURRENT} value={*current} on_change={bind(&current)} />
            <Select<u32> label="inbox count" options={WAITING} value={*waiting} on_change={bind(&waiting)} />
            <Toggle label="icons" value={*icons} on_change={bind(&icons)} />
            <Select<Text> label="Text" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
        </>
    };

    let on_select = {
        let current = current.clone();
        Callback::from(move |value: AttrValue| {
            if let Some(&(_, v, _)) = SECTIONS.iter().find(|(v, ..)| *v == value.as_str()) {
                current.set(v);
            }
        })
    };

    let code = snippet(
        &[
            "let items = vec![".to_owned(),
            "    BottomNavItem::new(\"boards\", \"Boards\").icon(html! { <Icon glyph={Glyph::Check} /> }),".to_owned(),
            format!("    BottomNavItem::new(\"inbox\", \"Inbox\").count({}),", *waiting),
            "];".to_owned(),
        ],
        El::new("AppShell")
            .expr("start", "html! { <strong>{ \"Title\" }</strong> }")
            .expr("end", "html! { <Button>{ \"Menu\" }</Button> }")
            .expr(
                "nav",
                format!(
                    "html! {{ <BottomNav label=\"Sections\" {{items}} value=\"{}\" {{on_select}} /> }}",
                    *current
                ),
            )
            .line("{ page }"),
    );

    let title = SECTIONS
        .iter()
        .find(|(v, ..)| *v == *current)
        .map_or("", |(_, label, _)| *label);

    html! {
        <Demo {controls} {code}>
            <AppShell class={class::SHELL_BOX}
                start={html! { <strong class={sockel::class::TRUNCATE}>{ text.of(title) }</strong> }}
                end={html! { <button type="button" class={class::BUTTON}>{ "⋯" }</button> }}
                nav={html! {
                    <BottomNav label="Sections" items={items(*text, *icons, *waiting)}
                        value={AttrValue::from(*current)} {on_select} />
                }}>
                <div class={class::SAMPLE}>{ text.of(&format!("{title} page")) }</div>
                <div class={classes!(class::SAMPLE, class::TALL)}>{ text.of("It scrolls under the bar") }</div>
                <div class={classes!(class::SAMPLE, class::TALL)}>{ text.of("and over the bottom nav") }</div>
            </AppShell>
        </Demo>
    }
}

const OPENED: &[(&str, Option<usize>)] =
    &[("none", None), ("Item 1", Some(0)), ("Item 3", Some(2))];

const SHARES: &[(&str, f64)] = &[("0.4", 0.4), ("0.3", 0.3), ("0.6", 0.6)];

#[function_component]
pub fn SplitViewPage() -> Html {
    let open = use_state(|| Some(0usize));
    let at = use_state(|| 0.4);
    let settled = use_state(|| None::<f64>);
    let back = use_state(|| true);
    let text = use_state(|| Text::Short);

    let controls = html! {
        <>
            <Select<Option<usize>> label="open" options={OPENED} value={*open} on_change={bind(&open)} />
            <Select<f64> label="at" options={SHARES} value={*at} on_change={bind(&at)} />
            <Toggle label="back button" value={*back} on_change={bind(&back)} />
            <Select<Text> label="Text" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <span class={classes!(class::CONTROL_LABEL, sockel::class::NUMERIC)}>
                { settled.map_or("not moved".to_owned(), |s| format!("settled at {s:.2}")) }
            </span>
        </>
    };

    let list = html! {
        <div>
            { for (0..6).map(|n| {
                let current = *open == Some(n);
                let open = open.clone();
                let onclick = Callback::from(move |_| open.set(Some(n)));
                html! {
                    <PickRow name={html! { { text.of(&format!("Item {}", n + 1)) } }}
                        {current} {onclick} />
                }
            }) }
        </div>
    };
    let detail = open.map(|n| {
        let close = {
            let open = open.clone();
            Callback::from(move |_| open.set(None))
        };
        html! {
            <div class={class::SAMPLE}>
                if *back {
                    <button type="button" class={classes!(class::BUTTON, sockel::class::SPLIT_VIEW_BACK)}
                        onclick={close}>{ "Back" }</button>
                }
                <p>{ text.of(&format!("Item {} in detail", n + 1)) }</p>
            </div>
        }
    });

    let code = snippet(
        &["let open = use_state(|| Some(0));".to_owned()],
        El::new("SplitView")
            .expr("list", "html! { <div>{ rows }</div> }")
            .expr("detail", "open.map(|n| html! { <Detail {n} /> })")
            .expr("empty", "html! { <EmptyState title=\"Nothing open\" /> }")
            .expr_if(*at != 0.4, "at", format!("{:.1}", *at))
            .expr("on_settle", "keep"),
    );

    html! {
        <Demo {controls} {code}>
            <div class={class::SPLIT_BOX}>
                <SplitView {list} {detail} at={*at}
                    on_settle={Callback::from({ let settled = settled.clone(); move |s| settled.set(Some(s)) })}
                    empty={html! { <div class={class::SAMPLE}>{ text.of("Nothing open") }</div> }} />
            </div>
        </Demo>
    }
}
