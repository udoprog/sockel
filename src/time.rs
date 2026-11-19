//! Time: a moment said relative to now ("3m ago", "43m left") or as a date,
//! in a box of a fixed width that redraws only itself, and only when its
//! text changes (`style/_time.scss`).
//!
//! Every [`Time`] on the page shares one clock: each asks to be woken at the
//! instant its own text next changes, and a single `setTimeout` is kept
//! pointed at the earliest of them. A label reading "2h ago" is woken once an
//! hour, and a page full of them holds one timer.

use crate::testid::with_testid;
use std::cell::RefCell;
use std::collections::BTreeMap;

use phrasebook02::yew023::use_phrases_with;
use phrasebook02::{Args, Phrases};
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use yew::prelude::*;

use crate::class;
use crate::phrasebookcatalog01::L10N;

/// How a [`Time`] says its moment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimeStyle {
    /// How long ago a past moment was: "just now", "3m ago".
    #[default]
    Ago,
    /// How long since a past moment, with no "ago": "5m".
    Since,
    /// How long until a moment: "4m", then "now" once it has come.
    Until,
    /// A countdown: "43m left", down to "0s left".
    Left,
    /// The moment itself, in local time: "2026-11-19 04:02".
    At,
}

/// What a [`Time`] says, before the language words it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TimeText {
    /// The count it takes (`$n`), when any, and a message of the catalog.
    Said(Option<i64>, &'static str),
    /// A duration inside a message: the count, the duration's message and
    /// the message taking its words as `$span`.
    Within(i64, &'static str, &'static str),
}

/// The unit a span of `seconds` is counted in, its size in seconds and the
/// catalog's messages for it, ago and as a bare span.
fn unit(seconds: i64) -> (i64, &'static str, &'static str) {
    match seconds {
        ..60 => (1, "sockel-time-ago-seconds", "sockel-time-span-seconds"),
        60..3600 => (60, "sockel-time-ago-minutes", "sockel-time-span-minutes"),
        3600..86400 => (3600, "sockel-time-ago-hours", "sockel-time-span-hours"),
        _ => (86400, "sockel-time-ago-days", "sockel-time-span-days"),
    }
}

/// Below this many seconds a past moment is "just now".
pub const JUST_NOW: i64 = 5;

/// What `style` says of `at` (seconds since the epoch) at `now`; `None` for
/// [`TimeStyle::At`], which is the date itself.
pub fn time_text(style: TimeStyle, at: i64, now: i64) -> Option<TimeText> {
    let span = |seconds: i64| {
        let seconds = seconds.max(0);
        let (size, _, id) = unit(seconds);
        (seconds / size, id)
    };
    Some(match style {
        TimeStyle::Ago => {
            let delta = (now - at).max(0);
            if delta < JUST_NOW {
                TimeText::Said(None, "sockel-time-just-now")
            } else {
                let (size, id, _) = unit(delta);
                TimeText::Said(Some(delta / size), id)
            }
        }
        TimeStyle::Since => {
            let (n, id) = span(now - at);
            TimeText::Said(Some(n), id)
        }
        TimeStyle::Until if at <= now => TimeText::Said(None, "sockel-time-due"),
        TimeStyle::Until => {
            let (n, id) = span(at - now);
            TimeText::Said(Some(n), id)
        }
        TimeStyle::Left => {
            let (n, id) = span(at - now);
            TimeText::Within(n, id, "sockel-time-left")
        }
        TimeStyle::At => return None,
    })
}

/// The first second after `now` at which `style`'s text for `at` changes,
/// or `None` when it never will again (a countdown that ran out, a date).
///
/// Counting up, the text holds until the next whole unit since `at`;
/// counting down, a count of whole units drops one second after the
/// boundary, since the count floors ("43m" holds from 43m59s to 43m00s).
pub fn time_next(style: TimeStyle, at: i64, now: i64) -> Option<i64> {
    match style {
        TimeStyle::Ago | TimeStyle::Since => {
            let delta = now - at;
            if delta < 0 {
                // A moment still to come reads as "just now" until it is
                // past, then counts up.
                return Some(at);
            }
            if style == TimeStyle::Ago && delta < JUST_NOW {
                return Some(at + JUST_NOW);
            }
            let (size, _, _) = unit(delta);
            Some(at + (delta / size + 1) * size)
        }
        TimeStyle::Until | TimeStyle::Left => {
            let remaining = at - now;
            if remaining <= 0 {
                return None;
            }
            let (size, _, _) = unit(remaining);
            Some(at - (remaining / size) * size + 1)
        }
        TimeStyle::At => None,
    }
}

/// The words of `text` in `phrases`' language.
fn say(phrases: &Phrases, text: &TimeText) -> String {
    match *text {
        TimeText::Said(n, id) => {
            let mut args = Args::new();
            if let Some(n) = n {
                args.set("n", n);
            }
            phrases.msg(id, &args)
        }
        TimeText::Within(n, inner, outer) => {
            let mut args = Args::new();
            args.set("n", n);
            let span = phrases.msg(inner, &args);
            let mut args = Args::new();
            args.set("span", span.as_str());
            phrases.msg(outer, &args)
        }
    }
}

/// The local date and time of `at` (seconds since the epoch) as
/// `YYYY-MM-DD HH:MM`, and as an ISO 8601 instant for `datetime`.
fn local(at: i64) -> (String, String) {
    if !cfg!(target_arch = "wasm32") {
        return (String::new(), String::new());
    }
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(at as f64 * 1000.0));
    let text = format!(
        "{:04}-{:02}-{:02} {:02}:{:02}",
        date.get_full_year(),
        date.get_month() + 1,
        date.get_date(),
        date.get_hours(),
        date.get_minutes()
    );
    let iso = date.to_iso_string().as_string().unwrap_or_default();
    (text, iso)
}

/// Seconds since the epoch, by the browser's clock.
pub fn now_seconds() -> i64 {
    if cfg!(target_arch = "wasm32") {
        (js_sys::Date::now() / 1000.0).floor() as i64
    } else {
        0
    }
}

/// The longest delay `setTimeout` honours (a signed 32-bit count of
/// milliseconds); a wake further out is armed for this long and re-arms.
const MAX_DELAY_MS: f64 = 2_147_483_647.0;

thread_local! {
    static CLOCK: RefCell<Clock> = RefCell::new(Clock::default());
}

#[derive(Default)]
struct Clock {
    next: u64,
    /// Every pending wake: when, in milliseconds since the epoch, and whom.
    wakes: BTreeMap<u64, (f64, Callback<()>)>,
    /// The one armed timeout: the deadline it was armed for and its handle.
    armed: Option<(f64, i32)>,
    fire: Option<Closure<dyn FnMut()>>,
}

impl Clock {
    /// Points the one timeout at the earliest wake, leaving it alone when it
    /// already is.
    fn rearm(&mut self) {
        let earliest = self
            .wakes
            .values()
            .map(|(due, _)| *due)
            .min_by(f64::total_cmp);
        if self.armed.map(|(due, _)| due) == earliest {
            return;
        }
        let Some(window) = web_sys::window() else {
            return;
        };
        if let Some((_, handle)) = self.armed.take() {
            window.clear_timeout_with_handle(handle);
        }
        let Some(due) = earliest else {
            return;
        };
        let fire = self
            .fire
            .get_or_insert_with(|| Closure::<dyn FnMut()>::new(fire));
        let delay = (due - js_sys::Date::now()).clamp(0.0, MAX_DELAY_MS);
        if let Ok(handle) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            fire.as_ref().unchecked_ref(),
            delay as i32,
        ) {
            self.armed = Some((due, handle));
        }
    }
}

/// The timeout went off: wakes whoever is due, after the clock is released
/// (a woken component asks for its next wake), and re-arms for the rest.
fn fire() {
    let due = CLOCK.with_borrow_mut(|clock| {
        clock.armed = None;
        let now = js_sys::Date::now();
        let ids: Vec<u64> = clock
            .wakes
            .iter()
            .filter(|(_, (due, _))| *due <= now)
            .map(|(id, _)| *id)
            .collect();
        let due: Vec<Callback<()>> = ids
            .into_iter()
            .filter_map(|id| clock.wakes.remove(&id))
            .map(|(_, callback)| callback)
            .collect();
        clock.rearm();
        due
    });
    for callback in due {
        callback.emit(());
    }
}

/// A pending wake of the shared clock; dropping it cancels the wake.
struct Wake(u64);

impl Wake {
    fn at(due: i64, callback: Callback<()>) -> Self {
        CLOCK.with_borrow_mut(|clock| {
            let id = clock.next;
            clock.next += 1;
            clock.wakes.insert(id, (due as f64 * 1000.0, callback));
            clock.rearm();
            Wake(id)
        })
    }
}

impl Drop for Wake {
    fn drop(&mut self) {
        let id = self.0;
        // A wake dropped as the page is torn down may outlive the clock.
        let _ = CLOCK.try_with(|clock| {
            let mut clock = clock.borrow_mut();
            if clock.wakes.remove(&id).is_some() {
                clock.rearm();
            }
        });
    }
}

/// Redraws the calling component at `due` (seconds since the epoch), or
/// never for `None`, on the clock every [`Time`] shares. Woken, the
/// component reads the time again and asks for its next instant.
#[hook]
pub fn use_wake_at(due: Option<i64>) {
    let woken = use_state(|| 0u64);
    use_effect_with((due, *woken), move |&(due, count)| {
        let wake = due.map(|due| Wake::at(due, Callback::from(move |()| woken.set(count + 1))));
        move || drop(wake)
    });
}

/// A time's classes: `time`, `short` for a span with no words around it
/// ([`TimeStyle::Since`], [`TimeStyle::Until`]), `wide` for a date, `end`
/// when its text keeps to the box's end.
pub fn time(style: TimeStyle, end: bool) -> Classes {
    classes!(
        class::TIME,
        matches!(style, TimeStyle::Since | TimeStyle::Until).then_some(class::SHORT),
        (style == TimeStyle::At).then_some(class::WIDE),
        end.then_some(class::END)
    )
}

#[derive(Properties, Default, PartialEq)]
pub struct TimeProps {
    /// The moment, in seconds since the epoch.
    pub at: i64,
    #[prop_or_default]
    pub style: TimeStyle,
    /// The time it is now, in seconds since the epoch; the browser's clock
    /// when `None`. Given, it is not woken: the owner redraws it.
    #[prop_or_default]
    pub now: Option<i64>,
    /// The text keeps to the box's end (a column of times, right-aligned).
    #[prop_or_default]
    pub end: bool,
    /// The native tooltip; the moment's local date and time when `None`
    /// (for every style but [`TimeStyle::At`]).
    #[prop_or_default]
    pub title: Option<AttrValue>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// A `<time>` of a fixed width (`--time-width`, `--time-short-width`,
/// `--time-wide-width`) in figures of one width, so neither its ticking nor
/// its unit changing moves what is beside it. It redraws itself, and nothing
/// else, at the instant its text next changes.
#[function_component]
pub fn Time(props: &TimeProps) -> Html {
    let phrases = use_phrases_with(&L10N);
    let now = props.now.unwrap_or_else(now_seconds);
    let next = match props.now {
        Some(_) => None,
        None => time_next(props.style, props.at, now),
    };
    use_wake_at(next);

    let (date, iso) = local(props.at);
    let text = match time_text(props.style, props.at, now) {
        Some(text) => say(&phrases, &text),
        None => date.clone(),
    };
    let title = props.title.clone().or_else(|| {
        (props.style != TimeStyle::At && !date.is_empty()).then(|| AttrValue::from(date))
    });

    with_testid(
        html! {
            <time class={classes!(time(props.style, props.end), props.class.clone())}
                datetime={(!iso.is_empty()).then_some(iso)} {title}>
                { text }
            </time>
        },
        props.testid.clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    fn english(style: TimeStyle, at: i64, now: i64) -> String {
        let phrases = Phrases::from_catalog(&L10N);
        match time_text(style, at, now) {
            Some(text) => say(&phrases, &text),
            None => String::new(),
        }
    }

    /// Fluent isolates placeables; the words without the marks.
    fn plain(text: String) -> String {
        text.replace(['\u{2068}', '\u{2069}'], "")
    }

    #[test]
    fn a_time_names_its_style_and_end() {
        assert_eq!(names(time(TimeStyle::Ago, false)), [class::TIME]);
        assert_eq!(
            names(time(TimeStyle::Until, true)),
            [class::TIME, class::SHORT, class::END]
        );
        assert_eq!(
            names(time(TimeStyle::At, false)),
            [class::TIME, class::WIDE]
        );
    }

    #[test]
    fn each_style_reads_in_english() {
        assert_eq!(plain(english(TimeStyle::Ago, 100, 102)), "just now");
        assert_eq!(plain(english(TimeStyle::Ago, 0, 42)), "42s ago");
        assert_eq!(plain(english(TimeStyle::Ago, 0, 3 * 60 + 5)), "3m ago");
        assert_eq!(plain(english(TimeStyle::Ago, 0, 2 * 3600)), "2h ago");
        assert_eq!(plain(english(TimeStyle::Ago, 0, 3 * 86400)), "3d ago");
        assert_eq!(plain(english(TimeStyle::Since, 0, 300)), "5m");
        assert_eq!(plain(english(TimeStyle::Until, 240, 0)), "4m");
        assert_eq!(plain(english(TimeStyle::Until, 0, 10)), "now");
        assert_eq!(plain(english(TimeStyle::Left, 43 * 60, 0)), "43m left");
        assert_eq!(plain(english(TimeStyle::Left, 0, 10)), "0s left");
    }

    /// The text holds until the instant `time_next` names, and changes at it.
    #[test]
    fn the_next_instant_is_where_the_text_changes() {
        for style in [TimeStyle::Ago, TimeStyle::Since] {
            let at = 1_000_000;
            for now in at..at + 2 * 86_400 + 7 {
                let next = time_next(style, at, now).unwrap();
                assert!(next > now, "{style:?} {now}: {next} is not later");
                assert_eq!(time_text(style, at, now), time_text(style, at, next - 1));
                assert_ne!(time_text(style, at, next - 1), time_text(style, at, next));
            }
        }
        for style in [TimeStyle::Until, TimeStyle::Left] {
            let at = 1_000_000;
            for now in at - 2 * 86_400..at {
                let next = time_next(style, at, now).unwrap();
                assert!(next > now, "{style:?} {now}: {next} is not later");
                assert_eq!(time_text(style, at, now), time_text(style, at, next - 1));
                assert_ne!(time_text(style, at, next - 1), time_text(style, at, next));
            }
            assert_eq!(time_next(style, at, at), None, "{style:?} ran out");
        }
        assert_eq!(time_next(TimeStyle::At, 0, 0), None);
    }

    #[test]
    fn a_label_counting_hours_wakes_once_an_hour() {
        assert_eq!(time_next(TimeStyle::Ago, 0, 2 * 3600 + 17), Some(3 * 3600));
        assert_eq!(time_next(TimeStyle::Ago, 0, 61), Some(120));
    }

    #[test]
    fn a_given_now_renders_without_the_clock() {
        #[function_component]
        fn Shown() -> Html {
            html! { <Time at={0} now={Some(90)} class={class::SMALL} /> }
        }
        let html = futures::executor::block_on(yew::LocalServerRenderer::<Shown>::new().render());
        assert!(
            html.contains(&format!("class=\"{} {}\"", class::TIME, class::SMALL)),
            "{html}"
        );
        assert!(plain(html).contains(">1m ago<"));
    }
}
