use yew::prelude::*;

use sockel::{Pager, PagerSummary, page_count, pager, pager_button, pager_slots};

use crate::class;
use crate::code::{El, snippet};
use crate::controls::{Demo, Select, Toggle, bind};

const PAGES: &[(&str, usize)] = &[
    ("20", 20),
    ("1", 1),
    ("5", 5),
    ("8", 8),
    ("9", 9),
    ("1000", 1000),
];
const SLOTS: &[(&str, usize)] = &[("7", 7), ("5", 5), ("9", 9)];

fn shown(call: &str, classes: &Classes) -> Html {
    html! { <code class={class::CLASS_CALL}>{ format!("{call} → \"{}\"", classes.to_string()) }</code> }
}

#[function_component]
pub fn PagerPage() -> Html {
    let pages = use_state(|| 20usize);
    let slots = use_state(|| 7usize);
    let edges = use_state(|| true);
    let phone_steps = use_state(|| true);
    let titles = use_state(|| false);
    let summary = use_state(|| false);
    let disabled = use_state(|| false);
    let busy = use_state(|| false);
    let page = use_state(|| 0usize);

    let controls = html! {
        <>
            <Select<usize> label="Pages" options={PAGES} value={*pages} on_change={bind(&pages)} />
            <Select<usize> label="Slots" options={SLOTS} value={*slots} on_change={bind(&slots)} />
            <Toggle label="edges" value={*edges} on_change={bind(&edges)} />
            <Toggle label="phone_steps" value={*phone_steps} on_change={bind(&phone_steps)} />
            <Toggle label="titles" value={*titles} on_change={bind(&titles)} />
            <Toggle label="summary" value={*summary} on_change={bind(&summary)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
            <Toggle label="busy" value={*busy} on_change={bind(&busy)} />
        </>
    };
    let code = snippet(
        &[],
        El::new("Pager")
            .expr("page", "page")
            .expr("pages", pages.to_string())
            .expr("on_page", "on_page.clone()")
            .expr_if(*slots != 7, "slots", slots.to_string())
            .expr_if(!*edges, "edges", "false")
            .expr_if(!*phone_steps, "phone_steps", "false")
            .flag("titles", *titles)
            .expr_if(
                *summary,
                "summary",
                "Some(PagerSummary::new(shown, position))",
            )
            .flag("disabled", *disabled)
            .flag("busy", *busy),
    );
    let current = (*page).min(*pages - 1);
    let counts = summary.then(|| {
        PagerSummary::new(
            format!(
                "{}–{} of {}",
                current * 50 + 1,
                current * 50 + 50,
                50 * *pages
            ),
            format!("Page {} of {}", current + 1, *pages),
        )
    });
    let layout = pager_slots(*pages, current, *slots)
        .iter()
        .map(|slot| format!("{slot:?}"))
        .collect::<Vec<_>>()
        .join(" ");

    html! {
        <Demo {controls} {code}>
            <Pager page={current} pages={*pages} slots={*slots} edges={*edges} phone_steps={*phone_steps} titles={*titles}
                disabled={*disabled} busy={*busy} summary={counts} on_page={bind(&page)} />
            <span class={class::CONTROL_LABEL}>{ format!("Page {} of {}", current + 1, *pages) }</span>
            <span class={class::CONTROL_LABEL}>{ format!("Items 1 to 50 of {} fill {} pages", 50 * *pages, page_count(50 * *pages, 50)) }</span>
            <code class={class::CLASS_CALL}>{ format!("pager_slots({}, {current}, {}) → {layout}", *pages, *slots) }</code>
            { shown("pager(disabled, busy)", &pager(*disabled, *busy)) }
            { shown("pager_button(edge)", &pager_button(true)) }
        </Demo>
    }
}
