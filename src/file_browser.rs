//! FileBrowser: a dialog choosing a path on a machine the browser cannot
//! see, to open, to save to or a folder (`style/_file-browser.scss`).
//!
//! It knows nothing of how directories are listed: the owner answers each
//! [`ListRequest`] through `list`.

use std::rc::Rc;

use phrasebook02::args;
use phrasebook02::yew023::use_phrases_with;
use yew::prelude::*;

use crate::phrasebookcatalog01::L10N;
use crate::{
    Button, ButtonSize, ButtonVariant, Cluster, Dialog, DialogSize, Field, Gap, Glyph, Icon,
    Notice, PickRow, RowList, Stack, TextInput, Tone, class, field_describedby,
};

/// Whether a [`FileBrowser`] chooses a file to open, a path to save to or a
/// folder.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FileBrowserMode {
    /// Choose an existing file.
    #[default]
    Open,
    /// Choose a path to save to, which may not exist yet.
    Save,
    /// Choose the listed directory. Only directories are listed, and
    /// `extensions` does not apply.
    Folder,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryKind {
    Directory,
    File,
    /// Neither, such as a socket: listed, never chosen.
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListingEntry {
    pub name: String,
    pub kind: EntryKind,
    pub size: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaceKind {
    Home,
    Documents,
    Downloads,
    /// A file system root, labelled with its path.
    Root,
}

/// A place offered above the location.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Place {
    pub kind: PlaceKind,
    pub path: String,
}

/// One listed directory.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Listing {
    /// Its absolute path.
    pub path: String,
    pub parent: Option<String>,
    /// What joins a directory and a name on the listing machine.
    pub separator: String,
    pub entries: Vec<ListingEntry>,
    /// The name in it the requested path named, when that was not a
    /// directory.
    pub file: Option<String>,
    pub places: Vec<Place>,
}

impl Listing {
    fn join(&self, name: &str) -> String {
        if self.path.ends_with(self.separator.as_str()) {
            format!("{}{name}", self.path)
        } else {
            format!("{}{}{name}", self.path, self.separator)
        }
    }

    fn find(&self, name: &str) -> Option<&ListingEntry> {
        self.entries.iter().find(|e| e.name == name)
    }

    /// Its last name, or the whole path at a root.
    fn leaf(&self) -> &str {
        let trimmed = self.path.trim_end_matches(self.separator.as_str());

        match trimmed.rsplit_once(self.separator.as_str()) {
            Some((_, leaf)) if !leaf.is_empty() => leaf,
            _ if trimmed.is_empty() => &self.path,
            _ => trimmed,
        }
    }
}

/// Asks the owner to list `path`: a directory, or a file to list the
/// directory of; empty for the user's home. With `nearest`, a missing path
/// lists its closest existing ancestor instead of failing. A newer request
/// replaces an older one, whose answer is then ignored.
pub struct ListRequest {
    pub path: String,
    pub nearest: bool,
    pub reply: Callback<Result<Listing, String>>,
}

#[derive(Properties, PartialEq)]
pub struct FileBrowserProps {
    pub open: bool,
    #[prop_or_default]
    pub mode: FileBrowserMode,
    pub title: AttrValue,
    /// Where it opens: a file or a directory, empty for the user's home.
    #[prop_or_default]
    pub path: AttrValue,
    /// The name suffixes a chosen file must have (`.gz`); any when empty.
    #[prop_or_default]
    pub extensions: Vec<AttrValue>,
    pub list: Callback<ListRequest>,
    /// The chosen path.
    pub on_choose: Callback<String>,
    pub on_dismiss: Callback<()>,
    /// A directory's mark, else a built-in folder.
    #[prop_or_default]
    pub folder_icon: Option<Html>,
    /// A file's mark, else a built-in page.
    #[prop_or_default]
    pub file_icon: Option<Html>,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

#[derive(Clone, Default, PartialEq)]
struct State {
    listing: Option<Rc<Listing>>,
    error: Option<String>,
    loading: bool,
    /// The editable location, the listed path unless being typed.
    location: String,
    /// The chosen name in the listed directory.
    name: String,
}

enum Action {
    Reset,
    Loading,
    Listed(FileBrowserMode, Listing),
    Failed(String),
    Location(String),
    Name(String),
}

impl Reducible for State {
    type Action = Action;

    fn reduce(self: Rc<Self>, action: Action) -> Rc<Self> {
        let mut state = (*self).clone();

        match action {
            Action::Reset => return Rc::new(State::default()),
            Action::Loading => {
                state.loading = true;
                state.error = None;
            }
            Action::Listed(mode, listing) => {
                // Saving keeps a typed name while browsing to where it goes.
                if mode == FileBrowserMode::Open || listing.file.is_some() {
                    state.name = listing.file.clone().unwrap_or_default();
                }

                state.location = listing.path.clone();
                state.listing = Some(Rc::new(listing));
                state.loading = false;
                state.error = None;
            }
            Action::Failed(error) => {
                state.loading = false;
                state.error = Some(error);
            }
            Action::Location(location) => state.location = location,
            Action::Name(name) => state.name = name,
        }

        Rc::new(state)
    }
}

fn accepts(extensions: &[AttrValue], name: &str) -> bool {
    extensions.is_empty() || extensions.iter().any(|e| name.ends_with(&**e))
}

/// The path `name` in `listing` chooses, if it may be chosen.
fn chosen(
    mode: FileBrowserMode,
    listing: &Listing,
    name: &str,
    extensions: &[AttrValue],
) -> Option<String> {
    if mode == FileBrowserMode::Folder {
        return Some(listing.path.clone());
    }

    let name = name.trim();

    if name.is_empty() || !accepts(extensions, name) {
        return None;
    }

    match (mode, listing.find(name).map(|e| e.kind)) {
        (FileBrowserMode::Open, Some(EntryKind::File))
        | (FileBrowserMode::Save, None | Some(EntryKind::File)) => Some(listing.join(name)),
        _ => None,
    }
}

/// A file browser's loading or empty note's classes.
pub fn file_browser_note() -> Classes {
    classes!(class::FILE_BROWSER_NOTE)
}

/// A dialog browsing directories the owner lists, to choose a file to open,
/// a path to save to or, in [`FileBrowserMode::Folder`], the listed
/// directory: places, an editable location, the directory's
/// entries (hidden ones on request) and, saving, a name that warns when it
/// replaces a file.
#[function_component]
pub fn FileBrowser(props: &FileBrowserProps) -> Html {
    let phrases = use_phrases_with(&L10N);
    let state = use_reducer(State::default);
    let latest = use_mut_ref(|| 0u32);
    let show_hidden = use_state_eq(|| false);
    let mode = props.mode;

    let navigate = {
        let dispatcher = state.dispatcher();
        let latest = latest.clone();
        let list = props.list.clone();

        Callback::from(move |(path, nearest): (String, bool)| {
            let serial = {
                let mut latest = latest.borrow_mut();
                *latest = latest.wrapping_add(1);
                *latest
            };

            dispatcher.dispatch(Action::Loading);

            let dispatcher = dispatcher.clone();
            let latest = latest.clone();

            let reply = Callback::from(move |result: Result<Listing, String>| {
                if *latest.borrow() != serial {
                    return;
                }

                dispatcher.dispatch(match result {
                    Ok(listing) => Action::Listed(mode, listing),
                    Err(error) => Action::Failed(error),
                });
            });

            list.emit(ListRequest {
                path,
                nearest,
                reply,
            });
        })
    };

    {
        let navigate = navigate.clone();
        let dispatcher = state.dispatcher();
        let latest = latest.clone();
        let path = props.path.to_string();

        use_effect_with(props.open, move |&open| {
            if open {
                dispatcher.dispatch(Action::Reset);
                navigate.emit((path, true));
            } else {
                let mut latest = latest.borrow_mut();
                *latest = latest.wrapping_add(1);
            }
        });
    }

    let listing = state.listing.clone();

    let set_name = {
        let dispatcher = state.dispatcher();
        Callback::from(move |name: String| dispatcher.dispatch(Action::Name(name)))
    };

    let chosen = listing
        .as_ref()
        .and_then(|listing| chosen(mode, listing, &state.name, &props.extensions));

    let current = state.name.trim();

    let replaces = mode == FileBrowserMode::Save
        && listing.as_ref().is_some_and(|listing| {
            listing
                .find(current)
                .is_some_and(|e| e.kind == EntryKind::File)
        });

    let visible = |e: &&ListingEntry| {
        (*show_hidden || !e.name.starts_with('.'))
            && (mode != FileBrowserMode::Folder || e.kind == EntryKind::Directory)
    };

    let rows = listing
        .as_ref()
        .map(|listing| {
            listing
                .entries
                .iter()
                .filter(visible)
                .map(|entry| {
                    let path = listing.join(&entry.name);
                    let is_dir = entry.kind == EntryKind::Directory;

                    let onclick = {
                        let navigate = navigate.clone();
                        let set_name = set_name.clone();
                        let name = entry.name.clone();

                        Callback::from(move |_: MouseEvent| {
                            if is_dir {
                                navigate.emit((path.clone(), false));
                            } else {
                                set_name.emit(name.clone());
                            }
                        })
                    };

                    let disabled = match entry.kind {
                        EntryKind::Directory => false,
                        EntryKind::File => !accepts(&props.extensions, &entry.name),
                        EntryKind::Other => true,
                    };

                    let detail = entry.size.map(|size| html!({ format_size(size) }));

                    let mark = if is_dir {
                        props.folder_icon.clone().unwrap_or_else(folder_icon)
                    } else {
                        props.file_icon.clone().unwrap_or_else(file_icon)
                    };

                    html! {
                        <PickRow key={entry.name.clone()} name={html!({ entry.name.clone() })}
                            {detail} {mark} current={!is_dir && entry.name == current}
                            {disabled} {onclick} />
                    }
                })
                .collect::<Html>()
        })
        .unwrap_or_default();

    let note = if listing.is_none() && state.loading {
        Some(phrases.text("sockel-file-browser-loading"))
    } else if listing
        .as_ref()
        .is_some_and(|l| !l.entries.iter().any(|e| visible(&e)))
    {
        Some(if mode == FileBrowserMode::Folder {
            phrases.text("sockel-file-browser-no-folders")
        } else {
            phrases.text("sockel-file-browser-empty")
        })
    } else {
        None
    };

    let places = listing
        .as_ref()
        .map(|listing| {
            listing
                .places
                .iter()
                .map(|place| {
                    let onclick = {
                        let navigate = navigate.clone();
                        let path = place.path.clone();
                        Callback::from(move |_: MouseEvent| navigate.emit((path.clone(), false)))
                    };

                    let label = match place.kind {
                        PlaceKind::Home => phrases.text("sockel-file-browser-home"),
                        PlaceKind::Documents => phrases.text("sockel-file-browser-documents"),
                        PlaceKind::Downloads => phrases.text("sockel-file-browser-downloads"),
                        PlaceKind::Root => place.path.as_str().into(),
                    };

                    html! {
                        <Button size={ButtonSize::Small} variant={ButtonVariant::Quiet}
                            selected={listing.path == place.path} tip={place.path.clone()} {onclick}>
                            { label }
                        </Button>
                    }
                })
                .collect::<Html>()
        })
        .unwrap_or_default();

    let up = {
        let parent = listing.as_ref().and_then(|l| l.parent.clone());
        let disabled = parent.is_none();
        let navigate = navigate.clone();
        let onclick = Callback::from(move |_: MouseEvent| {
            if let Some(parent) = &parent {
                navigate.emit((parent.clone(), false));
            }
        });
        let label = phrases.text("sockel-file-browser-up");

        html! {
            <Button size={ButtonSize::Small} variant={ButtonVariant::Quiet}
                icon={html!(<Icon glyph={Glyph::ChevronUp} />)}
                label={label.clone()} tip={label} {disabled} {onclick} />
        }
    };

    let hidden = {
        let show_hidden = show_hidden.clone();
        let glyph = if *show_hidden {
            Glyph::Eye
        } else {
            Glyph::EyeOff
        };
        let label = phrases.text("sockel-file-browser-hidden");

        html! {
            <Button size={ButtonSize::Small} variant={ButtonVariant::Quiet}
                icon={html!(<Icon {glyph} />)} pressed={Some(*show_hidden)}
                label={label.clone()} tip={label}
                onclick={Callback::from(move |_: MouseEvent| show_hidden.set(!*show_hidden))} />
        }
    };

    let on_location = {
        let dispatcher = state.dispatcher();
        Callback::from(move |location: String| dispatcher.dispatch(Action::Location(location)))
    };

    let on_location_key = {
        let navigate = navigate.clone();
        let location = state.location.clone();

        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                e.prevent_default();
                navigate.emit((location.clone(), false));
            }
        })
    };

    let error = state.error.clone().map(|error| {
        html! {
            <Notice tone={Tone::Danger}>{ error }</Notice>
        }
    });

    let name_field = (mode == FileBrowserMode::Save).then(|| {
        const ID: &str = "sockel-file-browser-name";
        let hint = replaces.then(|| html!({ phrases.text("sockel-file-browser-replaces") }));
        let describedby = field_describedby(ID, hint.is_some(), false);

        html! {
            <Field id={ID} label={html!({ phrases.text("sockel-file-browser-name") })} {hint}>
                <TextInput id={ID} value={state.name.clone()} on_input={set_name.clone()}
                    {describedby} />
            </Field>
        }
    });

    let choose = {
        let on_choose = props.on_choose.clone();
        let chosen = chosen.clone();

        Callback::from(move |_: MouseEvent| {
            if let Some(path) = &chosen {
                on_choose.emit(path.clone());
            }
        })
    };

    let choose_label = match mode {
        FileBrowserMode::Open => phrases.text("sockel-file-browser-open"),
        FileBrowserMode::Save => phrases.text("sockel-file-browser-save"),
        FileBrowserMode::Folder => match &listing {
            Some(listing) => phrases
                .msg(
                    "sockel-file-browser-choose",
                    &args([("name", listing.leaf())]),
                )
                .into(),
            None => phrases.text("sockel-file-browser-choose-none"),
        },
    };

    let actions = html! {
        <>
            <Button onclick={props.on_dismiss.reform(|_| ())}>
                { phrases.text("sockel-file-browser-cancel") }
            </Button>
            <Button variant={ButtonVariant::Primary} disabled={chosen.is_none()}
                title={chosen.clone()} onclick={choose}>{ choose_label }</Button>
        </>
    };

    let head_actions = html! { <>{ up }{ hidden }</> };

    html! {
        <Dialog open={props.open} on_dismiss={props.on_dismiss.reform(|_| ())}
            title={props.title.clone()} size={DialogSize::WideTall} {head_actions} {actions}
            testid={props.testid.clone()}>
            <Stack>
                <Cluster gap={Gap::Tight}>{ places }</Cluster>
                <TextInput value={state.location.clone()} on_input={on_location}
                    on_keydown={on_location_key}
                    label={phrases.text("sockel-file-browser-location")}
                    enterkeyhint="go" spellcheck={false} busy={state.loading} />
                { error }
                if let Some(note) = note {
                    <p class={file_browser_note()}>{ note }</p>
                }
                <RowList tag="div" divided={true} boxed={true}>{ rows }</RowList>
                { name_field }
            </Stack>
        </Dialog>
    }
}

/// `size` bytes, in binary units past 1023.
fn format_size(size: u64) -> String {
    const UNITS: [&str; 4] = ["KB", "MB", "GB", "TB"];

    if size < 1024 {
        return format!("{size} B");
    }

    let mut value = size as f64 / 1024.0;
    let mut unit = "KB";

    for next in UNITS.iter().skip(1) {
        if value < 1024.0 {
            break;
        }

        value /= 1024.0;
        unit = next;
    }

    format!("{value:.1} {unit}")
}

fn folder_icon() -> Html {
    glyph(html! {
        <path d="M1.5 4.5a1 1 0 0 1 1-1h3.5l1.5 1.5h6a1 1 0 0 1 1 1v6.5a1 1 0 0 1-1 1h-11a1 1 0 0 1-1-1z" />
    })
}

fn file_icon() -> Html {
    glyph(html! {
        <><path d="M4 1.5h5l3.5 3.5v9a.5.5 0 0 1-.5.5H4a.5.5 0 0 1-.5-.5V2a.5.5 0 0 1 .5-.5z" /><path d="M9 1.5V5h3.5" /></>
    })
}

fn glyph(paths: Html) -> Html {
    html! {
        <span class={class::GLYPH} aria-hidden="true">
            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"
                stroke-linecap="round" stroke-linejoin="round" focusable="false">
                { paths }
            </svg>
        </span>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn listing() -> Listing {
        let entry = |name: &str, kind| ListingEntry {
            name: name.into(),
            kind,
            size: None,
        };

        Listing {
            path: "/home/me".into(),
            parent: Some("/home".into()),
            separator: "/".into(),
            entries: vec![
                entry("docs", EntryKind::Directory),
                entry("a.gz", EntryKind::File),
                entry("b.txt", EntryKind::File),
                entry("sock", EntryKind::Other),
            ],
            file: None,
            places: Vec::new(),
        }
    }

    fn render(mode: FileBrowserMode) -> String {
        let props = FileBrowserProps {
            open: true,
            mode,
            title: "Choose".into(),
            path: AttrValue::default(),
            extensions: Vec::new(),
            list: Callback::noop(),
            on_choose: Callback::noop(),
            on_dismiss: Callback::noop(),
            folder_icon: None,
            file_icon: None,
            testid: None,
        };

        futures::executor::block_on(
            yew::LocalServerRenderer::<FileBrowser>::with_props(props)
                .hydratable(false)
                .render(),
        )
    }

    #[test]
    fn the_note_is_a_class() {
        let names: Vec<_> = file_browser_note()
            .into_iter()
            .map(|c| c.to_string())
            .collect();
        assert_eq!(names, [class::FILE_BROWSER_NOTE]);
    }

    #[test]
    fn it_is_a_wide_tall_dialog_with_rows() {
        let html = render(FileBrowserMode::Open);
        assert!(html.contains(class::DIALOG), "{html}");
        assert!(html.contains(class::ROW_LIST), "{html}");
        assert!(html.contains("Open"), "{html}");
        assert!(!html.contains("sockel-file-browser-name"), "{html}");
    }

    #[test]
    fn saving_asks_for_a_name() {
        let html = render(FileBrowserMode::Save);
        assert!(html.contains("id=\"sockel-file-browser-name\""), "{html}");
        assert!(html.contains("File name"), "{html}");
    }

    #[test]
    fn opening_chooses_an_accepted_file_only() {
        let listing = listing();
        let gz = [AttrValue::from(".gz")];
        let open = FileBrowserMode::Open;
        assert_eq!(
            chosen(open, &listing, "a.gz", &gz).as_deref(),
            Some("/home/me/a.gz")
        );
        assert_eq!(chosen(open, &listing, "b.txt", &gz), None);
        assert_eq!(chosen(open, &listing, "docs", &[]), None);
        assert_eq!(chosen(open, &listing, "sock", &[]), None);
        assert_eq!(chosen(open, &listing, "new.gz", &gz), None);
        assert_eq!(chosen(open, &listing, " ", &[]), None);
    }

    #[test]
    fn saving_chooses_a_new_or_existing_file() {
        let mut listing = listing();
        listing.path = "/".into();
        let save = FileBrowserMode::Save;
        assert_eq!(
            chosen(save, &listing, "new.gz", &[]).as_deref(),
            Some("/new.gz")
        );
        assert_eq!(
            chosen(save, &listing, "b.txt", &[]).as_deref(),
            Some("/b.txt")
        );
        assert_eq!(chosen(save, &listing, "docs", &[]), None);
    }

    #[test]
    fn a_folder_is_the_listed_directory() {
        let listing = listing();
        let folder = FileBrowserMode::Folder;
        let gz = [AttrValue::from(".gz")];
        assert_eq!(
            chosen(folder, &listing, "", &[]).as_deref(),
            Some("/home/me")
        );
        assert_eq!(
            chosen(folder, &listing, "a.gz", &gz).as_deref(),
            Some("/home/me")
        );
        assert_eq!(
            chosen(folder, &listing, "docs", &[]).as_deref(),
            Some("/home/me")
        );
    }

    #[test]
    fn a_leaf_is_the_last_name_or_the_root() {
        let mut listing = listing();
        assert_eq!(listing.leaf(), "me");
        listing.path = "/home/me/".into();
        assert_eq!(listing.leaf(), "me");
        listing.path = "/".into();
        assert_eq!(listing.leaf(), "/");
        listing.path = "C:\\".into();
        listing.separator = "\\".into();
        assert_eq!(listing.leaf(), "C:");
        listing.path = "C:\\Users".into();
        assert_eq!(listing.leaf(), "Users");
    }

    #[test]
    fn choosing_a_folder_asks_for_no_name() {
        let html = render(FileBrowserMode::Folder);
        assert!(html.contains("Choose"), "{html}");
        assert!(!html.contains("sockel-file-browser-name"), "{html}");
    }

    #[test]
    fn sizes() {
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(3 * 1024 * 1024), "3.0 MB");
    }
}
