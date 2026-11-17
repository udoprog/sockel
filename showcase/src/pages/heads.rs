use yew::prelude::*;

use plinth::{Head, HeadProps, PageHead, SectionHead};

use crate::controls::{COUNTS, Demo, Select, Text, Toggle, bind, button};

#[derive(Properties, PartialEq)]
pub struct HeadPageProps {
    pub kind: Head,
}

/// PageHead's or SectionHead's page: both take [`HeadProps`].
#[function_component]
pub fn HeadPage(props: &HeadPageProps) -> Html {
    let title = use_state(|| Text::Short);
    let help = use_state(|| true);
    let back = use_state(|| false);
    let meta = use_state(|| false);
    let compact = use_state(|| false);
    let actions = use_state(|| 1);
    let labels = use_state(|| Text::Short);

    let controls = html! {
        <>
            <Select<Text> label="title" options={Text::OPTIONS} value={*title} on_change={bind(&title)} />
            <Select<usize> label="Actions" options={COUNTS} value={*actions} on_change={bind(&actions)} />
            <Select<Text> label="Action labels" options={Text::OPTIONS} value={*labels} on_change={bind(&labels)} />
            <Toggle label="help" value={*help} on_change={bind(&help)} />
            <Toggle label="back" value={*back} on_change={bind(&back)} />
            <Toggle label="meta" value={*meta} on_change={bind(&meta)} />
            <Toggle label="class compact" value={*compact} on_change={bind(&compact)} />
        </>
    };

    let children: Html = (1..=*actions)
        .map(|n| button(labels.of(&format!("Action {n}")), n == *actions, false))
        .collect();

    let head = HeadProps {
        title: title.of("The title").into(),
        help: help.then(|| {
            html! {
                <button type="button" class="help-tip" aria-label="Help" title="What this is">{ "?" }</button>
            }
        }),
        head_testid: None,
        testid: None,
        class: classes!(compact.then_some("compact")),
        back: back.then(|| {
            html! {
                <a href="#" class="crumb-back page-head-back"
                    onclick={Callback::from(|e: MouseEvent| e.prevent_default())}>
                    <span aria-hidden="true">{ "←" }</span>
                    <span class="label-text page-head-back-label">{ "Back" }</span>
                </a>
            }
        }),
        back_label: back.then(|| AttrValue::from("Breadcrumb")),
        meta: meta.then(|| {
            html! {
                <div class="page-head-meta">
                    <span class="chip">{ "keyword" }</span>
                    <span class="chip">{ "another" }</span>
                </div>
            }
        }),
        children,
    };

    let shown = match props.kind {
        Head::Page => html! { <PageHead ..head /> },
        Head::Section => html! { <SectionHead ..head /> },
    };

    html! { <Demo {controls}>{ shown }</Demo> }
}
