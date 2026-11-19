//! Confirm, ConfirmPopover, ConfirmButton, ConfirmInline and the host for
//! promise-style confirms (`style/_confirm.scss`).

use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

use phrasebook02::yew023::use_phrases_with;
use web_sys::HtmlElement;
use yew::prelude::*;

use crate::phrasebookcatalog01::{L10N, label};
use crate::popover::focus_lost;
use crate::{
    ActionGroup, Button, ButtonSize, ButtonVariant, Dialog, DialogDismiss, Dismiss, Glyph, Icon,
    Notice, Popover, PopoverAlign, Side, Tone, class,
};

fn accept_variant(danger: bool) -> ButtonVariant {
    if danger {
        ButtonVariant::Danger
    } else {
        ButtonVariant::Primary
    }
}

/// What may close a confirm: nothing while its action is running.
fn dismissal(busy: bool) -> DialogDismiss {
    if busy {
        DialogDismiss::Actions
    } else {
        DialogDismiss::Any
    }
}

fn focus(node: &NodeRef) {
    if let Some(element) = node.cast::<HtmlElement>() {
        let options = web_sys::FocusOptions::new();
        options.set_prevent_scroll(true);
        _ = element.focus_with_options(&options);
    }
}

/// The text a confirm says, what it affects and what went wrong.
fn body(explanation: &AttrValue, details: &Html, error: Option<&AttrValue>) -> Html {
    html! {
        <div class={class::CONFIRM}>
            <p class={class::CONFIRM_TEXT}>{ explanation.clone() }</p>
            { details.clone() }
            if let Some(error) = error {
                <Notice tone={Tone::Danger}>{ error.clone() }</Notice>
            }
        </div>
    }
}

/// The cancel and accept buttons, the accept last. The safe one takes the
/// focus (`focus` says which) so Enter never destroys by habit.
#[allow(clippy::too_many_arguments)]
fn buttons(
    cancel_label: &AttrValue,
    accept: &AttrValue,
    icon: &Option<Html>,
    danger: bool,
    busy: bool,
    accept_disabled: bool,
    on_cancel: &Callback<()>,
    on_accept: &Callback<()>,
    cancel_ref: &NodeRef,
    accept_ref: &NodeRef,
) -> Html {
    html! {
        <>
            <Button variant={ButtonVariant::Ghost} disabled={busy} node_ref={cancel_ref.clone()}
                onclick={on_cancel.reform(|_: MouseEvent| ())} testid="confirm-cancel">
                { cancel_label.clone() }
            </Button>
            <Button variant={accept_variant(danger)} {busy} icon={icon.clone()}
                disabled={accept_disabled} node_ref={accept_ref.clone()} onclick={on_accept.reform(|_: MouseEvent| ())}
                testid="confirm-accept">
                { accept.clone() }
            </Button>
        </>
    }
}

#[derive(Properties, PartialEq)]
pub struct ConfirmProps {
    pub open: bool,
    /// Its heading and accessible name, naming the action ("Delete list?").
    pub title: AttrValue,
    /// What happens and what is lost.
    pub explanation: AttrValue,
    /// A list of what it affects, below the explanation.
    #[prop_or_default]
    pub details: Html,
    /// The accepting button's label.
    pub accept: AttrValue,
    /// The cancelling button's label, else the catalog's.
    #[prop_or_default]
    pub cancel_label: Option<AttrValue>,
    /// The accepting button's icon.
    #[prop_or_default]
    pub icon: Option<Html>,
    /// Accepting destroys something: the danger button, and the focus on
    /// Cancel. Otherwise the accepting button is primary and has the focus.
    #[prop_or(true)]
    pub danger: bool,
    /// Working on it: the accepting button is busy and nothing closes it.
    #[prop_or_default]
    pub busy: bool,
    /// The accepting button is disabled until the app's condition holds
    /// (the name typed in `details`, say).
    #[prop_or_default]
    pub accept_disabled: bool,
    /// What went wrong when accepting failed, below the explanation.
    #[prop_or_default]
    pub error: Option<AttrValue>,
    pub on_accept: Callback<()>,
    /// Cancel, Escape, the close button and a press on the scrim.
    pub on_cancel: Callback<()>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// A [`Dialog`] asking before an action: the explanation, then Cancel and the
/// accepting button pinned below. Cancel has the focus when accepting
/// destroys something. Escape, the close button and the scrim cancel, unless
/// it is busy; the owner sets `open` to false.
#[function_component]
pub fn Confirm(props: &ConfirmProps) -> Html {
    let phrases = use_phrases_with(&L10N);
    let cancel_label = label(&phrases, &props.cancel_label, "sockel-confirm-cancel");
    let (cancel_ref, accept_ref) = (use_node_ref(), use_node_ref());
    {
        let (cancel_ref, accept_ref) = (cancel_ref.clone(), accept_ref.clone());
        use_effect_with((props.open, props.danger), move |&(open, danger)| {
            if open {
                focus(if danger { &cancel_ref } else { &accept_ref });
            }
        });
    }
    let on_dismiss = props.on_cancel.reform(|_: Dismiss| ());
    let actions = buttons(
        &cancel_label,
        &props.accept,
        &props.icon,
        props.danger,
        props.busy,
        props.accept_disabled,
        &props.on_cancel,
        &props.on_accept,
        &cancel_ref,
        &accept_ref,
    );

    html! {
        <Dialog open={props.open} {on_dismiss} title={props.title.clone()}
            dismiss={dismissal(props.busy)} class={props.class.clone()}
            testid={props.testid.clone()} {actions}>
            { body(&props.explanation, &props.details, props.error.as_ref()) }
        </Dialog>
    }
}

#[derive(Properties, PartialEq)]
pub struct ConfirmPopoverProps {
    /// What it opens beside, and where focus returns.
    #[prop_or_default]
    pub anchor: NodeRef,
    /// As for [`crate::PopoverProps::anchor_element`].
    #[prop_or_default]
    pub anchor_element: Option<web_sys::Element>,
    pub open: bool,
    /// Its accessible name, and the sheet's heading on a phone.
    pub title: AttrValue,
    pub explanation: AttrValue,
    #[prop_or_default]
    pub details: Html,
    pub accept: AttrValue,
    #[prop_or_default]
    pub cancel_label: Option<AttrValue>,
    #[prop_or_default]
    pub icon: Option<Html>,
    /// As for [`Confirm`].
    #[prop_or(true)]
    pub danger: bool,
    #[prop_or_default]
    pub busy: bool,
    /// As for [`Confirm`].
    #[prop_or_default]
    pub accept_disabled: bool,
    #[prop_or_default]
    pub error: Option<AttrValue>,
    #[prop_or_default]
    pub side: Side,
    #[prop_or_default]
    pub align: PopoverAlign,
    /// What it says on one line (`explanation`, then `details`, such as a
    /// name in bold) above Cancel and the accepting button joined as their
    /// icons alone, named by `cancel_label` and `accept`. As wide as what it
    /// says, up to the viewport; on a phone a sheet without a head.
    #[prop_or_default]
    pub compact: bool,
    /// An icon before what a compact confirm says.
    #[prop_or_default]
    pub explanation_icon: Option<Html>,
    pub on_accept: Callback<()>,
    /// Cancel, Escape and a press outside; not while busy.
    pub on_cancel: Callback<()>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// A confirm popover's classes: `compact` for one as wide as what it says.
pub fn confirm_popover(compact: bool) -> Classes {
    classes!(class::CONFIRM_POPOVER, compact.then_some(class::COMPACT))
}

/// A compact confirm: one line, then Cancel (`Glyph::Close`) and the
/// accepting button (`icon`, else `Glyph::Check`) joined, icons alone.
fn compact(
    props: &ConfirmPopoverProps,
    cancel_label: &AttrValue,
    cancel_ref: &NodeRef,
    accept_ref: &NodeRef,
) -> Html {
    let accept_icon = props
        .icon
        .clone()
        .unwrap_or_else(|| html! { <Icon glyph={Glyph::Check} /> });

    html! {
        <div class={class::CONFIRM}>
            <p class={classes!(class::CONFIRM_TEXT, class::CONFIRM_LINE)}>
                { props.explanation_icon.clone().unwrap_or_default() }
                <span>{ props.explanation.clone() }</span>
                { props.details.clone() }
            </p>
            if let Some(error) = &props.error {
                <Notice tone={Tone::Danger}>{ error.clone() }</Notice>
            }
            <div class={class::CONFIRM_PAIR} role="group">
                <Button icon={html! { <Icon glyph={Glyph::Close} /> }} disabled={props.busy}
                    label={cancel_label.clone()} title={cancel_label.clone()}
                    node_ref={cancel_ref.clone()} class={class::CONFIRM_NO}
                    onclick={props.on_cancel.reform(|_: MouseEvent| ())} testid="confirm-cancel" />
                <Button variant={accept_variant(props.danger)} busy={props.busy}
                    disabled={props.accept_disabled} icon={accept_icon} label={props.accept.clone()} title={props.accept.clone()}
                    node_ref={accept_ref.clone()} class={class::CONFIRM_YES}
                    onclick={props.on_accept.reform(|_: MouseEvent| ())} testid="confirm-accept" />
            </div>
        </div>
    }
}

/// A confirm beside the element that asked, in a [`Popover`] a fixed width
/// wide: the explanation, then Cancel and the accepting button, or with
/// `compact` one line and two icons. Focus goes to Cancel when accepting
/// destroys something; Escape and a press outside cancel, and focus returns
/// to the anchor. A sheet on a phone.
#[function_component]
pub fn ConfirmPopover(props: &ConfirmPopoverProps) -> Html {
    let phrases = use_phrases_with(&L10N);
    let cancel_label = label(&phrases, &props.cancel_label, "sockel-confirm-cancel");
    let (cancel_ref, accept_ref) = (use_node_ref(), use_node_ref());
    let on_dismiss = {
        let (on_cancel, busy) = (props.on_cancel.clone(), props.busy);
        Callback::from(move |_: Dismiss| {
            if !busy {
                on_cancel.emit(());
            }
        })
    };

    let content = if props.compact {
        compact(props, &cancel_label, &cancel_ref, &accept_ref)
    } else {
        html! {
            <>
                { body(&props.explanation, &props.details, props.error.as_ref()) }
                <ActionGroup end=true>
                    { buttons(&cancel_label, &props.accept, &props.icon, props.danger,
                        props.busy, props.accept_disabled, &props.on_cancel, &props.on_accept, &cancel_ref, &accept_ref) }
                </ActionGroup>
            </>
        }
    };

    html! {
        <Popover anchor={props.anchor.clone()} anchor_element={props.anchor_element.clone()}
            open={props.open} {on_dismiss}
            side={props.side} align={props.align} title={props.title.clone()}
            head={!props.compact} trap=true
            focus={if props.danger { cancel_ref.clone() } else { accept_ref.clone() }}
            class={classes!(confirm_popover(props.compact), props.class.clone())}
            testid={props.testid.clone()}>
            { content }
        </Popover>
    }
}

#[derive(Properties, PartialEq)]
pub struct ConfirmButtonProps {
    #[prop_or_default]
    pub variant: ButtonVariant,
    #[prop_or_default]
    pub size: ButtonSize,
    #[prop_or_default]
    pub icon: Option<Html>,
    /// Working on the confirmed action: the button is busy.
    #[prop_or_default]
    pub busy: bool,
    #[prop_or_default]
    pub disabled: bool,
    pub title: AttrValue,
    pub explanation: AttrValue,
    #[prop_or_default]
    pub details: Html,
    pub accept: AttrValue,
    #[prop_or_default]
    pub cancel_label: Option<AttrValue>,
    #[prop_or_default]
    pub accept_icon: Option<Html>,
    #[prop_or(true)]
    pub danger: bool,
    /// As for [`Confirm`]: its popover's accepting button.
    #[prop_or_default]
    pub accept_disabled: bool,
    #[prop_or_default]
    pub side: Side,
    #[prop_or_default]
    pub align: PopoverAlign,
    pub on_accept: Callback<()>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// The button's label.
    #[prop_or_default]
    pub children: Html,
}

/// A [`Button`] that asks in a [`ConfirmPopover`] beside it before it acts:
/// the button keeps its box whether the popover is open or not.
#[function_component]
pub fn ConfirmButton(props: &ConfirmButtonProps) -> Html {
    let open = use_state(|| false);
    let anchor = use_node_ref();
    let ask = {
        let open = open.clone();
        Callback::from(move |_: MouseEvent| open.set(true))
    };
    let on_cancel = {
        let open = open.clone();
        Callback::from(move |()| open.set(false))
    };
    let on_accept = {
        let (open, accepted) = (open.clone(), props.on_accept.clone());
        Callback::from(move |()| {
            open.set(false);
            accepted.emit(());
        })
    };

    html! {
        <>
            <Button variant={props.variant} size={props.size} icon={props.icon.clone()}
                busy={props.busy} disabled={props.disabled} node_ref={anchor.clone()}
                expanded={Some(*open)} haspopup="dialog" class={props.class.clone()}
                testid={props.testid.clone()} onclick={ask}>
                { props.children.clone() }
            </Button>
            <ConfirmPopover {anchor} open={*open} title={props.title.clone()}
                explanation={props.explanation.clone()} details={props.details.clone()}
                accept={props.accept.clone()} cancel_label={props.cancel_label.clone()}
                icon={props.accept_icon.clone()} danger={props.danger}
                accept_disabled={props.accept_disabled}
                side={props.side} align={props.align} {on_accept} {on_cancel} />
        </>
    }
}

#[derive(Properties, PartialEq)]
pub struct ConfirmInlineProps {
    /// The button's variant and size, kept by the No and Yes buttons.
    #[prop_or_default]
    pub variant: ButtonVariant,
    #[prop_or_default]
    pub size: ButtonSize,
    #[prop_or_default]
    pub icon: Option<Html>,
    #[prop_or_default]
    pub disabled: bool,
    /// Working on the confirmed action: Yes is busy and No is disabled.
    #[prop_or_default]
    pub busy: bool,
    /// Whether it is asking, when the owner holds that state; otherwise it
    /// holds it itself.
    #[prop_or_default]
    pub asking: Option<bool>,
    /// Asked to start or stop asking.
    #[prop_or_default]
    pub on_ask: Callback<bool>,
    /// The group's accessible name while asking, else the catalog's.
    #[prop_or_default]
    pub question: Option<AttrValue>,
    #[prop_or_default]
    pub no_label: Option<AttrValue>,
    #[prop_or_default]
    pub yes_label: Option<AttrValue>,
    /// Yes is the danger button; otherwise primary.
    #[prop_or(true)]
    pub danger: bool,
    pub on_accept: Callback<()>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// The button's label.
    #[prop_or_default]
    pub children: Html,
}

/// The classes of the wrapper, and of a part that is `away`.
pub fn confirm_inline(part: &'static str, away: bool) -> Classes {
    classes!(part, away.then_some(class::AWAY))
}

/// A button that asks in place: it is replaced by No and Yes in the same
/// box. Both are always laid out in one grid cell and the one not shown is
/// hidden (`visibility`), so the box is as wide as the wider of the two and
/// as high as the button whether it asks or not, and a row it stands in
/// never moves. No takes the focus; Escape and No go back, and focus
/// returns to the button.
#[function_component]
pub fn ConfirmInline(props: &ConfirmInlineProps) -> Html {
    let phrases = use_phrases_with(&L10N);
    let own = use_state(|| false);
    let asking = props.asking.unwrap_or(*own);
    let set = {
        let (own, on_ask) = (own.clone(), props.on_ask.clone());
        Callback::from(move |value: bool| {
            own.set(value);
            on_ask.emit(value);
        })
    };
    let (idle_ref, ask_ref, no_ref) = (use_node_ref(), use_node_ref(), use_node_ref());

    {
        let (idle_ref, ask_ref, no_ref) = (idle_ref.clone(), ask_ref.clone(), no_ref.clone());
        let was = use_mut_ref(|| false);
        use_effect_with(asking, move |&asking| {
            let before = was.replace(asking);
            if asking {
                focus(&no_ref);
            } else if before {
                let active = web_sys::window()
                    .and_then(|w| w.document())
                    .and_then(|d| d.active_element());
                let inside = ask_ref
                    .cast::<web_sys::Element>()
                    .zip(active)
                    .is_some_and(|(ask, active)| ask.contains(Some(&active)));
                if inside || focus_lost() {
                    focus(&idle_ref);
                }
            }
        });
    }

    let onkeydown = {
        let set = set.clone();
        Callback::from(move |e: KeyboardEvent| {
            if asking && e.key() == "Escape" {
                e.stop_propagation();
                set.emit(false);
            }
        })
    };
    let ask = set.reform(|e: MouseEvent| {
        e.stop_propagation();
        true
    });
    let no = set.reform(|e: MouseEvent| {
        e.stop_propagation();
        false
    });
    let yes = {
        let (set, accepted) = (set.clone(), props.on_accept.clone());
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            set.emit(false);
            accepted.emit(());
        })
    };

    html! {
        <span class={classes!(class::CONFIRM_INLINE, props.class.clone())} {onkeydown}
            data-testid={props.testid.clone()}>
            <Button variant={props.variant} size={props.size} icon={props.icon.clone()}
                disabled={props.disabled} node_ref={idle_ref}
                class={confirm_inline(class::CONFIRM_IDLE, asking)} onclick={ask}>
                { props.children.clone() }
            </Button>
            <span ref={ask_ref} class={confirm_inline(class::CONFIRM_ASK, !asking)}
                role="group"
                aria-label={label(&phrases, &props.question, "sockel-confirm-inline-question")}>
                <Button size={props.size} variant={ButtonVariant::Ghost} disabled={props.busy}
                    node_ref={no_ref} onclick={no} testid="confirm-cancel">
                    { label(&phrases, &props.no_label, "sockel-confirm-inline-no") }
                </Button>
                <Button size={props.size} variant={accept_variant(props.danger)}
                    busy={props.busy} onclick={yes} testid="confirm-accept">
                    { label(&phrases, &props.yes_label, "sockel-confirm-inline-yes") }
                </Button>
            </span>
        </span>
    }
}

/// A question put to the [`ConfirmHost`]: what a [`Confirm`] shows.
#[derive(Clone, PartialEq)]
pub struct ConfirmRequest {
    pub title: AttrValue,
    pub explanation: AttrValue,
    pub details: Html,
    /// The accepting button's label, else the catalog's ("Confirm").
    pub accept: Option<AttrValue>,
    /// The cancelling button's label, else the catalog's.
    pub cancel_label: Option<AttrValue>,
    pub icon: Option<Html>,
    pub danger: bool,
}

impl ConfirmRequest {
    /// A destructive question, with the catalog's labels.
    pub fn new(title: impl Into<AttrValue>, explanation: impl Into<AttrValue>) -> Self {
        Self {
            title: title.into(),
            explanation: explanation.into(),
            details: Html::default(),
            accept: None,
            cancel_label: None,
            icon: None,
            danger: true,
        }
    }
}

#[derive(Default)]
struct Slot {
    value: Option<bool>,
    waker: Option<Waker>,
}

/// The sending end of an [`Answer`].
#[derive(Clone, Default)]
struct Reply(Rc<RefCell<Slot>>);

impl Reply {
    /// The first answer stands.
    fn send(&self, value: bool) {
        let mut slot = self.0.borrow_mut();
        if slot.value.is_none() {
            slot.value = Some(value);
            if let Some(waker) = slot.waker.take() {
                waker.wake();
            }
        }
    }
}

/// What [`ask`] returns: `true` when the user accepted, `false` when they
/// cancelled, or another question replaced theirs.
pub struct Answer(Reply);

impl Future for Answer {
    type Output = bool;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<bool> {
        let mut slot = (self.0).0.borrow_mut();
        match slot.value {
            Some(value) => Poll::Ready(value),
            None => {
                slot.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

/// The question showing, and who is waiting for its answer.
#[derive(Default)]
struct Showing(Option<Reply>);

impl Showing {
    /// Shows another question; the one showing is answered `false`.
    fn replace(&mut self, reply: Reply) {
        if let Some(old) = self.0.replace(reply) {
            old.send(false);
        }
    }

    fn resolve(&mut self, value: bool) {
        if let Some(reply) = self.0.take() {
            reply.send(value);
        }
    }
}

type Ask = Callback<(ConfirmRequest, Reply)>;

thread_local! {
    static HOST: RefCell<Option<Ask>> = const { RefCell::new(None) };
}

/// Puts `request` to the [`ConfirmHost`] and resolves with the answer, so
/// any code can `if ask(request).await { .. }`. Without a host (none is
/// mounted), it is `false` at once.
pub fn ask(request: ConfirmRequest) -> Answer {
    let reply = Reply::default();
    match HOST.with_borrow(Clone::clone) {
        Some(host) => host.emit((request, reply.clone())),
        None => reply.send(false),
    }
    Answer(reply)
}

/// Shows the questions [`ask`] puts, one at a time in a [`Confirm`]: render
/// it once, near the root of the app. A question put while another shows
/// replaces it, the first answered `false`.
#[function_component]
pub fn ConfirmHost() -> Html {
    let phrases = use_phrases_with(&L10N);
    let shown = use_state(|| None::<ConfirmRequest>);
    let showing = use_mut_ref(Showing::default);

    {
        let (shown, showing) = (shown.setter(), showing.clone());
        use_effect_with((), move |()| {
            let (set, held) = (shown, showing.clone());
            HOST.set(Some(Callback::from(
                move |(request, reply): (ConfirmRequest, Reply)| {
                    held.borrow_mut().replace(reply);
                    set.set(Some(request));
                },
            )));
            move || {
                HOST.set(None);
                showing.borrow_mut().resolve(false);
            }
        });
    }

    let answer = |value: bool| {
        let (shown, showing) = (shown.setter(), showing.clone());
        Callback::from(move |()| {
            showing.borrow_mut().resolve(value);
            shown.set(None);
        })
    };
    let request = (*shown).clone();

    html! {
        <Confirm open={request.is_some()}
            title={request.as_ref().map(|r| r.title.clone()).unwrap_or_default()}
            explanation={request.as_ref().map(|r| r.explanation.clone()).unwrap_or_default()}
            details={request.as_ref().map(|r| r.details.clone()).unwrap_or_default()}
            accept={label(&phrases, &request.as_ref().and_then(|r| r.accept.clone()),
                "sockel-confirm-accept")}
            cancel_label={request.as_ref().and_then(|r| r.cancel_label.clone())}
            icon={request.as_ref().and_then(|r| r.icon.clone())}
            danger={request.as_ref().is_none_or(|r| r.danger)}
            on_accept={answer(true)} on_cancel={answer(false)} testid="confirm-dialog" />
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn a_part_is_away_as_a_modifier() {
        assert_eq!(
            names(confirm_inline(class::CONFIRM_IDLE, false)),
            [class::CONFIRM_IDLE]
        );
        assert_eq!(
            names(confirm_inline(class::CONFIRM_ASK, true)),
            [class::CONFIRM_ASK, class::AWAY]
        );
    }

    #[test]
    fn danger_picks_the_accepting_variant() {
        assert_eq!(accept_variant(true), ButtonVariant::Danger);
        assert_eq!(accept_variant(false), ButtonVariant::Primary);
    }

    #[test]
    fn escape_cancels_unless_it_is_busy() {
        assert!(dismissal(false).allows(Dismiss::Escape));
        assert!(!dismissal(true).allows(Dismiss::Escape));
    }

    fn run(answer: Answer) -> Option<bool> {
        let mut answer = Box::pin(answer);
        let waker = futures::task::noop_waker();
        match answer.as_mut().poll(&mut Context::from_waker(&waker)) {
            Poll::Ready(value) => Some(value),
            Poll::Pending => None,
        }
    }

    #[test]
    fn the_host_resolves_confirm_and_cancel() {
        let mut showing = Showing::default();
        for (value, expected) in [(true, true), (false, false)] {
            let reply = Reply::default();
            let answer = Answer(reply.clone());
            showing.replace(reply);
            assert_eq!(run(Answer(answer.0.clone())), None);
            showing.resolve(value);
            assert_eq!(run(answer), Some(expected));
        }
    }

    #[test]
    fn a_new_question_cancels_the_one_showing() {
        let mut showing = Showing::default();
        let (first, second) = (Reply::default(), Reply::default());
        showing.replace(first.clone());
        showing.replace(second.clone());
        assert_eq!(run(Answer(first)), Some(false));
        assert_eq!(run(Answer(second.clone())), None);
        showing.resolve(true);
        assert_eq!(run(Answer(second)), Some(true));
    }

    #[test]
    fn the_first_answer_stands() {
        let reply = Reply::default();
        reply.send(true);
        reply.send(false);
        assert_eq!(run(Answer(reply)), Some(true));
    }

    #[test]
    fn without_a_host_it_is_cancelled() {
        assert_eq!(
            run(ask(ConfirmRequest::new("Delete?", "Gone."))),
            Some(false)
        );
    }

    fn render_confirm(open: bool, danger: bool, error: bool) -> String {
        let props = ConfirmProps {
            open,
            title: "Delete list?".into(),
            explanation: "It cannot be undone.".into(),
            details: Html::default(),
            accept: "Delete".into(),
            cancel_label: None,
            icon: None,
            danger,
            busy: false,
            accept_disabled: false,
            error: error.then(|| "It failed.".into()),
            on_accept: Callback::noop(),
            on_cancel: Callback::noop(),
            class: Classes::new(),
            testid: None,
        };
        futures::executor::block_on(
            yew::LocalServerRenderer::<Confirm>::with_props(props)
                .hydratable(false)
                .render(),
        )
    }

    #[test]
    fn it_asks_in_a_dialog_with_its_error_and_both_buttons() {
        assert_eq!(render_confirm(false, true, true), "");
        let html = render_confirm(true, true, true);
        for part in [
            class::CONFIRM,
            class::CONFIRM_TEXT,
            class::DANGER,
            "It failed.",
            ">Cancel<",
        ] {
            assert!(html.contains(part), "{part}: {html}");
        }
        assert!(
            html.contains("confirm-cancel") && html.contains("confirm-accept"),
            "{html}"
        );
        assert!(!render_confirm(true, false, false).contains(class::DANGER));
    }

    #[test]
    fn the_accept_button_waits_for_its_condition() {
        let render = |accept_disabled| {
            futures::executor::block_on(
                yew::LocalServerRenderer::<ConfirmPopover>::with_props(yew::props!(
                    ConfirmPopoverProps {
                        open: true,
                        title: AttrValue::from("Delete board?"),
                        explanation: AttrValue::from("Type its name."),
                        accept: AttrValue::from("Delete"),
                        accept_disabled: accept_disabled,
                        on_accept: Callback::noop(),
                        on_cancel: Callback::noop(),
                    }
                ))
                .hydratable(false)
                .render(),
            )
        };
        let accept = |html: &str| {
            let at = html.find("confirm-accept").expect("accept button");
            let end = at + html[at..].find('>').expect("its tag");
            html[at..end].contains("disabled")
        };
        let on = render(true);
        assert!(!accept(&render(false)));
        assert!(accept(&on), "{on}");
    }

    #[test]
    fn compact_is_a_modifier() {
        assert_eq!(names(confirm_popover(false)), [class::CONFIRM_POPOVER]);
        assert_eq!(
            names(confirm_popover(true)),
            [class::CONFIRM_POPOVER, class::COMPACT]
        );
    }

    fn render_popover(compact: bool) -> String {
        let props = ConfirmPopoverProps {
            anchor: NodeRef::default(),
            anchor_element: None,
            open: true,
            title: "Remove Notes".into(),
            explanation: "Remove".into(),
            details: html! { <strong>{ "Notes" }</strong> },
            accept: "Yes".into(),
            cancel_label: Some("No".into()),
            icon: None,
            danger: true,
            busy: false,
            accept_disabled: false,
            error: None,
            side: Side::Below,
            align: PopoverAlign::Start,
            compact,
            explanation_icon: None,
            on_accept: Callback::noop(),
            on_cancel: Callback::noop(),
            class: Classes::new(),
            testid: None,
        };
        futures::executor::block_on(
            yew::LocalServerRenderer::<ConfirmPopover>::with_props(props)
                .hydratable(false)
                .render(),
        )
    }

    #[test]
    fn compact_asks_on_one_line_with_two_named_icons() {
        let html = render_popover(true);
        for part in [
            class::CONFIRM_LINE,
            class::CONFIRM_PAIR,
            class::CONFIRM_NO,
            class::CONFIRM_YES,
            "aria-label=\"No\"",
            "title=\"Yes\"",
            "<strong>Notes</strong>",
        ] {
            assert!(html.contains(part), "{part}: {html}");
        }
        assert!(!html.contains(class::POPOVER_HEAD), "{html}");
        assert!(!html.contains(">No<"), "{html}");
        let full = render_popover(false);
        assert!(full.contains(class::POPOVER_HEAD), "{full}");
        assert!(!full.contains(class::CONFIRM_PAIR), "{full}");
    }
}
