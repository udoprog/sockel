use yew::prelude::*;

use crate::class;

use plinth::{
    Button, ButtonSize, EmptyState, ErrorBox, Glyph, Icon, Notice, NoticeTone, empty_state, notice,
};

use crate::controls::{Demo, Select, Text, Toggle, bind};

const TONES: &[(&str, NoticeTone)] = &[
    ("Info", NoticeTone::Info),
    ("Success", NoticeTone::Success),
    ("Warning", NoticeTone::Warning),
    ("Error", NoticeTone::Error),
];

fn shown(call: &str, classes: &Classes) -> Html {
    html! { <code class={class::CLASS_CALL}>{ format!("{call} → \"{}\"", classes.to_string()) }</code> }
}

#[function_component]
pub fn NoticePage() -> Html {
    let tone = use_state(NoticeTone::default);
    let text = use_state(|| Text::Short);
    let icon = use_state(|| true);
    let action = use_state(|| true);
    let close = use_state(|| true);
    let open = use_state(|| true);
    let closes = use_state(|| 0);

    let controls = html! {
        <>
            <Select<NoticeTone> label="tone" options={TONES} value={*tone} on_change={bind(&tone)} />
            <Select<Text> label="Text" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <Toggle label="icon" value={*icon} on_change={bind(&icon)} />
            <Toggle label="action" value={*action} on_change={bind(&action)} />
            <Toggle label="onclose" value={*close} on_change={bind(&close)} />
            <Toggle label="Shown" value={*open} on_change={bind(&open)} />
        </>
    };

    let onclose = {
        let open = open.clone();
        let closes = closes.clone();
        Callback::from(move |_: MouseEvent| {
            closes.set(*closes + 1);
            open.set(false);
        })
    };

    html! {
        <Demo {controls}>
            { shown("notice(tone)", &notice(*tone)) }
            if *open {
                <Notice tone={*tone}
                    icon={icon.then(|| html! { <Icon glyph={Glyph::Check} /> })}
                    action={action.then(|| html! { <Button size={ButtonSize::Small}>{ "Undo" }</Button> })}
                    onclose={close.then_some(onclose)}>
                    { text.of("Your changes were saved") }
                </Notice>
            }
            <span class={class::CONTROL_LABEL}>
                { format!("Closed {} times; closing collapses the notice's space, by design (tick Shown to bring it back).", *closes) }
            </span>
        </Demo>
    }
}

const CAUSES: &[(&str, Text)] = &[
    ("Short", Text::Short),
    ("Long", Text::Long),
    ("Long, unbroken", Text::Unbroken),
    ("None", Text::Empty),
];

#[function_component]
pub fn ErrorBoxPage() -> Html {
    let text = use_state(|| Text::Short);
    let titled = use_state(|| true);
    let cause = use_state(|| Text::Short);
    let retry = use_state(|| true);
    let retrying = use_state(|| false);
    let retries = use_state(|| 0);

    let controls = html! {
        <>
            <Select<Text> label="Message" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <Toggle label="title" value={*titled} on_change={bind(&titled)} />
            <Select<Text> label="cause" options={CAUSES} value={*cause} on_change={bind(&cause)} />
            <Toggle label="onretry" value={*retry} on_change={bind(&retry)} />
            <Toggle label="retrying" value={*retrying} on_change={bind(&retrying)} />
        </>
    };

    let onretry = {
        let retries = retries.clone();
        Callback::from(move |_: MouseEvent| retries.set(*retries + 1))
    };
    let cause_text = (*cause != Text::Empty)
        .then(|| AttrValue::from(cause.of("TypeError: failed to fetch /api/items")));

    html! {
        <Demo {controls}>
            <ErrorBox title={titled.then_some(AttrValue::Static("Could not load the items"))}
                cause={cause_text} onretry={retry.then_some(onretry)} retrying={*retrying}>
                { text.of("The list is not available right now") }
            </ErrorBox>
            <span class={class::CONTROL_LABEL}>{ format!("Retried {} times", *retries) }</span>
        </Demo>
    }
}

#[function_component]
pub fn EmptyStatePage() -> Html {
    let title = use_state(|| Text::Short);
    let hint = use_state(|| Text::Short);
    let icon = use_state(|| true);
    let actions = use_state(|| true);
    let compact = use_state(|| true);

    let controls = html! {
        <>
            <Select<Text> label="title" options={Text::OPTIONS} value={*title} on_change={bind(&title)} />
            <Select<Text> label="hint" options={Text::OPTIONS} value={*hint} on_change={bind(&hint)} />
            <Toggle label="icon" value={*icon} on_change={bind(&icon)} />
            <Toggle label="Actions" value={*actions} on_change={bind(&actions)} />
            <Toggle label="compact" value={*compact} on_change={bind(&compact)} />
        </>
    };

    html! {
        <Demo {controls}>
            { shown("empty_state(compact)", &empty_state(*compact)) }
            <EmptyState title={title.of("No items yet")} compact={*compact}
                icon={icon.then(|| html! { <Icon glyph={Glyph::Search} /> })}
                hint={(*hint != Text::Empty).then(|| html! { { hint.of("Add one to get started") } })}>
                if *actions { <Button size={ButtonSize::Small}>{ "Add item" }</Button> }
            </EmptyState>
            <div class={class::SAMPLE}>{ "A list row, for comparison" }</div>
        </Demo>
    }
}
