use yew::prelude::*;

use crate::class;

use sockel::{
    Button, ButtonSize, EmptyState, ErrorBox, Glyph, Icon, Notice, Tone, empty_state, notice,
};

use crate::code::{El, snippet, text_node};
use crate::controls::{Demo, Select, Text, Toggle, bind};

const TONES: &[(&str, Tone)] = &[
    ("Neutral", Tone::Neutral),
    ("Accent", Tone::Accent),
    ("Success", Tone::Success),
    ("Warning", Tone::Warning),
    ("Danger", Tone::Danger),
];

fn shown(call: &str, classes: &Classes) -> Html {
    html! { <code class={class::CLASS_CALL}>{ format!("{call} → \"{}\"", classes.to_string()) }</code> }
}

#[function_component]
pub fn NoticePage() -> Html {
    let tone = use_state(|| Tone::Accent);
    let text = use_state(|| Text::Short);
    let icon = use_state(|| true);
    let action = use_state(|| true);
    let close = use_state(|| true);
    let open = use_state(|| true);
    let closes = use_state(|| 0);

    let controls = html! {
        <>
            <Select<Tone> label="tone" options={TONES} value={*tone} on_change={bind(&tone)} />
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

    let words = text.of("Your changes were saved");
    let code = snippet(
        &[],
        El::new("Notice")
            .expr_if(*tone != Tone::Accent, "tone", format!("Tone::{:?}", *tone))
            .expr_if(*icon, "icon", "html! { <Icon glyph={Glyph::Check} /> }")
            .expr_if(
                *action,
                "action",
                "html! { <Button size={ButtonSize::Small}>{ \"Undo\" }</Button> }",
            )
            .expr_if(*close, "onclose", "on_close")
            .line_if(!words.is_empty(), text_node(&words)),
    );

    html! {
        <Demo {controls} {code}>
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

const CAUSE_LIST: &[&str] = &[
    "error sending request for url (http://localhost/api/items)",
    "connection refused (os error 111)",
];

#[function_component]
pub fn ErrorBoxPage() -> Html {
    let text = use_state(|| Text::Short);
    let titled = use_state(|| true);
    let cause = use_state(|| Text::Short);
    let causes = use_state(|| false);
    let retry = use_state(|| true);
    let retrying = use_state(|| false);
    let retries = use_state(|| 0);
    let icon = use_state(|| false);
    let dismiss = use_state(|| false);
    let dismissed = use_state(|| 0);

    let controls = html! {
        <>
            <Select<Text> label="Message" options={Text::OPTIONS} value={*text} on_change={bind(&text)} />
            <Toggle label="title" value={*titled} on_change={bind(&titled)} />
            <Select<Text> label="cause" options={CAUSES} value={*cause} on_change={bind(&cause)} />
            <Toggle label="causes" value={*causes} on_change={bind(&causes)} />
            <Toggle label="onretry" value={*retry} on_change={bind(&retry)} />
            <Toggle label="retrying" value={*retrying} on_change={bind(&retrying)} />
            <Toggle label="icon" value={*icon} on_change={bind(&icon)} />
            <Toggle label="ondismiss" value={*dismiss} on_change={bind(&dismiss)} />
        </>
    };

    let ondismiss = {
        let dismissed = dismissed.clone();
        Callback::from(move |_: MouseEvent| dismissed.set(*dismissed + 1))
    };
    let onretry = {
        let retries = retries.clone();
        Callback::from(move |_: MouseEvent| retries.set(*retries + 1))
    };
    let cause_text = (*cause != Text::Empty)
        .then(|| AttrValue::from(cause.of("TypeError: failed to fetch /api/items")));

    let cause_list = if *causes {
        CAUSE_LIST.iter().map(|c| AttrValue::Static(c)).collect()
    } else {
        Vec::new()
    };

    let words = text.of("The list is not available right now");
    let code = snippet(
        &[],
        El::new("ErrorBox")
            .text_if(*titled, "title", "Could not load the items")
            .text_if(
                *cause != Text::Empty,
                "cause",
                &cause.of("TypeError: failed to fetch /api/items"),
            )
            .expr_if(
                *causes,
                "causes",
                format!(
                    "vec![{}]",
                    CAUSE_LIST
                        .iter()
                        .map(|c| format!("{c:?}.into()"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
            .expr_if(*retry, "onretry", "on_retry")
            .flag("retrying", *retrying)
            .expr_if(*icon, "icon", "html! { \"!\" }")
            .expr_if(*dismiss, "ondismiss", "on_dismiss")
            .line_if(!words.is_empty(), text_node(&words)),
    );

    html! {
        <Demo {controls} {code}>
            <ErrorBox title={titled.then_some(AttrValue::Static("Could not load the items"))}
                cause={cause_text} causes={cause_list} onretry={retry.then_some(onretry)} retrying={*retrying}
                icon={icon.then(|| html! { "!" })}
                ondismiss={dismiss.then_some(ondismiss)}>
                { text.of("The list is not available right now") }
            </ErrorBox>
            <span class={class::CONTROL_LABEL}>{ format!("Retried {} times, dismissed {} times", *retries, *dismissed) }</span>
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
    let bare_icon = use_state(|| false);

    let controls = html! {
        <>
            <Select<Text> label="title" options={Text::OPTIONS} value={*title} on_change={bind(&title)} />
            <Select<Text> label="hint" options={Text::OPTIONS} value={*hint} on_change={bind(&hint)} />
            <Toggle label="icon" value={*icon} on_change={bind(&icon)} />
            <Toggle label="Actions" value={*actions} on_change={bind(&actions)} />
            <Toggle label="compact" value={*compact} on_change={bind(&compact)} />
            <Toggle label="bare_icon" value={*bare_icon} on_change={bind(&bare_icon)} />
        </>
    };

    let code = snippet(
        &[],
        El::new("EmptyState")
            .text("title", &title.of("No items yet"))
            .flag("compact", *compact)
            .expr_if(*icon, "icon", "html! { <Icon glyph={Glyph::Search} /> }")
            .flag("bare_icon", *icon && *bare_icon)
            .expr_if(
                *hint != Text::Empty,
                "hint",
                format!("html! {{ {:?} }}", hint.of("Add one to get started")),
            )
            .line_if(
                *actions,
                "<Button size={ButtonSize::Small}>{ \"Add item\" }</Button>",
            ),
    );

    html! {
        <Demo {controls} {code}>
            { shown("empty_state(compact)", &empty_state(*compact)) }
            <EmptyState title={title.of("No items yet")} compact={*compact} bare_icon={*bare_icon}
                icon={icon.then(|| html! { <Icon glyph={Glyph::Search} /> })}
                hint={(*hint != Text::Empty).then(|| html! { { hint.of("Add one to get started") } })}>
                if *actions { <Button size={ButtonSize::Small}>{ "Add item" }</Button> }
            </EmptyState>
            <div class={class::SAMPLE}>{ "A list row, for comparison" }</div>
        </Demo>
    }
}
