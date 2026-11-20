use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use yew::prelude::*;

use sockel::{
    ChipSelect, Field, FilterChoice, FilterPicker, InputSize, LibraryToolbar, SearchOption,
    SearchSelect, Stack, field_describedby, search_matches,
};

use super::fields::use_control_id;
use crate::class;
use crate::code::{El, snippet};
use crate::controls::{Demo, Select as Control, Text, Toggle, bind};

const SIZES: &[(&str, InputSize)] = &[
    ("Regular", InputSize::Regular),
    ("Small", InputSize::Small),
    ("Touch", InputSize::Touch),
];

const TEXTS: &[(&str, Text)] = &[
    ("Short", Text::Short),
    ("Long", Text::Long),
    ("Long, unbroken", Text::Unbroken),
];

/// What the options are and where they come from.
#[derive(Clone, Copy, PartialEq)]
enum Source {
    Local,
    /// Loaded after a delay per query, the shorter queries slower, so
    /// answers arrive out of order.
    Remote,
    /// Loading, nothing yet.
    Loading,
    /// Results for an older query than the one typed.
    Stale,
    Empty,
}

const SOURCES: &[(&str, Source)] = &[
    ("local", Source::Local),
    ("remote (out of order)", Source::Remote),
    ("loading", Source::Loading),
    ("stale results", Source::Stale),
    ("empty", Source::Empty),
];

/// Calls `f` after `ms` milliseconds.
fn after(ms: i32, f: impl FnOnce() + 'static) {
    if let Some(window) = web_sys::window() {
        let f = Closure::once_into_js(f);
        _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(f.unchecked_ref(), ms);
    }
}

fn fruit(text: Text) -> Vec<SearchOption<usize>> {
    let names = [
        ("Apple", "Pome", None),
        ("Pear", "Pome", Some("Conference")),
        ("Quince", "Pome", None),
        ("Cherry", "Stone", None),
        ("Plum", "Stone", Some("Victoria")),
        ("Apricot", "Stone", None),
        ("Peach", "Stone", None),
        ("Lemon", "Citrus", None),
        ("Lime", "Citrus", None),
        ("Orange", "Citrus", Some("Seville")),
    ];
    names
        .iter()
        .enumerate()
        .map(|(i, &(name, group, detail))| {
            let label = if i == 0 {
                text.of(name)
            } else {
                name.to_owned()
            };
            let option = SearchOption::new(i, label).group(group);
            match detail {
                Some(d) => option.detail(d),
                None => option,
            }
        })
        .collect()
}

#[derive(Clone, Copy, PartialEq)]
struct SearchSettings {
    text: Text,
    source: Source,
    size: InputSize,
    fill: bool,
    invalid: bool,
    disabled: bool,
    paged: bool,
    create: bool,
    free_text: bool,
}

/// How many local matches a page of the paged sample lists.
const SEARCH_PAGE: usize = 4;

#[derive(Properties, PartialEq)]
struct SearchSampleProps {
    settings: SearchSettings,
    value: Option<usize>,
    on_change: Callback<usize>,
}

#[function_component]
fn SearchSample(props: &SearchSampleProps) -> Html {
    let s = props.settings;
    let id = use_control_id();
    // Values created from the field, after the fruit.
    let created = use_state(Vec::<String>::new);
    let mut all = fruit(s.text);
    let first_created = all.len();
    all.extend(
        created
            .iter()
            .enumerate()
            .map(|(i, name)| SearchOption::new(first_created + i, name.clone()).group("Added")),
    );
    // Free text given, standing in for a chosen fruit.
    let free = use_state(|| None::<String>);
    // The query and how many of its matches are listed, when paged.
    let paging = use_state(|| (String::new(), SEARCH_PAGE));
    // The latest query asked for, and the answer last kept.
    let asked = use_mut_ref(String::new);
    let answer = use_state(|| (String::new(), all.clone()));
    let on_search = {
        let (asked, answer, all) = (asked.clone(), answer.clone(), all.clone());
        let paging = paging.clone();
        Callback::from(move |query: String| {
            paging.set((query.clone(), SEARCH_PAGE));
            if s.source != Source::Remote {
                return;
            }
            *asked.borrow_mut() = query.clone();
            let found: Vec<_> = search_matches(&all, &query)
                .into_iter()
                .map(|i| all[i].clone())
                .collect();
            let delay = 900 - 150 * query.len().min(5) as i32;
            let (asked, answer) = (asked.clone(), answer.clone());
            after(delay, move || {
                if *asked.borrow() == query {
                    answer.set((query, found));
                }
            });
        })
    };
    let paged = s.paged && s.source == Source::Local;
    let (options, answers, busy, filter) = match s.source {
        Source::Local if paged => {
            let found: Vec<_> = search_matches(&all, &paging.0)
                .into_iter()
                .map(|i| all[i].clone())
                .collect();
            (found, None, false, false)
        }
        Source::Local => (all, None, false, true),
        Source::Remote => {
            let (q, found) = (*answer).clone();
            let busy = *asked.borrow() != q;
            (found, Some(AttrValue::from(q)), busy, false)
        }
        Source::Loading => (Vec::new(), None, true, true),
        Source::Stale => (all, Some(AttrValue::Static("an older query")), true, false),
        Source::Empty => (Vec::new(), None, false, true),
    };
    let (options, more) = if paged {
        let limit = paging.1;
        let more = options.len() > limit;
        (options.into_iter().take(limit).collect(), more)
    } else {
        (options, false)
    };
    let on_more = {
        let paging = paging.clone();
        Callback::from(move |()| {
            let (query, limit) = (*paging).clone();
            paging.set((query, limit + SEARCH_PAGE));
        })
    };
    let on_change = {
        let (free, on_change) = (free.clone(), props.on_change.clone());
        Callback::from(move |v| {
            free.set(None);
            on_change.emit(v);
        })
    };
    let on_create = s.create.then(|| {
        let (created, on_change) = (created.clone(), on_change.clone());
        Callback::from(move |name: String| {
            let mut next = (*created).clone();
            next.push(name);
            on_change.emit(first_created + next.len() - 1);
            created.set(next);
        })
    });
    let on_text = {
        let free = free.clone();
        Callback::from(move |text: String| free.set(Some(text)))
    };
    let value = if free.is_some() { None } else { props.value };
    let text = (*free).clone().map(AttrValue::from);
    let error = s
        .invalid
        .then(|| html! { "Pick a fruit that is in season." });

    html! {
        <Stack>
            <Field id={id.clone()} label={html! { "Fruit" }} error={error} reserve_error=true
                disabled={s.disabled} busy={busy}>
                <SearchSelect<usize> id={id.clone()} {options} {value}
                    {on_change} {on_search} {filter} {answers} {busy} {more} {on_more}
                    {on_create} free_text={s.free_text} {on_text} {text}
                    label="Search fruit" size={s.size} fill={s.fill} invalid={s.invalid}
                    disabled={s.disabled} describedby={field_describedby(&id, false, s.invalid)}
                    testid="search-select" />
            </Field>
            <p class={class::SAMPLE}>{ "Type to filter; the list keeps its width and height loading, loaded or empty. Arrows, PageUp and PageDown move; Enter picks; Escape closes. Paged lists four matches at a time; new value offers what is typed; free text keeps it." }</p>
        </Stack>
    }
}

#[function_component]
pub fn SearchSelectPage() -> Html {
    let settings = use_state(|| SearchSettings {
        text: Text::Short,
        source: Source::Local,
        size: InputSize::Regular,
        fill: false,
        invalid: false,
        disabled: false,
        paged: false,
        create: false,
        free_text: false,
    });
    let value = use_state(|| Some(3usize));
    let s = *settings;
    let set = |f: fn(&mut SearchSettings, bool)| {
        let settings = settings.clone();
        Callback::from(move |on| {
            let mut next = *settings;
            f(&mut next, on);
            settings.set(next);
        })
    };
    let pick = {
        let value = value.clone();
        Callback::from(move |v: usize| value.set(Some(v)))
    };

    let controls = html! {
        <>
            <Control<Source> label="options" options={SOURCES} value={s.source} on_change={{
                let settings = settings.clone();
                Callback::from(move |source| settings.set(SearchSettings { source, ..*settings }))
            }} />
            <Control<Text> label="Option text" options={TEXTS} value={s.text} on_change={{
                let settings = settings.clone();
                Callback::from(move |text| settings.set(SearchSettings { text, ..*settings }))
            }} />
            <Control<InputSize> label="size" options={SIZES} value={s.size} on_change={{
                let settings = settings.clone();
                Callback::from(move |size| settings.set(SearchSettings { size, ..*settings }))
            }} />
            <Toggle label="fill" value={s.fill} on_change={set(|s, on| s.fill = on)} />
            <Toggle label="invalid" value={s.invalid} on_change={set(|s, on| s.invalid = on)} />
            <Toggle label="disabled" value={s.disabled} on_change={set(|s, on| s.disabled = on)} />
            <Toggle label="paged (local)" value={s.paged} on_change={set(|s, on| s.paged = on)} />
            <Toggle label="new value" value={s.create} on_change={set(|s, on| s.create = on)} />
            <Toggle label="free text" value={s.free_text} on_change={set(|s, on| s.free_text = on)} />
        </>
    };

    let remote = matches!(s.source, Source::Remote | Source::Stale);
    let code = snippet(
        &[
            "let options = vec![".to_owned(),
            "    SearchOption::new(Fruit::Apple, \"Apple\").group(\"Pome\"),".to_owned(),
            "    SearchOption::new(Fruit::Pear, \"Pear\").group(\"Pome\").detail(\"Conference\"),"
                .to_owned(),
            "];".to_owned(),
        ],
        El::new("SearchSelect<Fruit>")
            .expr("id", "id")
            .expr("options", "options")
            .expr("value", "fruit")
            .expr("on_change", "set_fruit")
            .expr_if(remote, "on_search", "load")
            .flag("filter", !remote)
            .expr_if(remote, "answers", "answered_query")
            .flag(
                "busy",
                matches!(s.source, Source::Loading | Source::Stale | Source::Remote),
            )
            .text("label", "Search fruit")
            .variant("size", s.size)
            .flag("fill", s.fill)
            .flag("invalid", s.invalid)
            .flag("disabled", s.disabled)
            .expr_if(s.paged && s.source == Source::Local, "more", "has_more")
            .expr_if(s.paged && s.source == Source::Local, "on_more", "next_page")
            .expr_if(s.create, "on_create", "add_fruit")
            .flag("free_text", s.free_text)
            .expr_if(s.free_text, "on_text", "set_text")
            .expr_if(s.free_text, "text", "text"),
    );

    html! {
        <Demo {controls} {code}>
            <SearchSample settings={s} value={*value} on_change={pick} />
        </Demo>
    }
}

const PICK_MINIMUMS: &[(&str, usize)] = &[("0", 0), ("1", 1), ("2", 2)];

#[function_component]
pub fn ChipSelectPage() -> Html {
    let value = use_state(|| vec![2usize, 0]);
    let ordered = use_state(|| false);
    let min = use_state(|| 1usize);
    let invalid = use_state(|| false);
    let disabled = use_state(|| false);

    let options: Vec<SearchOption<usize>> = ["Apple", "Pear", "Cherry", "Plum", "Fig"]
        .into_iter()
        .enumerate()
        .map(|(i, label)| SearchOption::new(i, label))
        .collect();

    let controls = html! {
        <>
            <Toggle label="ordered" value={*ordered} on_change={bind(&ordered)} />
            <Control<usize> label="min" options={PICK_MINIMUMS} value={*min} on_change={bind(&min)} />
            <Toggle label="invalid" value={*invalid} on_change={bind(&invalid)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
        </>
    };

    let code = snippet(
        &[],
        El::new("ChipSelect<Fruit>")
            .expr("options", "options")
            .expr("value", "picks")
            .expr("on_change", "set_picks")
            .text("label", "Fruit")
            .flag("ordered", *ordered)
            .expr_if(*min > 0, "min", min.to_string())
            .text("empty_label", "No other fruit")
            .flag("invalid", *invalid)
            .flag("disabled", *disabled),
    );

    html! {
        <Demo {controls} {code}>
            <ChipSelect<usize> {options} value={(*value).clone()} on_change={bind(&value)}
                label="Fruit" ordered={*ordered} min={*min} empty_label="No other fruit"
                invalid={*invalid} disabled={*disabled} />
        </Demo>
    }
}

const GENRES: [&str; 6] = ["Rock", "Jazz", "Folk", "Classical", "Electronic", "Hip hop"];

const GENRE_COUNTS: &[(&str, usize)] = &[("6", 6), ("40", 40)];

const CHOSEN: &[(&str, Option<&[usize]>)] = &[
    ("none (all)", Some(&[])),
    ("one", Some(&[1])),
    ("three", Some(&[3, 0, 5])),
    ("every one", Some(&[0, 1, 2, 3, 4, 5])),
    ("picked by hand", None),
];

#[function_component]
pub fn FilterPickerPage() -> Html {
    let chosen = use_state(|| vec![1usize]);
    let text = use_state(|| Text::Short);
    let disabled = use_state(|| false);
    let count = use_state(|| 6usize);
    let query = use_state(String::new);
    let preset = CHOSEN
        .iter()
        .find_map(|(_, c)| c.filter(|c| *c == chosen.as_slice()));

    let choices: Vec<FilterChoice<usize>> = (0..*count)
        .map(|i| {
            let g = GENRES[i % GENRES.len()];
            let label = match i {
                3 => text.of(g),
                _ if i >= GENRES.len() => format!("{g} {}", i / GENRES.len() + 1),
                _ => g.to_owned(),
            };
            FilterChoice::new(i, label)
        })
        .collect();
    let on_change = bind(&chosen);

    let controls = html! {
        <>
            <Control<Option<&'static [usize]>> label="chosen"
                options={CHOSEN}
                value={preset} on_change={{
                    let chosen = chosen.clone();
                    Callback::from(move |c: Option<&'static [usize]>| {
                        if let Some(c) = c {
                            chosen.set(c.to_vec());
                        }
                    })
                }} />
            <Control<usize> label="choices" options={GENRE_COUNTS} value={*count} on_change={bind(&count)} />
            <Control<Text> label="Choice text" options={TEXTS} value={*text} on_change={bind(&text)} />
            <Toggle label="disabled" value={*disabled} on_change={bind(&disabled)} />
        </>
    };

    let code = snippet(
        &["let choices: Vec<FilterChoice<Genre>> = genres.iter().map(..).collect();".to_owned()],
        El::new("FilterPicker<Genre>")
            .expr("choices", "choices")
            .expr("chosen", "genres")
            .expr("on_change", "set_genres")
            .text("label", "Genre")
            .text("all", "All")
            .flag("disabled", *disabled),
    );

    html! {
        <Demo {controls} {code}>
            <Stack>
                <LibraryToolbar value={(*query).clone()} label="Search albums" testid="albums-search"
                    on_input={bind(&query)}>
                    <FilterPicker<usize> choices={choices} chosen={(*chosen).clone()}
                        {on_change} label="Genre" disabled={*disabled} testid="genre-filter" />
                </LibraryToolbar>
                <p class={class::SAMPLE}>{ "Check any number of genres: the button keeps its width, its count's room kept with none. A pick leaves the list open; past eight choices it has a search field and a fixed height." }</p>
            </Stack>
        </Demo>
    }
}
