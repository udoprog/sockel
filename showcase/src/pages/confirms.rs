use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::class;

use plinth::{
    Align, Button, ButtonSize, ButtonVariant, Cluster, Confirm, ConfirmButton, ConfirmHost,
    ConfirmInline, ConfirmPopover, ConfirmRequest, Glyph, Icon, Justify, PopoverAlign, Side, Stack,
    ask,
};

use crate::code::{El, snippet};
use crate::controls::{Demo, Select, Text, Toggle, bind};

const VARIANTS: &[(&str, ButtonVariant)] = &[
    ("Secondary", ButtonVariant::Secondary),
    ("Primary", ButtonVariant::Primary),
    ("Ghost", ButtonVariant::Ghost),
    ("Quiet", ButtonVariant::Quiet),
    ("Danger", ButtonVariant::Danger),
];

const SIZES: &[(&str, ButtonSize)] = &[
    ("Regular", ButtonSize::Regular),
    ("Small", ButtonSize::Small),
    ("Touch", ButtonSize::Touch),
];

const SIDES: &[(&str, Side)] = &[
    ("Below", Side::Below),
    ("Above", Side::Above),
    ("Start", Side::Start),
    ("End", Side::End),
];

const ALIGNS: &[(&str, PopoverAlign)] = &[
    ("Start", PopoverAlign::Start),
    ("Center", PopoverAlign::Center),
    ("End", PopoverAlign::End),
];

fn trash() -> Html {
    html! { <Icon glyph={Glyph::Close} /> }
}

const ICON_CODE: &str = "html! { <Icon glyph={Glyph::Close} /> }";

/// What a confirm says and shows, as the controls set it.
#[derive(Properties, PartialEq, Clone)]
struct Setup {
    title: Text,
    explanation: Text,
    danger: bool,
    busy: bool,
    error: bool,
    details: bool,
    icon: bool,
}

impl Setup {
    fn title(&self) -> String {
        self.title.of("Delete the list?")
    }

    fn explanation(&self) -> String {
        self.explanation
            .of("The list and its 12 items are removed for good.")
    }

    fn details(&self) -> Html {
        if !self.details {
            return Html::default();
        }
        html! {
            <ul>
                <li>{ "12 items" }</li>
                <li>{ "3 shared links" }</li>
            </ul>
        }
    }

    fn error(&self) -> Option<AttrValue> {
        self.error
            .then(|| "Could not delete: the server did not answer.".into())
    }

    fn icon(&self) -> Option<Html> {
        self.icon.then(trash)
    }
}

/// The states every confirm page shares, and the controls for them.
struct Common {
    title: UseStateHandle<Text>,
    explanation: UseStateHandle<Text>,
    danger: UseStateHandle<bool>,
    busy: UseStateHandle<bool>,
    error: UseStateHandle<bool>,
    details: UseStateHandle<bool>,
    icon: UseStateHandle<bool>,
}

#[hook]
fn use_common() -> Common {
    Common {
        title: use_state(|| Text::Short),
        explanation: use_state(|| Text::Short),
        danger: use_state(|| true),
        busy: use_state(|| false),
        error: use_state(|| false),
        details: use_state(|| false),
        icon: use_state(|| false),
    }
}

impl Common {
    fn setup(&self) -> Setup {
        Setup {
            title: *self.title,
            explanation: *self.explanation,
            danger: *self.danger,
            busy: *self.busy,
            error: *self.error,
            details: *self.details,
            icon: *self.icon,
        }
    }

    fn controls(&self) -> Html {
        html! {
            <>
                <Select<Text> label="title" options={Text::OPTIONS} value={*self.title} on_change={bind(&self.title)} />
                <Select<Text> label="explanation" options={Text::OPTIONS} value={*self.explanation} on_change={bind(&self.explanation)} />
                <Toggle label="danger" value={*self.danger} on_change={bind(&self.danger)} />
                <Toggle label="busy" value={*self.busy} on_change={bind(&self.busy)} />
                <Toggle label="error" value={*self.error} on_change={bind(&self.error)} />
                <Toggle label="details" value={*self.details} on_change={bind(&self.details)} />
                <Toggle label="icon" value={*self.icon} on_change={bind(&self.icon)} />
            </>
        }
    }

    /// The props shared by the three, as code.
    fn code(&self, el: El, accept: &str) -> El {
        let s = self.setup();
        el.text("title", &s.title())
            .text("explanation", &s.explanation())
            .text("accept", accept)
            .expr_if(!s.danger, "danger", "false")
            .flag("busy", s.busy)
            .expr_if(s.error, "error", "Some(\"Could not delete.\".into())")
            .expr_if(s.details, "details", "html! { <ul>{ \"...\" }</ul> }")
            .expr_if(s.icon, "icon", ICON_CODE)
    }
}

#[derive(Properties, PartialEq)]
struct ConfirmDemoProps {
    setup: Setup,
    open: bool,
    on_open: Callback<bool>,
}

/// One `Confirm`, open as `open` says.
#[function_component]
fn ConfirmOf(props: &ConfirmDemoProps) -> Html {
    let s = &props.setup;
    let close = props.on_open.reform(|()| false);

    html! {
        <Confirm open={props.open} title={s.title()} explanation={s.explanation()}
            details={s.details()} accept="Delete" icon={s.icon()} danger={s.danger}
            busy={s.busy} error={s.error()} on_accept={close.clone()} on_cancel={close} />
    }
}

#[derive(Properties, PartialEq)]
struct FrameProps {
    setup: Setup,
}

/// A frame's openers: the dialog, and a promise-style question put to the host.
#[function_component]
fn ConfirmDemo(props: &FrameProps) -> Html {
    let open = use_state(|| false);
    let answer = use_state(|| "not asked".to_owned());
    let show = {
        let open = open.clone();
        Callback::from(move |_: MouseEvent| open.set(true))
    };
    let put = {
        let (setup, answer) = (props.setup.clone(), answer.clone());
        Callback::from(move |_: MouseEvent| {
            let mut request = ConfirmRequest::new(setup.title(), setup.explanation());
            request.accept = Some("Delete".into());
            request.danger = setup.danger;
            request.details = setup.details();
            request.icon = setup.icon();
            let answer = answer.clone();
            answer.set("waiting".to_owned());
            spawn_local(async move {
                let yes = ask(request).await;
                answer.set(if yes { "accepted" } else { "cancelled" }.to_owned());
            });
        })
    };

    html! {
        <Stack>
            <Cluster>
                <Button onclick={show}>{ "Open the confirm" }</Button>
                <Button variant={ButtonVariant::Ghost} onclick={put}>{ "ask(..).await" }</Button>
            </Cluster>
            <p class={class::SAMPLE}>{ format!("The host's answer: {}", *answer) }</p>
            <ConfirmOf setup={props.setup.clone()} open={*open} on_open={bind(&open)} />
        </Stack>
    }
}

#[function_component]
pub fn ConfirmPage() -> Html {
    let common = use_common();
    let open = use_state(|| false);
    let setup = common.setup();

    let controls = html! {
        <>
            { common.controls() }
            <Toggle label="open" value={*open} on_change={bind(&open)} />
        </>
    };
    let code = snippet(
        &[],
        common
            .code(El::new("Confirm"), "Delete")
            .expr("open", "*open")
            .expr("on_accept", "on_accept")
            .expr("on_cancel", "on_cancel"),
    );

    html! {
        <>
            <ConfirmHost />
            <Demo {controls} {code}>
                <ConfirmDemo setup={setup.clone()} />
            </Demo>
            <ConfirmOf {setup} open={*open} on_open={bind(&open)} />
        </>
    }
}

#[derive(Properties, PartialEq)]
struct PopoverDemoProps {
    setup: Setup,
    side: Side,
    align: PopoverAlign,
    open: bool,
    on_open: Callback<bool>,
    button: bool,
    compact: bool,
}

/// A compact confirm's line: the name it asks about, in bold, one item
/// with its question mark.
fn subject(details: bool) -> Html {
    if details {
        html! { <span><strong>{ "Quarterly report" }</strong>{ "?" }</span> }
    } else {
        Html::default()
    }
}

const SUBJECT_CODE: &str =
    "html! { <span><strong>{ \"Quarterly report\" }</strong>{ \"?\" }</span> }";

#[function_component]
fn PopoverDemo(props: &PopoverDemoProps) -> Html {
    let s = &props.setup;
    let anchor = use_node_ref();
    let own = use_state(|| false);
    let open = props.open || *own;
    let close = {
        let (own, on_open) = (own.clone(), props.on_open.clone());
        Callback::from(move |()| {
            own.set(false);
            on_open.emit(false);
        })
    };
    let ask = {
        let own = own.clone();
        Callback::from(move |_: MouseEvent| own.set(true))
    };

    html! {
        <Stack>
            <Cluster>
                if props.button {
                    <ConfirmButton variant={ButtonVariant::Danger} title={s.title()}
                        explanation={s.explanation()} details={s.details()} accept="Delete"
                        accept_icon={s.icon()} danger={s.danger} side={props.side}
                        align={props.align} on_accept={close.clone()} busy={s.busy}>
                        { "Delete" }
                    </ConfirmButton>
                } else if props.compact {
                    <Button node_ref={anchor.clone()} variant={ButtonVariant::Danger}
                        expanded={Some(open)} haspopup="dialog" onclick={ask}>{ "Delete" }</Button>
                    <ConfirmPopover {anchor} {open} compact=true title={s.title()}
                        explanation="Delete" details={subject(s.details)}
                        explanation_icon={s.icon()} accept="Yes" cancel_label="No"
                        danger={s.danger} busy={s.busy} error={s.error()} side={props.side}
                        align={props.align} on_accept={close.clone()} on_cancel={close} />
                } else {
                    <Button node_ref={anchor.clone()} variant={ButtonVariant::Danger}
                        expanded={Some(open)} haspopup="dialog" onclick={ask}>{ "Delete" }</Button>
                    <ConfirmPopover {anchor} {open} title={s.title()} explanation={s.explanation()}
                        details={s.details()} accept="Delete" icon={s.icon()} danger={s.danger}
                        busy={s.busy} error={s.error()} side={props.side} align={props.align}
                        on_accept={close.clone()} on_cancel={close} />
                }
            </Cluster>
            <p class={class::SAMPLE}>{ "Nothing around the button moves as the confirm opens beside it." }</p>
        </Stack>
    }
}

#[function_component]
pub fn ConfirmPopoverPage() -> Html {
    let common = use_common();
    let side = use_state(Side::default);
    let align = use_state(PopoverAlign::default);
    let button = use_state(|| false);
    let compact = use_state(|| false);
    let open = use_state(|| false);
    let setup = common.setup();

    let controls = html! {
        <>
            { common.controls() }
            <Select<Side> label="side" options={SIDES} value={*side} on_change={bind(&side)} />
            <Select<PopoverAlign> label="align" options={ALIGNS} value={*align} on_change={bind(&align)} />
            <Toggle label="ConfirmButton" value={*button} on_change={bind(&button)} />
            <Toggle label="compact" value={*compact} on_change={bind(&compact)} />
            <Toggle label="open" value={*open} on_change={bind(&open)} />
        </>
    };
    let code = if *button {
        snippet(
            &[],
            common
                .code(El::new("ConfirmButton"), "Delete")
                .variant("variant", ButtonVariant::Danger)
                .variant("side", *side)
                .variant("align", *align)
                .expr("on_accept", "on_accept")
                .line("{ \"Delete\" }"),
        )
    } else if *compact {
        snippet(
            &["let anchor = use_node_ref();".to_owned()],
            El::new("ConfirmPopover")
                .expr("anchor", "anchor")
                .expr("open", "*open")
                .flag("compact", true)
                .text("title", &setup.title())
                .text("explanation", "Delete")
                .expr_if(setup.details, "details", SUBJECT_CODE)
                .expr_if(setup.icon, "explanation_icon", ICON_CODE)
                .text("accept", "Yes")
                .text("cancel_label", "No")
                .expr_if(!setup.danger, "danger", "false")
                .flag("busy", setup.busy)
                .expr_if(setup.error, "error", "Some(\"Could not delete.\".into())")
                .variant("side", *side)
                .variant("align", *align)
                .expr("on_accept", "on_accept")
                .expr("on_cancel", "on_cancel"),
        )
    } else {
        snippet(
            &["let anchor = use_node_ref();".to_owned()],
            common
                .code(El::new("ConfirmPopover"), "Delete")
                .expr("anchor", "anchor")
                .expr("open", "*open")
                .variant("side", *side)
                .variant("align", *align)
                .expr("on_accept", "on_accept")
                .expr("on_cancel", "on_cancel"),
        )
    };

    html! {
        <Demo {controls} {code}>
            <PopoverDemo {setup} side={*side} align={*align} open={*open}
                on_open={bind(&open)} button={*button} compact={*compact} />
        </Demo>
    }
}

#[function_component]
pub fn ConfirmInlinePage() -> Html {
    let label = use_state(|| Text::Short);
    let variant = use_state(|| ButtonVariant::Danger);
    let size = use_state(ButtonSize::default);
    let danger = use_state(|| true);
    let disabled = use_state(|| false);
    let busy = use_state(|| false);
    let icon = use_state(|| false);
    let asking = use_state(|| false);
    let accepted = use_state(|| 0_u32);

    let controls = html! {
        <>
            <Select<Text> label="label" options={Text::OPTIONS} value={*label} on_change={bind(&label)} />
            <Select<ButtonVariant> label="variant" options={VARIANTS} value={*variant} on_change={bind(&variant)} />
            <Select<ButtonSize> label="size" options={SIZES} value={*size} on_change={bind(&size)} />
            <Toggle label="danger" value={*danger} on_change={bind(&danger)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
            <Toggle label="busy" value={*busy} on_change={bind(&busy)} />
            <Toggle label="icon" value={*icon} on_change={bind(&icon)} />
            <Toggle label="asking" value={*asking} on_change={bind(&asking)} />
        </>
    };
    let accept = {
        let accepted = accepted.clone();
        Callback::from(move |()| accepted.set(*accepted + 1))
    };
    let name = label.of("Delete");
    let code = snippet(
        &[],
        El::new("ConfirmInline")
            .variant("variant", *variant)
            .variant("size", *size)
            .expr_if(!*danger, "danger", "false")
            .flag("disabled", *disabled)
            .flag("busy", *busy)
            .expr_if(*icon, "icon", ICON_CODE)
            .expr("asking", "Some(*asking)")
            .expr("on_ask", "on_ask")
            .expr("on_accept", "on_accept")
            .line(crate::code::text_node(&name)),
    );

    html! {
        <Demo {controls} {code}>
            <Stack>
                <div class={class::SAMPLE}>
                    <Cluster justify={Justify::Between} align={Align::Center}>
                        <span>{ "Quarterly report" }</span>
                        <ConfirmInline variant={*variant} size={*size} danger={*danger}
                            disabled={*disabled} busy={*busy} asking={Some(*asking)}
                            icon={(*icon).then(trash)} on_ask={bind(&asking)}
                            on_accept={accept}>
                            { name }
                        </ConfirmInline>
                    </Cluster>
                </div>
                <p class={class::SAMPLE}>{ format!("Accepted {} times; the row above keeps its size asking or not.", *accepted) }</p>
            </Stack>
        </Demo>
    }
}
