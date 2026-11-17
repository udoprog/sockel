use yew::prelude::*;

use crate::class;

use sockel::{
    Button, ButtonVariant, Card, Cluster, Dialog, Dismiss, Glyph, Icon, Menu, MenuItem,
    OverlayHost, Overlays, Popover, Stack, Toast, ToastHost, Tooltip, use_overlay,
};

use crate::code::{El, snippet};
use crate::controls::{Demo, Select, Toggle, bind};

/// The colours of the element the overlays are asked to stand in.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum Frame {
    #[default]
    Page,
    Light,
    Dark,
}

const FRAMES: &[(&str, Frame)] = &[
    ("The page's", Frame::Page),
    ("Scoped light", Frame::Light),
    ("Scoped dark", Frame::Dark),
];

#[function_component]
fn Scene() -> Html {
    let popover = use_state(|| false);
    let dialog = use_state(|| false);
    let panel = use_state(|| false);
    let toast = use_state(|| 0u32);
    let undone = use_state(|| 0u32);
    let pop_anchor = use_node_ref();
    let tip_anchor = use_node_ref();

    let toggle_popover = {
        let popover = popover.clone();
        Callback::from(move |_: MouseEvent| popover.set(!*popover))
    };
    let close_popover = {
        let popover = popover.clone();
        Callback::from(move |_: Dismiss| popover.set(false))
    };
    let open_dialog = {
        let dialog = dialog.clone();
        Callback::from(move |_: MouseEvent| dialog.set(true))
    };
    let close_dialog = {
        let dialog = dialog.clone();
        Callback::from(move |_: Dismiss| dialog.set(false))
    };
    let done = {
        let dialog = dialog.clone();
        Callback::from(move |_: MouseEvent| dialog.set(false))
    };
    let toggle_panel = {
        let panel = panel.clone();
        Callback::from(move |_: MouseEvent| panel.set(!*panel))
    };
    let close_panel = {
        let panel = panel.clone();
        Callback::from(move |()| panel.set(false))
    };
    use_overlay(*panel, close_panel);
    let show_toast = {
        let toast = toast.clone();
        Callback::from(move |_: MouseEvent| toast.set(toast.wrapping_add(1) | 1))
    };
    let on_undo = {
        let (toast, undone) = (toast.clone(), undone.clone());
        Callback::from(move |()| {
            undone.set(*undone + 1);
            toast.set(0);
        })
    };
    let on_dismiss_toast = {
        let toast = toast.clone();
        Callback::from(move |()| toast.set(0))
    };

    html! {
        <Stack>
            <Cluster>
                <Button node_ref={pop_anchor.clone()} onclick={toggle_popover}>{ "A popover" }</Button>
                <Button variant={ButtonVariant::Primary} onclick={open_dialog}>{ "A dialog" }</Button>
                <Menu text={html! { "A menu" }} label="A menu on the page">
                    <MenuItem>{ "Rename" }</MenuItem>
                    <MenuItem>{ "Copy link" }</MenuItem>
                </Menu>
                <Button pressed={Some(*panel)} onclick={toggle_panel.clone()}>{ "A panel of the app's own" }</Button>
            </Cluster>
            <p class={class::SAMPLE}>{ "The page behind: it does not move as overlays open and close." }</p>
            if *panel {
                <Card>
                    <Stack>
                        <p class={class::PROSE}>{ "On the stack with use_overlay: Escape closes the menu first, then this panel." }</p>
                        <Cluster>
                            <Menu text={html! { "A menu" }} label="A menu in the panel">
                                <MenuItem>{ "Rename" }</MenuItem>
                            </Menu>
                            <Button onclick={toggle_panel}>{ "Close" }</Button>
                        </Cluster>
                    </Stack>
                </Card>
            }
            <Popover anchor={pop_anchor} open={*popover} on_dismiss={close_popover} title="Popover">
                <Stack>
                    <p class={class::PROSE}>{ "A tooltip over this popover: hover or focus the button." }</p>
                    <Cluster>
                        <Button node_ref={tip_anchor.clone()} size={sockel::ButtonSize::Small}>{ "Hover me" }</Button>
                        <Tooltip anchor={tip_anchor} text="Over the popover" />
                    </Cluster>
                </Stack>
            </Popover>
            <Dialog open={*dialog} on_dismiss={close_dialog} title="A dialog"
                actions={html! { <Button variant={ButtonVariant::Primary} onclick={done}>{ "Done" }</Button> }}>
                <Stack>
                    <p class={class::PROSE}>
                        { "Escape closes the innermost first. The toast's Undo works over the dialog." }
                    </p>
                    <Cluster>
                        <Menu text={html! { "A menu" }} label="A menu in the dialog">
                            <MenuItem>{ "Rename" }</MenuItem>
                            <MenuItem>{ "Copy link" }</MenuItem>
                        </Menu>
                        <Button onclick={show_toast}>{ "Show a toast" }</Button>
                    </Cluster>
                    <span class={class::CONTROL_LABEL}>{ format!("Undone {} times.", *undone) }</span>
                </Stack>
            </Dialog>
            if *toast != 0 {
                <ToastHost>
                    <Toast key={*toast} icon={html! { <Icon glyph={Glyph::Check} /> }}
                        action="Undo" onaction={on_undo} ondismiss={on_dismiss_toast}>
                        { "Saved" }
                    </Toast>
                </ToastHost>
            }
        </Stack>
    }
}

#[derive(Properties, PartialEq)]
struct ScopedProps {
    provider: bool,
    frame: Frame,
}

/// The scene in a frame that sets its own colours, with or without the host.
#[function_component]
fn Scoped(props: &ScopedProps) -> Html {
    let classes = classes!(
        class::SCOPE,
        (props.frame == Frame::Light).then_some(class::SCOPE_LIGHT),
        (props.frame == Frame::Dark).then_some(class::SCOPE_DARK),
    );
    let scene = if props.provider {
        html! { <OverlayHost><Scene /></OverlayHost> }
    } else {
        html! {
            <ContextProvider<Overlays> context={Overlays::fallback()}>
                <Scene />
            </ContextProvider<Overlays>>
        }
    };

    html! { <div class={classes}>{ scene }</div> }
}

#[function_component]
pub fn OverlaysPage() -> Html {
    let provider = use_state(|| true);
    let frame = use_state(Frame::default);

    let controls = html! {
        <>
            <Toggle label="OverlayHost" value={*provider} on_change={bind(&provider)} />
            <Select<Frame> label="frame colours" options={FRAMES} value={*frame} on_change={bind(&frame)} />
        </>
    };

    let scene = El::new("Dialog")
        .expr("open", "*open")
        .expr("on_dismiss", "on_dismiss")
        .line("<Menu .. />")
        .line("<ToastHost><Toast .. /></ToastHost>");
    let host = if *provider {
        El::new("OverlayHost").child(scene)
    } else {
        scene
    };
    let root = if *frame == Frame::Page {
        host
    } else {
        El::new("div")
            .text("class", "my-frame")
            .line("// sets --bg, --surface, --text, --accent")
            .child(host)
    };
    let code = snippet(&[], root);

    html! {
        <Demo {controls} {code}>
            <Scoped provider={*provider} frame={*frame} />
            <span class={class::CONTROL_LABEL}>
                { if *provider {
                    "With OverlayHost: overlays render inside the frame and take its colours."
                } else {
                    "Without one: overlays render in #sockel-overlay at the end of <body> and take the page's colours."
                } }
            </span>
        </Demo>
    }
}
