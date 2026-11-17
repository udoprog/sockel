use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use yew::prelude::*;

use crate::class;

use plinth::{
    Button, ButtonSize, ButtonVariant, Cluster, Dialog, DialogDismiss, DialogSize, Dismiss, Glyph,
    Icon, Popover, Stack, button,
};

use crate::code::{El, snippet};
use crate::controls::{Demo, Select, Text, Toggle, bind};

const SIZES: &[(&str, DialogSize)] = &[
    ("Default", DialogSize::Default),
    ("Wide", DialogSize::Wide),
    ("Tall", DialogSize::Tall),
    ("WideTall", DialogSize::WideTall),
];

const DISMISSES: &[(&str, DialogDismiss)] = &[
    ("Any", DialogDismiss::Any),
    ("NoOutside", DialogDismiss::NoOutside),
    ("Actions", DialogDismiss::Actions),
];

/// What the dialog's body holds.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Body {
    Short,
    /// Enough to scroll.
    Long,
    /// Short, then long once it has "loaded".
    Loading,
    Empty,
}

const BODIES: &[(&str, Body)] = &[
    ("Short", Body::Short),
    ("Long (scrolls)", Body::Long),
    ("Loading, then long", Body::Loading),
    ("Empty", Body::Empty),
];

#[derive(Properties, PartialEq, Clone)]
struct Setup {
    size: DialogSize,
    dismiss: DialogDismiss,
    title: Text,
    body: Body,
    icon: bool,
    head: bool,
    flush: bool,
    close_regular: bool,
    sheet: bool,
    actions: bool,
    start: bool,
    busy: bool,
}

/// Calls `f` after `ms` milliseconds.
fn after(ms: i32, f: impl FnOnce() + 'static) {
    if let Some(window) = web_sys::window() {
        let f = Closure::once_into_js(f);
        _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(f.unchecked_ref(), ms);
    }
}

#[derive(Properties, PartialEq)]
struct ContentProps {
    body: Body,
}

/// The body: some paragraphs, a popover and a dialog opened from it.
#[function_component]
fn Content(props: &ContentProps) -> Html {
    let loaded = use_state(|| false);
    {
        let loaded = loaded.clone();
        use_effect_with(props.body, move |body| {
            loaded.set(false);
            if *body == Body::Loading {
                after(600, move || loaded.set(true));
            }
        });
    }
    let more = use_node_ref();
    let popover = use_state(|| false);
    let nested = use_state(|| false);

    let paragraphs = match props.body {
        Body::Empty => 0,
        Body::Short => 1,
        Body::Loading if !*loaded => 1,
        Body::Long | Body::Loading => 12,
    };
    let open_popover = {
        let popover = popover.clone();
        Callback::from(move |_: MouseEvent| popover.set(!*popover))
    };
    let close_popover = {
        let popover = popover.clone();
        Callback::from(move |_: Dismiss| popover.set(false))
    };
    let open_nested = {
        let nested = nested.clone();
        Callback::from(move |_: MouseEvent| nested.set(true))
    };
    let close_nested = {
        let nested = nested.clone();
        Callback::from(move |_| nested.set(false))
    };
    let done = close_nested.reform(|_: MouseEvent| Dismiss::Close);

    html! {
        <Stack>
            { for (1..=paragraphs).map(|n| html! {
                <p class={class::PROSE}>{ format!("Paragraph {n}: what the dialog is about, long enough to wrap onto a second line in the default width.") }</p>
            }) }
            <Cluster>
                <button type="button" ref={more.clone()} aria-expanded={popover.to_string()}
                    class={button(ButtonVariant::Secondary, ButtonSize::Small, false, false)}
                    onclick={open_popover}>
                    <span class={plinth::class::BUTTON_LABEL}>{ "A popover" }</span>
                </button>
                <Button size={ButtonSize::Small} onclick={open_nested}>{ "Another dialog" }</Button>
            </Cluster>
            <Popover anchor={more} open={*popover} on_dismiss={close_popover} title="More">
                <p class={class::PROSE}>{ "Escape closes this first, then the dialog." }</p>
            </Popover>
            <Dialog open={*nested} on_dismiss={close_nested} title="Over the first"
                actions={html! { <Button variant={ButtonVariant::Primary} onclick={done}>{ "Done" }</Button> }}>
                <p class={class::PROSE}>{ "The dialog beneath is dimmed and out of reach until this one closes." }</p>
            </Dialog>
        </Stack>
    }
}

#[derive(Properties, PartialEq)]
struct DialogDemoProps {
    setup: Setup,
    open: bool,
    on_open: Callback<bool>,
}

/// One dialog, open as `open` says.
#[function_component]
fn DialogOf(props: &DialogDemoProps) -> Html {
    let s = &props.setup;
    let close = props.on_open.reform(|_| false);
    let cancel = props.on_open.reform(|_: MouseEvent| false);
    let actions = s.actions.then(|| {
        html! {
            <>
                <Button variant={ButtonVariant::Ghost} disabled={s.busy} onclick={cancel.clone()}>{ "Cancel" }</Button>
                <Button variant={ButtonVariant::Primary} busy={s.busy} onclick={cancel.clone()}>{ "Save" }</Button>
            </>
        }
    });
    let head = s.head.then(|| {
        html! { <Button variant={ButtonVariant::Quiet} size={ButtonSize::Small}
        icon={html! { <Icon glyph={Glyph::Ellipsis} /> }} label="More" /> }
    });
    let start = s.start.then(|| {
        html! { <Button variant={ButtonVariant::Danger} disabled={s.busy} onclick={cancel.clone()}>{ "Delete" }</Button> }
    });

    html! {
        <Dialog open={props.open} on_dismiss={close} title={s.title.of("Rename the list")}
            icon={s.icon.then(|| html! { <Icon glyph={Glyph::Ellipsis} /> })}
            size={s.size} dismiss={s.dismiss} flush={s.flush} sheet={s.sheet}
            head_actions={head.unwrap_or_default()}
            close_size={if s.close_regular { ButtonSize::Regular } else { ButtonSize::Small }}
            actions={actions.unwrap_or_default()} actions_start={start.unwrap_or_default()}>
            <Content body={s.body} />
        </Dialog>
    }
}

#[derive(Properties, PartialEq)]
struct FrameProps {
    setup: Setup,
}

/// A frame's opener, with its own open state.
#[function_component]
fn DialogDemo(props: &FrameProps) -> Html {
    let open = use_state(|| false);
    let toggle = {
        let open = open.clone();
        Callback::from(move |_: MouseEvent| open.set(true))
    };

    html! {
        <Stack>
            <Cluster>
                <Button onclick={toggle}>{ "Open the dialog" }</Button>
            </Cluster>
            <p class={class::SAMPLE}>{ "The page behind: it does not move, and its scrollbar keeps its room, as the dialog opens and closes." }</p>
            <DialogOf setup={props.setup.clone()} open={*open} on_open={bind(&open)} />
        </Stack>
    }
}

#[function_component]
pub fn DialogPage() -> Html {
    let size = use_state(DialogSize::default);
    let dismiss = use_state(DialogDismiss::default);
    let title = use_state(|| Text::Short);
    let body = use_state(|| Body::Short);
    let icon = use_state(|| false);
    let head = use_state(|| false);
    let flush = use_state(|| false);
    let close_regular = use_state(|| false);
    let sheet = use_state(|| true);
    let actions = use_state(|| true);
    let start = use_state(|| false);
    let busy = use_state(|| false);
    let open = use_state(|| false);

    let setup = Setup {
        size: *size,
        dismiss: *dismiss,
        title: *title,
        body: *body,
        icon: *icon,
        head: *head,
        flush: *flush,
        close_regular: *close_regular,
        sheet: *sheet,
        actions: *actions,
        start: *start,
        busy: *busy,
    };
    let controls = html! {
        <>
            <Select<DialogSize> label="size" options={SIZES} value={*size} on_change={bind(&size)} />
            <Select<DialogDismiss> label="dismiss" options={DISMISSES} value={*dismiss} on_change={bind(&dismiss)} />
            <Select<Text> label="title" options={Text::OPTIONS} value={*title} on_change={bind(&title)} />
            <Select<Body> label="Body" options={BODIES} value={*body} on_change={bind(&body)} />
            <Toggle label="icon" value={*icon} on_change={bind(&icon)} />
            <Toggle label="head_actions" value={*head} on_change={bind(&head)} />
            <Toggle label="flush" value={*flush} on_change={bind(&flush)} />
            <Toggle label="regular close" value={*close_regular} on_change={bind(&close_regular)} />
            <Toggle label="sheet (on a phone)" value={*sheet} on_change={bind(&sheet)} />
            <Toggle label="actions" value={*actions} on_change={bind(&actions)} />
            <Toggle label="actions_start" value={*start} on_change={bind(&start)} />
            <Toggle label="busy" value={*busy} on_change={bind(&busy)} />
            <Toggle label="open" value={*open} on_change={bind(&open)} />
        </>
    };

    let mut actions_code = String::new();
    if *actions {
        actions_code = format!(
            "<><Button variant={{ButtonVariant::Ghost}}{d} onclick={{cancel}}>{{ \"Cancel\" }}</Button><Button variant={{ButtonVariant::Primary}}{b} onclick={{save}}>{{ \"Save\" }}</Button></>",
            d = if *busy { " disabled=true" } else { "" },
            b = if *busy { " busy=true" } else { "" },
        );
    }
    let code = snippet(
        &[],
        El::new("Dialog")
            .expr("open", "*open")
            .expr("on_dismiss", "on_dismiss")
            .text("title", &title.of("Rename the list"))
            .expr_if(*icon, "icon", "html! { <Icon glyph={Glyph::Ellipsis} /> }")
            .expr_if(
                *head,
                "head_actions",
                "html! { <Button variant={ButtonVariant::Quiet} size={ButtonSize::Small} icon={more} label=\"More\" /> }",
            )
            .variant("size", *size)
            .variant("dismiss", *dismiss)
            .expr_if(*flush, "flush", "true")
            .expr_if(*close_regular, "close_size", "ButtonSize::Regular")
            .expr_if(!*sheet, "sheet", "false")
            .expr_if(*actions, "actions", format!("html! {{ {actions_code} }}"))
            .expr_if(
                *start,
                "actions_start",
                "html! { <Button variant={ButtonVariant::Danger} onclick={delete}>{ \"Delete\" }</Button> }",
            )
            .line_if(*body != Body::Empty, "<p>{ \"What the dialog is about.\" }</p>"),
    );

    html! {
        <>
            <Demo {controls} {code}>
                <DialogDemo setup={setup.clone()} />
            </Demo>
            <DialogOf {setup} open={*open} on_open={bind(&open)} />
        </>
    }
}
