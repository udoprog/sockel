//! DropZone (`style/_drop.scss`).

use std::cell::Cell;
use std::rc::Rc;

use phrasebook02::yew023::use_phrases_with;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::File;
use yew::prelude::*;

use crate::phrasebookcatalog01::{L10N, label};
use crate::{Button, ButtonSize, class};

/// Why a file was not taken.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// Not one of the types the zone accepts.
    Type,
    /// Larger than `max_size`.
    TooLarge,
}

/// Where a zone is, which its overlay draws.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DropState {
    #[default]
    Idle,
    /// A file is dragged somewhere on the page.
    Dragging,
    /// It is over the zone and would be taken.
    Over,
    /// It is over the zone and would not be taken.
    Refused,
    /// A file taken is being worked on.
    Busy,
    /// The last file was refused or failed.
    Error,
}

/// Whether a file named `name` of media type `kind` (empty when the browser
/// does not know it) is one of `accept`, written as for a file input: media
/// types (`image/png`, `image/*`) and extensions (`.pdf`), separated by
/// commas. An empty `accept` takes anything.
pub fn accepts(accept: &str, name: &str, kind: &str) -> bool {
    if accept.trim().is_empty() {
        return true;
    }

    let name = name.to_ascii_lowercase();
    let kind = kind.to_ascii_lowercase();

    accept
        .split(',')
        .map(|entry| entry.trim().to_ascii_lowercase())
        .any(|entry| {
            if entry.starts_with('.') {
                name.ends_with(&entry)
            } else if let Some(group) = entry.strip_suffix("/*") {
                kind.split_once('/').is_some_and(|(g, _)| g == group)
            } else {
                !entry.is_empty() && entry == kind
            }
        })
}

/// Why a file of `size` bytes would be refused, by type first, then size.
pub fn refusal(
    accept: &str,
    max_size: Option<u64>,
    name: &str,
    kind: &str,
    size: u64,
) -> Option<Refusal> {
    if !accepts(accept, name, kind) {
        Some(Refusal::Type)
    } else if max_size.is_some_and(|max| size > max) {
        Some(Refusal::TooLarge)
    } else {
        None
    }
}

/// A dropped, pasted or picked file, as far as the filter needs.
pub(crate) trait Incoming {
    fn name(&self) -> String;
    fn kind(&self) -> String;
    fn size(&self) -> u64;
}

impl Incoming for File {
    fn name(&self) -> String {
        File::name(self)
    }

    fn kind(&self) -> String {
        self.type_()
    }

    fn size(&self) -> u64 {
        web_sys::Blob::size(self) as u64
    }
}

/// The one way files get in, whether dropped, pasted or picked: the first
/// only unless `multiple`, the ones taken go to `on_files`, and the first
/// refusal to `on_refused`.
pub(crate) fn receive<T: Incoming>(
    accept: &str,
    max_size: Option<u64>,
    multiple: bool,
    mut files: Vec<T>,
    on_files: &Callback<Vec<T>>,
    on_refused: &Callback<Refusal>,
) -> Option<Refusal> {
    if !multiple {
        files.truncate(1);
    }

    let mut first = None;
    let mut taken = Vec::new();

    for file in files {
        match refusal(accept, max_size, &file.name(), &file.kind(), file.size()) {
            None => taken.push(file),
            Some(why) => _ = first.get_or_insert(why),
        }
    }

    if !taken.is_empty() {
        on_files.emit(taken);
    }

    if let Some(why) = first {
        on_refused.emit(why);
    }

    first
}

/// Whether a dragged file of media type `kind` is one of `accept`, while it
/// is dragged and its name is not known yet: `None` when that cannot be
/// told, since the type is unknown or `accept` names extensions only.
fn accepts_kind(accept: &str, kind: &str) -> Option<bool> {
    let by_type = accept.split(',').any(|entry| entry.contains('/'));

    if kind.is_empty() || !by_type {
        return None;
    }

    Some(accepts(accept, "", kind))
}

fn carries_files(event: &web_sys::DragEvent) -> bool {
    event.data_transfer().is_some_and(|transfer| {
        transfer
            .types()
            .iter()
            .any(|t| t.as_string().as_deref() == Some("Files"))
    })
}

fn drag_verdict(accept: &str, event: &web_sys::DragEvent) -> Option<bool> {
    let items = event.data_transfer()?.items();
    let mut verdict = None;

    for i in 0..items.length() {
        let Some(item) = items.get(i) else {
            continue;
        };

        if item.kind() != "file" {
            continue;
        }

        match accepts_kind(accept, &item.type_()) {
            Some(true) => return Some(true),
            Some(false) => verdict = Some(false),
            None => return None,
        }
    }

    verdict
}

fn files_of(list: Option<web_sys::FileList>) -> Vec<File> {
    list.map(|list| (0..list.length()).filter_map(|i| list.get(i)).collect())
        .unwrap_or_default()
}

/// Whether a paste comes from a text field, where it is text.
fn typing(event: &web_sys::Event) -> bool {
    let Some(target) = event
        .target()
        .and_then(|t| t.dyn_into::<web_sys::HtmlElement>().ok())
    else {
        return false;
    };

    matches!(
        target.tag_name().to_ascii_lowercase().as_str(),
        "input" | "textarea" | "select"
    ) || target.is_content_editable()
}

/// A listener on the window, removed when dropped. Not passive, since these
/// listeners cancel the drop's and paste's defaults.
struct WindowListener {
    kind: &'static str,
    closure: Closure<dyn Fn(web_sys::Event)>,
}

impl WindowListener {
    fn new(kind: &'static str, f: impl Fn(web_sys::Event) + 'static) -> Option<Self> {
        let closure = Closure::<dyn Fn(web_sys::Event)>::new(f);
        web_sys::window()?
            .add_event_listener_with_callback(kind, closure.as_ref().unchecked_ref())
            .ok()?;
        Some(Self { kind, closure })
    }
}

impl Drop for WindowListener {
    fn drop(&mut self) {
        if let Some(window) = web_sys::window() {
            _ = window.remove_event_listener_with_callback(
                self.kind,
                self.closure.as_ref().unchecked_ref(),
            );
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct DropZoneProps {
    #[prop_or_default]
    pub class: Classes,
    /// The files taken, as for a file input's `accept`; empty takes anything.
    #[prop_or_default]
    pub accept: AttrValue,
    /// The largest file taken, in bytes.
    #[prop_or_default]
    pub max_size: Option<u64>,
    /// Takes more than the first file of a drop, paste or pick.
    #[prop_or_default]
    pub multiple: bool,
    /// What the overlay says while a file is dragged near.
    #[prop_or_default]
    pub label: Option<AttrValue>,
    /// What it says of a file of a type it does not take.
    #[prop_or_default]
    pub reject: Option<AttrValue>,
    /// What it says of a file over `max_size`.
    #[prop_or_default]
    pub too_large: Option<AttrValue>,
    /// Set while a file taken is being worked on: what the overlay says
    /// meanwhile. The zone takes no other file then.
    #[prop_or_default]
    pub busy: Option<AttrValue>,
    /// What went wrong with the last file, said by the overlay until the
    /// app clears it.
    #[prop_or_default]
    pub error: Option<AttrValue>,
    /// Takes nothing and shows no overlay.
    #[prop_or_default]
    pub disabled: bool,
    /// Takes files pasted outside a text field as if dropped.
    #[prop_or_default]
    pub paste: bool,
    /// The zone may be taller than the screen: its label, larger, is kept
    /// on screen while the zone scrolls (`.plinth-tall` on the overlay and
    /// the label).
    #[prop_or_default]
    pub tall: bool,
    /// Shows a button with this label, which opens the file picker.
    #[prop_or_default]
    pub pick: Option<AttrValue>,
    /// The files taken, in order, from a drop, paste or pick.
    pub on_files: Callback<Vec<File>>,
    /// A file was refused, and why.
    #[prop_or_default]
    pub on_refused: Callback<Refusal>,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
    /// What the zone holds.
    #[prop_or_default]
    pub children: Html,
}

/// A zone's classes: `drop-zone`, its state and `disabled`.
pub fn drop_zone(state: DropState, disabled: bool) -> Classes {
    let state = match state {
        DropState::Idle => None,
        DropState::Dragging => Some(class::DRAGGING),
        DropState::Over => Some(class::OVER),
        DropState::Refused => Some(class::REFUSED),
        DropState::Busy => Some(class::BUSY),
        DropState::Error => Some(class::ERROR),
    };
    classes!(class::DROP_ZONE, state, disabled.then_some(class::DISABLED))
}

#[derive(Clone, Copy, PartialEq)]
enum Over {
    No,
    Yes,
    Refused,
}

/// A target for files dropped on it, pasted, or picked with its button.
/// While a file is dragged over the page it is outlined, over it tinted,
/// and the state's message, a refusal, a busy or an error is drawn by an
/// overlay inside it: the zone has a border of its own at all times and the
/// overlay takes no room, so no state or message changes its size. A drag
/// that carries no files passes through untouched.
#[function_component]
pub fn DropZone(props: &DropZoneProps) -> Html {
    let phrases = use_phrases_with(&L10N);
    let drop_label = label(&phrases, &props.label, "plinth-drop-zone-drop");
    let reject = label(&phrases, &props.reject, "plinth-drop-zone-reject");
    let page_drag = use_state_eq(|| false);
    let over = use_state_eq(|| Over::No);
    let refused = use_state_eq(|| None::<Refusal>);
    let zone = use_node_ref();
    let input = use_node_ref();
    let idle = props.disabled || props.busy.is_some();

    let take = {
        let accept = props.accept.clone();
        let max_size = props.max_size;
        let multiple = props.multiple;
        let on_files = props.on_files.clone();
        let on_refused = props.on_refused.clone();
        let refused = refused.clone();

        Callback::from(move |files: Vec<File>| {
            refused.set(receive(
                &accept,
                max_size,
                multiple,
                files,
                &on_files,
                &on_refused,
            ));
        })
    };

    {
        let page_drag = page_drag.clone();
        let over = over.clone();
        let take = take.clone();

        use_effect_with(
            (props.disabled, props.paste, idle),
            move |&(disabled, paste, idle)| {
                let mut listeners = Vec::new();

                if !disabled {
                    let depth = Rc::new(Cell::new(0_i32));

                    listeners.extend(WindowListener::new("dragenter", {
                        let depth = depth.clone();
                        let page_drag = page_drag.clone();
                        move |event| {
                            if let Some(event) = event.dyn_ref::<web_sys::DragEvent>()
                                && carries_files(event)
                            {
                                depth.set(depth.get() + 1);
                                page_drag.set(true);
                            }
                        }
                    }));

                    listeners.extend(WindowListener::new("dragleave", {
                        let depth = depth.clone();
                        let page_drag = page_drag.clone();
                        let over = over.clone();
                        move |event| {
                            if let Some(event) = event.dyn_ref::<web_sys::DragEvent>()
                                && carries_files(event)
                            {
                                depth.set((depth.get() - 1).max(0));

                                if depth.get() == 0 {
                                    page_drag.set(false);
                                    over.set(Over::No);
                                }
                            }
                        }
                    }));

                    // Dropped elsewhere, a file would replace the page with itself.
                    listeners.extend(WindowListener::new("dragover", |event| {
                        if let Some(event) = event.dyn_ref::<web_sys::DragEvent>()
                            && !event.default_prevented()
                            && carries_files(event)
                        {
                            event.prevent_default();

                            if let Some(transfer) = event.data_transfer() {
                                transfer.set_drop_effect("none");
                            }
                        }
                    }));

                    for kind in ["drop", "dragend"] {
                        let depth = depth.clone();
                        let page_drag = page_drag.clone();
                        let over = over.clone();

                        listeners.extend(WindowListener::new(kind, move |event| {
                            if kind == "drop"
                                && let Some(event) = event.dyn_ref::<web_sys::DragEvent>()
                                && !event.default_prevented()
                                && carries_files(event)
                            {
                                event.prevent_default();
                            }

                            depth.set(0);
                            page_drag.set(false);
                            over.set(Over::No);
                        }));
                    }

                    if paste && !idle {
                        listeners.extend(WindowListener::new("paste", move |event| {
                            if typing(&event) {
                                return;
                            }

                            let files = files_of(
                                event
                                    .dyn_ref::<web_sys::ClipboardEvent>()
                                    .and_then(|event| event.clipboard_data())
                                    .and_then(|data| data.files()),
                            );

                            if !files.is_empty() {
                                event.prevent_default();
                                take.emit(files);
                            }
                        }));
                    }
                }

                move || drop(listeners)
            },
        );
    }

    let ondragenter = {
        let over = over.clone();
        let refused = refused.clone();
        let accept = props.accept.clone();

        Callback::from(move |event: DragEvent| {
            if !carries_files(&event) {
                return;
            }

            event.prevent_default();
            refused.set(None);
            over.set(match drag_verdict(&accept, &event) {
                Some(false) => Over::Refused,
                _ => Over::Yes,
            });
        })
    };

    let ondragover = {
        let over = over.clone();
        let accept = props.accept.clone();

        Callback::from(move |event: DragEvent| {
            if !carries_files(&event) {
                return;
            }

            event.prevent_default();
            let verdict = drag_verdict(&accept, &event);

            if let Some(transfer) = event.data_transfer() {
                let effect = if idle || verdict == Some(false) {
                    "none"
                } else {
                    "copy"
                };
                transfer.set_drop_effect(effect);
            }

            over.set(match verdict {
                Some(false) => Over::Refused,
                _ => Over::Yes,
            });
        })
    };

    let ondragleave = {
        let over = over.clone();
        let zone = zone.clone();

        Callback::from(move |event: DragEvent| {
            // Into a child of the zone is still over it.
            let inside = event
                .related_target()
                .and_then(|target| target.dyn_into::<web_sys::Node>().ok())
                .zip(zone.cast::<web_sys::Node>())
                .is_some_and(|(target, zone)| zone.contains(Some(&target)));

            if !inside {
                over.set(Over::No);
            }
        })
    };

    let ondrop = {
        let over = over.clone();
        let page_drag = page_drag.clone();
        let take = take.clone();

        Callback::from(move |event: DragEvent| {
            if !carries_files(&event) {
                return;
            }

            event.prevent_default();
            over.set(Over::No);
            page_drag.set(false);

            if !idle {
                take.emit(files_of(event.data_transfer().and_then(|t| t.files())));
            }
        })
    };

    let onpointerdown = {
        let refused = refused.clone();
        Callback::from(move |_: PointerEvent| refused.set(None))
    };

    let onpick = {
        let input = input.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(input) = input.cast::<web_sys::HtmlInputElement>() {
                input.click();
            }
        })
    };

    let onchange = {
        let take = take.clone();
        Callback::from(move |event: Event| {
            let input = event.target_unchecked_into::<web_sys::HtmlInputElement>();
            take.emit(files_of(input.files()));
            // The same file can then be picked again.
            input.set_value("");
        })
    };

    let problem = props.error.clone().or_else(|| {
        refused.map(|why| match why {
            Refusal::Type => reject.clone(),
            Refusal::TooLarge => label(&phrases, &props.too_large, "plinth-drop-zone-too-large"),
        })
    });

    let (state, text) = if props.disabled {
        (DropState::Idle, None)
    } else if let Some(busy) = &props.busy {
        (DropState::Busy, Some(busy.clone()))
    } else if let Some(problem) = problem {
        (DropState::Error, Some(problem))
    } else {
        match (*over, *page_drag) {
            (Over::Refused, _) => (DropState::Refused, Some(reject)),
            (Over::Yes, _) => (DropState::Over, Some(drop_label.clone())),
            (Over::No, true) => (DropState::Dragging, Some(drop_label)),
            (Over::No, false) => (DropState::Idle, None),
        }
    };
    let alert = matches!(state, DropState::Error);
    let tall = props.tall.then_some(class::TALL);

    html! {
        <div ref={zone} class={classes!(drop_zone(state, props.disabled), props.class.clone())}
            {ondragenter} {ondragover} {ondragleave} {ondrop} {onpointerdown}
            data-testid={props.testid.clone()}>
            { props.children.clone() }
            if let Some(pick) = &props.pick {
                <div class={class::DROP_ZONE_PICK}>
                    <Button size={ButtonSize::Small} disabled={props.disabled || props.busy.is_some()}
                        onclick={onpick}>
                        { pick.clone() }
                    </Button>
                </div>
                <input type="file" hidden=true accept={props.accept.clone()}
                    multiple={props.multiple} {onchange} ref={input} />
            }
            if let Some(text) = text {
                <div class={classes!(class::DROP_ZONE_OVERLAY, tall)}
                    role={if alert { "alert" } else { "status" }}
                    aria-hidden={(!alert && state != DropState::Busy).then_some("true")}>
                    <span class={classes!(class::DROP_ZONE_LABEL, tall,
                        (state == DropState::Busy).then_some(class::BUSY))}>{ text }</span>
                </div>
            }
        </div>
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    const IMAGES: &str = "image/png,image/jpeg,image/webp";
    const DOCS: &str = ".pdf, .TXT";

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn images_by_type() {
        assert!(accepts(IMAGES, "a.png", "image/png"));
        assert!(accepts(IMAGES, "", "image/webp"));
        assert!(accepts(IMAGES, "x", "IMAGE/JPEG"));
        assert!(!accepts(IMAGES, "a.gif", "image/gif"));
        assert!(!accepts(IMAGES, "a.png", ""));
        assert!(accepts("image/*", "a.gif", "image/gif"));
        assert!(!accepts("image/*", "a.txt", "text/plain"));
    }

    #[test]
    fn documents_by_extension() {
        assert!(accepts(DOCS, "List.PDF", ""));
        assert!(accepts(DOCS, "notes.txt", "text/plain"));
        assert!(!accepts(DOCS, "notes.txt.zip", "application/zip"));
        assert!(accepts(".pdf,image/*", "a.png", "image/png"));
    }

    #[test]
    fn empty_accept_takes_anything() {
        assert!(accepts("", "a.bin", ""));
        assert!(accepts("  ", "a.bin", "application/octet-stream"));
    }

    #[test]
    fn size_is_checked_after_type() {
        assert_eq!(refusal(IMAGES, Some(10), "a.png", "image/png", 10), None);
        assert_eq!(
            refusal(IMAGES, Some(10), "a.png", "image/png", 11),
            Some(Refusal::TooLarge)
        );
        assert_eq!(
            refusal(IMAGES, Some(10), "a.gif", "image/gif", 99),
            Some(Refusal::Type)
        );
        assert_eq!(refusal(IMAGES, None, "a.png", "image/png", u64::MAX), None);
    }

    #[test]
    fn while_dragging() {
        assert_eq!(accepts_kind(IMAGES, "image/png"), Some(true));
        assert_eq!(accepts_kind(IMAGES, "application/pdf"), Some(false));
        assert_eq!(accepts_kind(IMAGES, ""), None);
        assert_eq!(accepts_kind(DOCS, "application/zip"), None);
    }

    #[derive(Clone, Debug, PartialEq)]
    struct Fake(&'static str, &'static str, u64);

    impl Incoming for Fake {
        fn name(&self) -> String {
            self.0.into()
        }

        fn kind(&self) -> String {
            self.1.into()
        }

        fn size(&self) -> u64 {
            self.2
        }
    }

    /// What `receive` hands on: the files taken, then the refusals.
    fn run(multiple: bool, files: Vec<Fake>) -> (Vec<Vec<Fake>>, Vec<Refusal>, Option<Refusal>) {
        let taken = Rc::new(RefCell::new(Vec::new()));
        let refused = Rc::new(RefCell::new(Vec::new()));
        let on_files = {
            let taken = taken.clone();
            Callback::from(move |files| taken.borrow_mut().push(files))
        };
        let on_refused = {
            let refused = refused.clone();
            Callback::from(move |why| refused.borrow_mut().push(why))
        };
        let first = receive(IMAGES, Some(100), multiple, files, &on_files, &on_refused);
        let (taken, refused) = (taken.borrow().clone(), refused.borrow().clone());
        (taken, refused, first)
    }

    #[test]
    fn dropped_and_pasted_files_take_the_same_route() {
        let png = Fake("a.png", "image/png", 5);
        let gif = Fake("b.gif", "image/gif", 5);
        let big = Fake("c.png", "image/png", 500);

        // The drop, paste and pick handlers all call `take`, which is
        // `receive`: the same files give the same result whichever way in.
        let (taken, refused, first) = run(true, vec![png.clone(), gif, big]);
        assert_eq!(taken, [vec![png.clone()]]);
        assert_eq!(refused, [Refusal::Type]);
        assert_eq!(first, Some(Refusal::Type));

        let (taken, refused, first) = run(true, vec![png.clone()]);
        assert_eq!(taken, [vec![png]]);
        assert!(refused.is_empty());
        assert_eq!(first, None);
    }

    #[test]
    fn one_file_unless_multiple() {
        let a = Fake("a.png", "image/png", 5);
        let b = Fake("b.png", "image/png", 5);

        assert_eq!(run(false, vec![a.clone(), b.clone()]).0, [vec![a.clone()]]);
        assert_eq!(run(true, vec![a.clone(), b.clone()]).0, [vec![a, b]]);
    }

    #[test]
    fn nothing_taken_emits_only_the_refusal() {
        let (taken, refused, _) = run(true, vec![Fake("a.png", "image/png", 500)]);
        assert!(taken.is_empty());
        assert_eq!(refused, [Refusal::TooLarge]);
    }

    #[test]
    fn state_is_a_modifier() {
        assert_eq!(names(drop_zone(DropState::Idle, false)), [class::DROP_ZONE]);
        for (state, modifier) in [
            (DropState::Dragging, class::DRAGGING),
            (DropState::Over, class::OVER),
            (DropState::Refused, class::REFUSED),
            (DropState::Busy, class::BUSY),
            (DropState::Error, class::ERROR),
        ] {
            assert_eq!(names(drop_zone(state, false)), [class::DROP_ZONE, modifier]);
        }
    }

    #[test]
    fn disabled_is_a_modifier() {
        assert_eq!(
            names(drop_zone(DropState::Idle, true)),
            [class::DROP_ZONE, class::DISABLED]
        );
    }

    fn render(props: DropZoneProps) -> String {
        futures::executor::block_on(
            yew::LocalServerRenderer::<DropZone>::with_props(props)
                .hydratable(false)
                .render(),
        )
    }

    #[test]
    fn tall_and_busy_mark_the_overlay_and_label() {
        let busy = || Some(AttrValue::Static("Saving"));
        let html = render(yew::props!(DropZoneProps {
            tall: true,
            busy: busy(),
            on_files: Callback::noop(),
        }));
        let overlay = format!("{} {}", class::DROP_ZONE_OVERLAY, class::TALL);
        let label = format!("{} {} {}", class::DROP_ZONE_LABEL, class::TALL, class::BUSY);
        assert!(html.contains(&overlay), "{html}");
        assert!(html.contains(&label), "{html}");

        let html = render(yew::props!(DropZoneProps {
            busy: busy(),
            on_files: Callback::noop(),
        }));
        assert!(!html.contains(class::TALL), "{html}");
    }
}
