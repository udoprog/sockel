//! Shortcuts: keys an app binds with [`use_shortcut`], scoped by the active
//! layer ([`use_shortcut_layer`]) and skipped while typing, listed by a help
//! dialog made from the same registry ([`ShortcutHelp`]), and drawn as
//! keycaps by [`Kbd`] (`style/_shortcuts.scss`).
//!
//! The registry is the page's, not a context's: the page has one keyboard,
//! so one listener on the document serves every shortcut.

use crate::testid::with_testid;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::OnceLock;

use phrasebook02::Phrases;
use phrasebook02::yew023::use_phrases_with;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::KeyboardEvent;
use yew::prelude::*;

use crate::phrasebookcatalog01::{L10N, label};
use crate::timer::Timer;
use crate::{Dialog, Dismiss, Overlays, class, use_overlays};

/// A key and the modifiers held with it, as `KeyboardEvent.key` names the
/// key.
///
/// [`Chord::parse`] reads it from text: `"?"`, `"n"`, `"S"` (Shift and s),
/// `"Mod+Enter"` (Ctrl, or Cmd on Apple platforms), `"Ctrl+Shift+k"`,
/// `"Escape"`, `"ArrowDown"`, `"Space"`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Chord {
    /// The key, a letter in lower case.
    pub key: String,
    /// Ctrl, or Cmd on Apple platforms: the platform's command key.
    pub primary: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

/// The key's name in a chord's text, as `KeyboardEvent.key` has it.
fn key_name(word: &str) -> String {
    match word.to_ascii_lowercase().as_str() {
        "esc" | "escape" => "Escape".into(),
        "space" | "spacebar" => " ".into(),
        "enter" | "return" => "Enter".into(),
        "tab" => "Tab".into(),
        "up" | "arrowup" => "ArrowUp".into(),
        "down" | "arrowdown" => "ArrowDown".into(),
        "left" | "arrowleft" => "ArrowLeft".into(),
        "right" | "arrowright" => "ArrowRight".into(),
        "home" => "Home".into(),
        "end" => "End".into(),
        "pageup" => "PageUp".into(),
        "pagedown" => "PageDown".into(),
        "backspace" => "Backspace".into(),
        "del" | "delete" => "Delete".into(),
        _ => word.into(),
    }
}

impl Chord {
    /// Reads a chord from text: modifiers and the key joined by `+`
    /// (`"Ctrl++"` is Ctrl and plus). A capital letter alone is Shift and
    /// that letter.
    pub fn parse(text: &str) -> Self {
        let mut chord = Chord::default();
        let (mods, key) = match text.strip_suffix("++") {
            Some(mods) => (mods, "+"),
            None => match text.rsplit_once('+') {
                Some((mods, key)) if !key.is_empty() => (mods, key),
                _ => ("", text),
            },
        };
        for part in mods.split('+').filter(|p| !p.is_empty()) {
            match part.to_ascii_lowercase().as_str() {
                "mod" | "primary" => chord.primary = true,
                "ctrl" | "control" => chord.ctrl = true,
                "alt" | "option" | "opt" => chord.alt = true,
                "shift" => chord.shift = true,
                "meta" | "cmd" | "command" | "super" => chord.meta = true,
                _ => {}
            }
        }
        let mut chars = key.chars();
        chord.key = match (chars.next(), chars.next()) {
            (Some(c), None) if c.is_alphabetic() => {
                if c.is_uppercase() {
                    chord.shift = true;
                }
                c.to_lowercase().collect()
            }
            (Some(_), None) => key.into(),
            _ => key_name(key),
        };
        chord
    }

    /// Whether the key is one printable character other than a letter
    /// (`?`, `!`, `/`, `1`), which Shift may be needed to type.
    fn symbol(&self) -> bool {
        let mut chars = self.key.chars();
        matches!((chars.next(), chars.next()), (Some(c), None) if !c.is_alphabetic() && c != ' ')
    }

    /// Whether a press of `key` with these modifiers is this chord; `apple`
    /// takes the primary modifier for Cmd rather than Ctrl. Shift is not
    /// asked of a symbol unless the chord names it.
    pub fn matches_parts(
        &self,
        key: &str,
        ctrl: bool,
        alt: bool,
        shift: bool,
        meta: bool,
        apple: bool,
    ) -> bool {
        let want_ctrl = self.ctrl || (self.primary && !apple);
        let want_meta = self.meta || (self.primary && apple);
        let mut chars = key.chars();
        let pressed: String = match (chars.next(), chars.next()) {
            (Some(c), None) => c.to_lowercase().collect(),
            _ => key.into(),
        };
        ctrl == want_ctrl
            && meta == want_meta
            && alt == self.alt
            && (shift == self.shift || (self.symbol() && !self.shift))
            && pressed == self.key
    }

    /// Whether `e` is this chord, on this platform.
    pub fn matches(&self, e: &KeyboardEvent) -> bool {
        self.matches_parts(
            &e.key(),
            e.ctrl_key(),
            e.alt_key(),
            e.shift_key(),
            e.meta_key(),
            is_apple(),
        )
    }

    /// The chord as text [`Chord::parse`] reads back the same: `"mod+enter"`.
    pub fn text(&self) -> String {
        let mut text = String::new();
        for (on, name) in [
            (self.primary, "mod+"),
            (self.ctrl, "ctrl+"),
            (self.alt, "alt+"),
            (self.shift, "shift+"),
            (self.meta, "meta+"),
        ] {
            if on {
                text.push_str(name);
            }
        }
        text.push_str(match self.key.as_str() {
            " " => "space",
            "+" if !text.is_empty() => "+",
            key => key,
        });
        text
    }

    /// The chord as `aria-keyshortcuts` names it: `"Control+Shift+K"`.
    pub fn aria(&self, apple: bool) -> String {
        let mut parts: Vec<String> = Vec::new();
        if self.ctrl || (self.primary && !apple) {
            parts.push("Control".into());
        }
        if self.alt {
            parts.push("Alt".into());
        }
        if self.shift {
            parts.push("Shift".into());
        }
        if self.meta || (self.primary && apple) {
            parts.push("Meta".into());
        }
        parts.push(match self.key.as_str() {
            " " => "Space".into(),
            key if key.chars().count() == 1 => key.to_uppercase(),
            key => key.into(),
        });
        parts.join("+")
    }

    /// The keycaps the chord is drawn as, in order: Apple's symbols there,
    /// the catalog's words elsewhere. A capital letter stands for Shift
    /// with it when nothing else is held.
    fn caps(&self, phrases: &Phrases, apple: bool) -> Vec<String> {
        let letter = self.key.chars().count() == 1 && self.key.chars().all(char::is_alphabetic);
        let lone_shift = letter && !self.ctrl && !self.alt && !self.meta && !self.primary;
        let text = |id: &str| phrases.text(id).to_string();
        let mut caps = Vec::new();
        if apple {
            if self.ctrl {
                caps.push("⌃".to_owned());
            }
            if self.alt {
                caps.push("⌥".to_owned());
            }
            if self.shift && !lone_shift {
                caps.push("⇧".to_owned());
            }
            if self.meta || self.primary {
                caps.push("⌘".to_owned());
            }
        } else {
            if self.ctrl || self.primary {
                caps.push(text("sockel-key-ctrl"));
            }
            if self.alt {
                caps.push(text("sockel-key-alt"));
            }
            if self.shift && !lone_shift {
                caps.push(text("sockel-key-shift"));
            }
            if self.meta {
                caps.push(text("sockel-key-meta"));
            }
        }
        caps.push(match self.key.as_str() {
            "ArrowUp" => "↑".to_owned(),
            "ArrowDown" => "↓".to_owned(),
            "ArrowLeft" => "←".to_owned(),
            "ArrowRight" => "→".to_owned(),
            " " => text("sockel-key-space"),
            "Enter" => text("sockel-key-enter"),
            "Escape" => text("sockel-key-escape"),
            "Tab" => text("sockel-key-tab"),
            "Backspace" => text("sockel-key-backspace"),
            "Delete" => text("sockel-key-delete"),
            "Home" => text("sockel-key-home"),
            "End" => text("sockel-key-end"),
            "PageUp" => text("sockel-key-page-up"),
            "PageDown" => text("sockel-key-page-down"),
            key if letter && self.shift => key.to_uppercase(),
            key => key.to_owned(),
        });
        caps
    }
}

impl From<&str> for Chord {
    fn from(text: &str) -> Self {
        Chord::parse(text)
    }
}

/// Whether the page runs on an Apple platform, where the primary modifier
/// is Cmd and keys are drawn as symbols; read once.
pub fn is_apple() -> bool {
    static APPLE: OnceLock<bool> = OnceLock::new();
    if !cfg!(target_arch = "wasm32") {
        return false;
    }
    *APPLE.get_or_init(|| {
        let Some(navigator) = web_sys::window().map(|w| w.navigator()) else {
            return false;
        };
        let platform = navigator.platform().unwrap_or_default();
        let platform = if platform.is_empty() {
            navigator.user_agent().unwrap_or_default()
        } else {
            platform
        };
        let platform = platform.to_ascii_lowercase();
        ["mac", "iphone", "ipad", "ipod"]
            .iter()
            .any(|name| platform.contains(name))
    })
}

/// A key binding: its chords, what it does (for the help) and where it
/// applies.
#[derive(Clone, Debug, PartialEq)]
pub struct Shortcut {
    /// Any of these presses it; the help shows them all.
    pub chords: Vec<Chord>,
    /// What it does, as the help lists it.
    pub label: AttrValue,
    /// The help's heading it is listed under; ungrouped ones come first.
    pub group: Option<AttrValue>,
    /// The layer it belongs to; `None` is the page's base layer.
    pub layer: Option<AttrValue>,
    /// It applies in every layer (help, a jump to the inbox), though not
    /// while an overlay is open.
    pub always: bool,
    /// It applies while typing in a field too (Mod+Enter, Escape).
    pub in_fields: bool,
    /// Left out of the help.
    pub hidden: bool,
    /// Off, as if not bound, while false.
    pub enabled: bool,
}

impl Shortcut {
    /// A binding of `chord` ([`Chord::parse`]'s text) doing `label`, in the
    /// base layer.
    pub fn new(chord: &str, label: impl Into<AttrValue>) -> Self {
        Self {
            chords: vec![Chord::parse(chord)],
            label: label.into(),
            group: None,
            layer: None,
            always: false,
            in_fields: false,
            hidden: false,
            enabled: true,
        }
    }

    /// Another chord that presses it (`j` or `ArrowDown`).
    pub fn or(mut self, chord: &str) -> Self {
        self.chords.push(Chord::parse(chord));
        self
    }

    pub fn group(mut self, group: impl Into<AttrValue>) -> Self {
        self.group = Some(group.into());
        self
    }

    pub fn layer(mut self, layer: impl Into<AttrValue>) -> Self {
        self.layer = Some(layer.into());
        self
    }

    pub fn always(mut self) -> Self {
        self.always = true;
        self
    }

    pub fn in_fields(mut self) -> Self {
        self.in_fields = true;
        self
    }

    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Whether a shortcut of `layer` (`always` in every one) applies while
/// `active` is the top layer (`None`: the base), with an overlay open or
/// not. An open overlay holds the base layer and `always` ones, and a
/// layer pushed from inside it keeps its own.
pub fn shortcut_applies(
    layer: Option<&str>,
    always: bool,
    active: Option<&str>,
    overlay_open: bool,
) -> bool {
    match layer {
        Some(layer) => active == Some(layer),
        None if always => !overlay_open,
        None => active.is_none() && !overlay_open,
    }
}

struct Entry {
    id: u64,
    shortcut: Shortcut,
    run: Rc<RefCell<Callback<KeyboardEvent>>>,
    overlays: Overlays,
}

#[derive(Default)]
struct Registry {
    next: u64,
    entries: Vec<Entry>,
    layers: Vec<(u64, AttrValue)>,
    watchers: Vec<(u64, Callback<()>)>,
    listener: Option<Closure<dyn Fn(KeyboardEvent)>>,
}

thread_local! {
    static REGISTRY: RefCell<Registry> = RefCell::new(Registry::default());
}

impl Registry {
    fn take_id(&mut self) -> u64 {
        self.next += 1;
        self.next
    }

    fn active(&self) -> Option<&str> {
        self.layers.last().map(|(_, name)| name.as_str())
    }

    /// The newest binding a press applies to.
    fn find(&self, e: &KeyboardEvent, typing: bool) -> Option<Callback<KeyboardEvent>> {
        let active = self.active();
        self.entries
            .iter()
            .rev()
            .find(|entry| {
                let s = &entry.shortcut;
                s.enabled
                    && (s.in_fields || !typing)
                    && shortcut_applies(
                        s.layer.as_deref(),
                        s.always,
                        active,
                        entry.overlays.any_open(),
                    )
                    && s.chords.iter().any(|chord| chord.matches(e))
            })
            .map(|entry| entry.run.borrow().clone())
    }

    /// The bindings that apply now, overlays aside, in the help's order:
    /// the oldest first, one entry per label and chords.
    fn listed(&self) -> Vec<Shortcut> {
        let active = self.active();
        let mut out: Vec<Shortcut> = Vec::new();
        for entry in &self.entries {
            let s = &entry.shortcut;
            if s.enabled
                && !s.hidden
                && shortcut_applies(s.layer.as_deref(), s.always, active, false)
                && !out
                    .iter()
                    .any(|o| o.label == s.label && o.chords == s.chords)
            {
                out.push(s.clone());
            }
        }
        out
    }

    fn watchers(&self) -> Vec<Callback<()>> {
        self.watchers.iter().map(|(_, w)| w.clone()).collect()
    }
}

/// Tells whoever lists the shortcuts that they changed, once the registry
/// is released.
fn changed(watchers: Vec<Callback<()>>) {
    for watcher in watchers {
        watcher.emit(());
    }
}

fn typing(e: &KeyboardEvent) -> bool {
    e.target()
        .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
        .is_some_and(|t| {
            matches!(t.tag_name().as_str(), "INPUT" | "TEXTAREA" | "SELECT")
                || t.closest("[contenteditable]").ok().flatten().is_some()
        })
}

fn on_key(e: KeyboardEvent) {
    if e.default_prevented() || e.is_composing() {
        return;
    }
    let typing = typing(&e);
    let found = REGISTRY.with_borrow(|r| r.find(&e, typing));
    if let Some(run) = found {
        e.prevent_default();
        run.emit(e);
    }
}

fn document() -> Option<web_sys::Document> {
    web_sys::window()?.document()
}

fn register(
    shortcut: Shortcut,
    run: Rc<RefCell<Callback<KeyboardEvent>>>,
    overlays: Overlays,
) -> u64 {
    let (id, watchers) = REGISTRY.with_borrow_mut(|r| {
        let id = r.take_id();
        r.entries.push(Entry {
            id,
            shortcut,
            run,
            overlays,
        });
        if r.listener.is_none()
            && let Some(document) = document()
        {
            let listener = Closure::<dyn Fn(KeyboardEvent)>::new(on_key);
            if document
                .add_event_listener_with_callback("keydown", listener.as_ref().unchecked_ref())
                .is_ok()
            {
                r.listener = Some(listener);
            }
        }
        (id, r.watchers())
    });
    changed(watchers);
    id
}

fn unregister(id: u64) {
    let watchers = REGISTRY.try_with(|r| {
        let mut r = r.borrow_mut();
        r.entries.retain(|e| e.id != id);
        if r.entries.is_empty()
            && let Some(listener) = r.listener.take()
            && let Some(document) = document()
        {
            _ = document
                .remove_event_listener_with_callback("keydown", listener.as_ref().unchecked_ref());
        }
        r.watchers()
    });
    changed(watchers.unwrap_or_default());
}

/// Binds `shortcut` to `run` while the calling component is mounted. The
/// newest `run` is the one called, and the newest binding of a chord wins
/// where several apply. A press it takes has its default prevented; one
/// already prevented (a component handled it) or composing is left alone.
#[hook]
pub fn use_shortcut(shortcut: Shortcut, run: Callback<KeyboardEvent>) {
    let latest = use_memo((), |()| Rc::new(RefCell::new(Callback::noop())));
    *latest.borrow_mut() = run;
    let overlays = use_overlays();
    use_effect_with(shortcut, move |shortcut| {
        let id = register(shortcut.clone(), (*latest).clone(), overlays);
        move || unregister(id)
    });
}

/// Makes `name` the active layer while the calling component is mounted
/// and `active`: only its shortcuts, and `always` ones, apply until it is
/// gone. Layers stack; the newest is active.
#[hook]
pub fn use_shortcut_layer(name: AttrValue, active: bool) {
    use_effect_with((name, active), |(name, active)| {
        let pushed = active.then(|| {
            let (id, watchers) = REGISTRY.with_borrow_mut(|r| {
                let id = r.take_id();
                r.layers.push((id, name.clone()));
                (id, r.watchers())
            });
            changed(watchers);
            id
        });
        move || {
            if let Some(id) = pushed {
                let watchers = REGISTRY.try_with(|r| {
                    let mut r = r.borrow_mut();
                    r.layers.retain(|(o, _)| *o != id);
                    r.watchers()
                });
                changed(watchers.unwrap_or_default());
            }
        }
    });
}

/// The shortcuts that apply now (open overlays aside), oldest first, kept
/// current as bindings and layers come and go: what a help lists.
#[hook]
pub fn use_shortcut_list() -> Vec<Shortcut> {
    let list = use_state(|| REGISTRY.with_borrow(Registry::listed));
    {
        let list = list.clone();
        use_effect_with((), move |()| {
            let id = REGISTRY.with_borrow_mut(|r| {
                let id = r.take_id();
                let watcher = list.clone();
                r.watchers.push((
                    id,
                    Callback::from(move |()| watcher.set(REGISTRY.with_borrow(Registry::listed))),
                ));
                id
            });
            // Bindings made in the same render as this one register in effects
            // of their own, which a state set here does not see through:
            // read the list again once they have all run.
            let refresh = Timer::start(0, move || {
                list.set(REGISTRY.with_borrow(Registry::listed));
            });
            move || {
                drop(refresh);
                let _ = REGISTRY.try_with(|r| r.borrow_mut().watchers.retain(|(o, _)| *o != id));
            }
        });
    }
    (*list).clone()
}

/// A keycap's classes: `kbd`.
pub fn kbd() -> Classes {
    classes!(class::KBD)
}

#[derive(Properties, PartialEq)]
pub struct KbdProps {
    /// The chord, as [`Chord::parse`] reads it.
    pub keys: AttrValue,
    /// Hidden from assistive technology, for keys drawn inside a control
    /// whose `aria-keyshortcuts` names them.
    #[prop_or_default]
    pub decorative: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// A chord as keycaps (`<kbd>` of `<kbd>`s), each in a box of the line's
/// height: Apple's symbols there (⌘ ⇧), the catalog's words elsewhere
/// (Ctrl, Shift).
#[function_component]
pub fn Kbd(props: &KbdProps) -> Html {
    let phrases = use_phrases_with(&L10N);
    let caps = Chord::parse(&props.keys).caps(&phrases, is_apple());
    with_testid(
        html! {
            <kbd class={classes!(kbd(), props.class.clone())}
                aria-hidden={props.decorative.then_some("true")}>
                { for caps.into_iter().map(|cap| html! { <kbd class={class::KBD_KEY}>{ cap }</kbd> }) }
            </kbd>
        },
        props.testid.clone(),
    )
}

#[derive(Properties, PartialEq)]
pub struct ShortcutHelpProps {
    pub open: bool,
    /// Asked to close it (the owner sets `open` to false).
    pub on_dismiss: Callback<Dismiss>,
    /// Its title, else the catalog's.
    #[prop_or_default]
    pub title: Option<AttrValue>,
    /// What it says with no shortcuts, else the catalog's.
    #[prop_or_default]
    pub empty_label: Option<AttrValue>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// The shortcuts grouped by their `group`, ungrouped first.
fn grouped(list: Vec<Shortcut>) -> Vec<(Option<AttrValue>, Vec<Shortcut>)> {
    let mut groups: Vec<(Option<AttrValue>, Vec<Shortcut>)> = Vec::new();
    for s in list {
        match groups.iter_mut().find(|(g, _)| *g == s.group) {
            Some((_, items)) => items.push(s),
            None => groups.push((s.group.clone(), vec![s])),
        }
    }
    groups.sort_by_key(|(g, _)| g.is_some());
    groups
}

/// A dialog listing the shortcuts that apply where it was opened, made from
/// the registry, so it cannot drift from the keys: each chord beside what it
/// does, under its group's heading.
#[function_component]
pub fn ShortcutHelp(props: &ShortcutHelpProps) -> Html {
    let phrases = use_phrases_with(&L10N);
    let list = use_shortcut_list();
    let title = label(&phrases, &props.title, "sockel-shortcut-help-title");
    let body = if list.is_empty() {
        html! {
            <p class={class::SHORTCUT_HELP_EMPTY}>
                { label(&phrases, &props.empty_label, "sockel-shortcut-help-none") }
            </p>
        }
    } else {
        html! {
            <>{ for grouped(list).into_iter().map(|(group, items)| html! {
                <section class={class::SHORTCUT_HELP_GROUP}>
                    if let Some(group) = group {
                        <h3 class={class::SHORTCUT_HELP_HEADING}>{ group }</h3>
                    }
                    <dl class={class::SHORTCUT_HELP_LIST}>
                        { for items.into_iter().map(|s| html! {
                            <div class={class::SHORTCUT_HELP_ROW}>
                                <dt class={class::SHORTCUT_HELP_KEYS}>
                                    { for s.chords.iter().map(|c| html! {
                                        <Kbd keys={c.text()} />
                                    }) }
                                </dt>
                                <dd class={class::SHORTCUT_HELP_LABEL}>{ s.label.clone() }</dd>
                            </div>
                        }) }
                    </dl>
                </section>
            }) }</>
        }
    };
    html! {
        <Dialog open={props.open} on_dismiss={props.on_dismiss.clone()} {title}
            class={classes!(class::SHORTCUT_HELP, props.class.clone())}
            testid={props.testid.clone()}>
            <div class={class::SHORTCUT_HELP_BODY}>{ body }</div>
        </Dialog>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_chord_reads_its_modifiers_and_key() {
        let c = Chord::parse("Mod+Enter");
        assert!(c.primary && !c.ctrl && c.key == "Enter");
        let c = Chord::parse("S");
        assert!(c.shift && c.key == "s");
        let c = Chord::parse("Ctrl++");
        assert!(c.ctrl && c.key == "+");
        assert_eq!(Chord::parse("+").key, "+");
        assert_eq!(Chord::parse("Esc").key, "Escape");
        assert_eq!(Chord::parse("Space").key, " ");
        assert_eq!(Chord::parse("?").key, "?");
    }

    #[test]
    fn a_press_matches_by_platform_and_shift() {
        let save = Chord::parse("Mod+Enter");
        assert!(save.matches_parts("Enter", true, false, false, false, false));
        assert!(!save.matches_parts("Enter", true, false, false, false, true));
        assert!(save.matches_parts("Enter", false, false, false, true, true));
        let help = Chord::parse("?");
        assert!(
            help.matches_parts("?", false, false, true, false, false),
            "Shift types ?"
        );
        let n = Chord::parse("n");
        assert!(n.matches_parts("n", false, false, false, false, false));
        assert!(
            !n.matches_parts("N", false, false, true, false, false),
            "S is not s"
        );
        let shut = Chord::parse("S");
        assert!(shut.matches_parts("S", false, false, true, false, false));
        assert!(!shut.matches_parts("s", false, false, false, false, false));
        assert!(
            !n.matches_parts("n", true, false, false, false, false),
            "Ctrl+n is not n"
        );
    }

    #[test]
    fn aria_names_the_chord_in_its_own_words() {
        assert_eq!(Chord::parse("Mod+Enter").aria(false), "Control+Enter");
        assert_eq!(Chord::parse("Mod+Enter").aria(true), "Meta+Enter");
        assert_eq!(Chord::parse("S").aria(false), "Shift+S");
        assert_eq!(Chord::parse("Space").aria(false), "Space");
        for text in [
            "Ctrl+Shift+k",
            "Mod+Enter",
            "S",
            "Space",
            "?",
            "Ctrl++",
            "+",
        ] {
            assert_eq!(
                Chord::parse(&Chord::parse(text).text()),
                Chord::parse(text),
                "{text}"
            );
        }
    }

    #[test]
    fn caps_are_symbols_on_apple_and_words_elsewhere() {
        let phrases = Phrases::from_catalog(&L10N);
        assert_eq!(
            Chord::parse("Mod+Enter").caps(&phrases, false),
            ["Ctrl", "Enter"]
        );
        assert_eq!(
            Chord::parse("Mod+Shift+k").caps(&phrases, true),
            ["⇧", "⌘", "K"]
        );
        assert_eq!(Chord::parse("S").caps(&phrases, false), ["S"]);
        assert_eq!(Chord::parse("ArrowDown").caps(&phrases, false), ["↓"]);
        assert_eq!(Chord::parse("Escape").caps(&phrases, false), ["Esc"]);
    }

    #[test]
    fn a_layer_holds_the_base_and_an_overlay_holds_both() {
        assert!(shortcut_applies(None, false, None, false));
        assert!(!shortcut_applies(None, false, Some("dialog"), false));
        assert!(!shortcut_applies(None, false, None, true));
        assert!(shortcut_applies(
            Some("dialog"),
            false,
            Some("dialog"),
            true
        ));
        assert!(!shortcut_applies(Some("dialog"), false, None, false));
        assert!(shortcut_applies(None, true, Some("dialog"), false));
        assert!(!shortcut_applies(None, true, None, true));
    }

    #[test]
    fn the_help_groups_ungrouped_first() {
        let list = vec![
            Shortcut::new("j", "Down").group("Move"),
            Shortcut::new("?", "Help"),
            Shortcut::new("k", "Up").group("Move"),
        ];
        let groups = grouped(list);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].0, None);
        assert_eq!(groups[1].1.len(), 2);
    }

    #[test]
    fn a_kbd_draws_each_key_in_its_own_cap() {
        #[function_component]
        fn Shown() -> Html {
            html! { <Kbd keys="Mod+k" decorative=true class={class::SMALL} /> }
        }
        let html = futures::executor::block_on(yew::LocalServerRenderer::<Shown>::new().render());
        assert!(
            html.contains(&format!("class=\"{} {}\"", class::KBD, class::SMALL)),
            "{html}"
        );
        assert!(html.contains("aria-hidden=\"true\""));
        assert_eq!(html.matches(class::KBD_KEY).count(), 2, "{html}");
    }
}
