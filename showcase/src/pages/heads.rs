use yew::prelude::*;

use crate::class;

use plinth::{Head, HeadLevel, HeadProps, PageHead, SectionHead};

use crate::code::{El, each, snippet, text_node};
use crate::controls::{COUNTS, Demo, Select, Text, Toggle, bind, button};

const LEVELS: &[(&str, Option<HeadLevel>)] = &[
    ("The head's own", None),
    ("h1", Some(HeadLevel::H1)),
    ("h2", Some(HeadLevel::H2)),
    ("h3", Some(HeadLevel::H3)),
    ("h4", Some(HeadLevel::H4)),
];

const COUNTS_UP_TO_3: &[(&str, usize)] = &[("0", 0), ("1", 1), ("2", 2), ("3", 3)];

const TITLE_CLASSES: &[(&str, &str)] = &[("None", ""), ("muted", class::MUTED)];

#[derive(Properties, PartialEq)]
pub struct HeadPageProps {
    pub kind: Head,
}

/// PageHead's or SectionHead's page: both take [`HeadProps`].
#[function_component]
pub fn HeadPage(props: &HeadPageProps) -> Html {
    let title = use_state(|| Text::Short);
    let title_span = use_state(|| false);
    let icon = use_state(|| false);
    let subtitle = use_state(|| false);
    let crumbs = use_state(|| 0);
    let label = use_state(|| false);
    let list = use_state(|| false);
    let nested = use_state(|| false);
    let help = use_state(|| true);
    let back = use_state(|| false);
    let meta = use_state(|| false);
    let compact = use_state(|| false);
    let one_row = use_state(|| false);
    let actions = use_state(|| 1);
    let labels = use_state(|| Text::Short);
    let level = use_state(|| None);
    let suffix = use_state(|| false);
    let title_class = use_state(|| "");
    let testids = use_state(|| false);

    let controls = html! {
        <>
            <Select<Text> label="title" options={Text::OPTIONS} value={*title} on_change={bind(&title)} />
            <Select<usize> label="Actions" options={COUNTS} value={*actions} on_change={bind(&actions)} />
            <Select<Text> label="Action labels" options={Text::OPTIONS} value={*labels} on_change={bind(&labels)} />
            <Select<Option<HeadLevel>> label="level" options={LEVELS} value={*level} on_change={bind(&level)} />
            <Select<&'static str> label="title_class" options={TITLE_CLASSES} value={*title_class} on_change={bind(&title_class)} />
            <Toggle label="title with a span" value={*title_span} on_change={bind(&title_span)} />
            <Toggle label="icon" value={*icon} on_change={bind(&icon)} />
            <Toggle label="subtitle" value={*subtitle} on_change={bind(&subtitle)} />
            <Select<usize> label="crumbs" options={COUNTS_UP_TO_3} value={*crumbs} on_change={bind(&crumbs)} />
            if props.kind == Head::Section {
                <Toggle label="class label" value={*label} on_change={bind(&label)} />
                <Toggle label="class list" value={*list} on_change={bind(&list)} />
                <Toggle label="class nested" value={*nested} on_change={bind(&nested)} />
            }
            <Toggle label="help" value={*help} on_change={bind(&help)} />
            <Toggle label="suffix" value={*suffix} on_change={bind(&suffix)} />
            <Toggle label="Test ids" value={*testids} on_change={bind(&testids)} />
            <Toggle label="back" value={*back} on_change={bind(&back)} />
            <Toggle label="meta" value={*meta} on_change={bind(&meta)} />
            <Toggle label="class compact" value={*compact} on_change={bind(&compact)} />
            <Toggle label="class one_row" value={*one_row} on_change={bind(&one_row)} />
        </>
    };

    let children: Html = (1..=*actions)
        .map(|n| button(labels.of(&format!("Action {n}")), n == *actions, false))
        .collect();

    let head = HeadProps {
        title: {
            let text = title.of("The title");
            if *title_span {
                html! { <>{ text }{ " " }<span class={class::MUTED}>{ "(span)" }</span></> }
            } else {
                html! { { text } }
            }
        },
        icon: icon.then(|| html! { { "\u{2605}" } }),
        subtitle: subtitle.then(|| html! { { "One muted line of facts under the row" } }),
        crumbs: ["Home", "Section", "Parent"]
            .into_iter()
            .take(*crumbs)
            .map(|name| html! { <a href="#" onclick={Callback::from(|e: MouseEvent| e.prevent_default())}>{ name }</a> })
            .collect(),
        crumbs_label: (*crumbs > 0).then(|| AttrValue::from("Trail")),
        help: help.then(|| {
            html! {
                <button type="button" class={class::HELP_TIP} aria-label="Help" title="What this is">{ "?" }</button>
            }
        }),
        level: *level,
        suffix: suffix.then(|| html! { <span class={class::CHIP}>{ "12" }</span> }),
        help_testid: testids.then(|| AttrValue::from("head-help")),
        title_class: classes!((!title_class.is_empty()).then_some(*title_class)),
        title_testid: testids.then(|| AttrValue::from("head-title")),
        head_testid: testids.then(|| AttrValue::from("head")),
        testid: None,
        class: classes!(
            compact.then_some(plinth::class::COMPACT),
            one_row.then_some(plinth::class::ONE_ROW),
            label.then_some(plinth::class::LABEL),
            list.then_some(plinth::class::LIST),
            nested.then_some(plinth::class::NESTED),
        ),
        back: back.then(|| {
            html! {
                <a href="#" class={plinth::class::PAGE_HEAD_BACK}
                    onclick={Callback::from(|e: MouseEvent| e.prevent_default())}>
                    <span aria-hidden="true">{ "←" }</span>
                    <span class={classes!(class::LABEL_TEXT, plinth::class::PAGE_HEAD_BACK_LABEL)}>{ "Back" }</span>
                </a>
            }
        }),
        back_label: back.then(|| AttrValue::from("Breadcrumb")),
        meta: meta.then(|| {
            html! {
                <div class={plinth::class::PAGE_HEAD_META}>
                    <span class={class::CHIP}>{ "keyword" }</span>
                    <span class={class::CHIP}>{ "another" }</span>
                </div>
            }
        }),
        children,
    };

    let title_code = {
        let words = text_node(&title.of("The title"));
        if *title_span {
            format!("html! {{ <>{words}{{ \" \" }}<span>{{ \"(span)\" }}</span></> }}")
        } else {
            format!("html! {{ {:?} }}", title.of("The title"))
        }
    };
    let modifiers: Vec<&str> = [
        (*compact, "plinth::class::COMPACT"),
        (*one_row, "plinth::class::ONE_ROW"),
        (*label, "plinth::class::LABEL"),
        (*list, "plinth::class::LIST"),
        (*nested, "plinth::class::NESTED"),
    ]
    .into_iter()
    .filter_map(|(on, name)| on.then_some(name))
    .collect();
    let crumb_names = ["home", "section", "parent"];
    let code = snippet(
        &[],
        El::new(match props.kind {
            Head::Page => "PageHead",
            Head::Section => "SectionHead",
        })
        .expr("title", title_code)
        .expr_if(*icon, "icon", "html! { \"\u{2605}\" }")
        .expr_if(
            *subtitle,
            "subtitle",
            "html! { \"One muted line of facts under the row\" }",
        )
        .expr_if(
            *crumbs > 0,
            "crumbs",
            format!("vec![{}]", crumb_names[..*crumbs].join(", ")),
        )
        .text_if(*crumbs > 0, "crumbs_label", "Trail")
        .expr_if(*help, "help", "html! { <HelpTip /> }")
        .expr_if(
            level.is_some(),
            "level",
            format!("HeadLevel::{:?}", level.unwrap_or(HeadLevel::H1)),
        )
        .expr_if(
            *suffix,
            "suffix",
            "html! { <span class=\"chip\">{ \"12\" }</span> }",
        )
        .text_if(*testids, "help_testid", "head-help")
        .text_if(!title_class.is_empty(), "title_class", "muted")
        .text_if(*testids, "title_testid", "head-title")
        .text_if(*testids, "head_testid", "head")
        .expr_if(
            !modifiers.is_empty(),
            "class",
            format!("classes!({})", modifiers.join(", ")),
        )
        .expr_if(*back, "back", "html! { <BackLink /> }")
        .text_if(*back, "back_label", "Breadcrumb")
        .expr_if(
            *meta,
            "meta",
            "html! { <div class={plinth::class::PAGE_HEAD_META}>{ keywords }</div> }",
        )
        .line_if(
            *actions > 0,
            each("actions", "action_button", *actions, "button"),
        ),
    );

    let shown = match props.kind {
        Head::Page => html! { <PageHead ..head /> },
        Head::Section => html! { <SectionHead ..head /> },
    };

    html! { <Demo {controls} {code}>{ shown }</Demo> }
}
