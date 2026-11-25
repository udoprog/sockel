use yew::prelude::*;

use crate::class;

use sockel::{
    Button, Cluster, EntryKind, FileBrowser, FileBrowserMode, ListRequest, Listing, ListingEntry,
    Place, PlaceKind, Stack,
};

use crate::code::{El, snippet};
use crate::controls::{Demo, Select, Toggle, bind};

const MODES: &[(&str, FileBrowserMode)] = &[
    ("Open", FileBrowserMode::Open),
    ("Save", FileBrowserMode::Save),
    ("Folder", FileBrowserMode::Folder),
];

const HOME: &str = "/home/me";

/// The made-up machine the demo lists: each directory and its entries.
/// A listed name, its kind and its size.
type Entry = (&'static str, EntryKind, Option<u64>);

const TREE: &[(&str, &[Entry])] = &[
    (
        "/",
        &[
            ("etc", EntryKind::Directory, None),
            ("home", EntryKind::Directory, None),
        ],
    ),
    ("/etc", &[("hosts", EntryKind::File, Some(220))]),
    ("/home", &[("me", EntryKind::Directory, None)]),
    (
        HOME,
        &[
            (".config", EntryKind::Directory, None),
            (".profile", EntryKind::File, Some(807)),
            ("Desktop", EntryKind::Directory, None),
            ("Documents", EntryKind::Directory, None),
            ("Downloads", EntryKind::Directory, None),
            ("Music", EntryKind::Directory, None),
            ("Pictures", EntryKind::Directory, None),
            ("Videos", EntryKind::Directory, None),
            ("archive_2026.gz", EntryKind::File, Some(3 * 1024 * 1024)),
            ("notes_typography.txt", EntryKind::File, Some(1536)),
            ("player.sock", EntryKind::Other, None),
            (
                "a file whose name is long enough to be cut with an ellipsis.gz",
                EntryKind::File,
                Some(42),
            ),
        ],
    ),
    (
        "/home/me/.config",
        &[("settings.toml", EntryKind::File, Some(96))],
    ),
    (
        "/home/me/Documents",
        &[
            ("draft_v2.txt", EntryKind::File, Some(12_288)),
            ("report.gz", EntryKind::File, Some(7_340_032)),
        ],
    ),
    ("/home/me/Desktop", &[]),
    ("/home/me/Downloads", &[]),
    (
        "/home/me/Music",
        &[("set_01.flac", EntryKind::File, Some(41_943_040))],
    ),
    (
        "/home/me/Pictures",
        &[("harbour.jpg", EntryKind::File, Some(2_411_724))],
    ),
    ("/home/me/Videos", &[]),
];

/// The places a service would send, in their order.
const PLACES: &[(PlaceKind, &str)] = &[
    (PlaceKind::Home, HOME),
    (PlaceKind::Desktop, "/home/me/Desktop"),
    (PlaceKind::Documents, "/home/me/Documents"),
    (PlaceKind::Downloads, "/home/me/Downloads"),
    (PlaceKind::Music, "/home/me/Music"),
    (PlaceKind::Pictures, "/home/me/Pictures"),
    (PlaceKind::Videos, "/home/me/Videos"),
    (PlaceKind::Root, "/"),
];

/// A place of the app's own, with its label and mark.
const APP_PLACE: &str = "/home/me/.config";

fn app_icon() -> Html {
    html! {
        <svg viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor"
            stroke-width="1.5" stroke-linejoin="round" aria-hidden="true" focusable="false">
            <path d="M8 1.5l1.9 4 4.3.5-3.2 3 .9 4.3L8 11.1l-3.9 2.2.9-4.3-3.2-3 4.3-.5z" />
        </svg>
    }
}

fn parent(path: &str) -> Option<String> {
    let path = path.trim_end_matches('/');
    let (parent, _) = path.rsplit_once('/')?;
    Some(if parent.is_empty() { "/" } else { parent }.to_owned())
}

fn listed(path: &str, file: Option<String>) -> Option<Listing> {
    let (path, entries) = TREE.iter().find(|(p, _)| *p == path)?;

    Some(Listing {
        path: (*path).to_owned(),
        parent: parent(path),
        separator: "/".into(),
        entries: entries
            .iter()
            .map(|(name, kind, size)| ListingEntry {
                name: (*name).to_owned(),
                kind: *kind,
                size: *size,
            })
            .collect(),
        file,
        places: PLACES
            .iter()
            .map(|(kind, path)| Place::new(*kind, *path))
            .chain([Place::new(PlaceKind::Other, APP_PLACE)
                .label("App settings")
                .icon(app_icon())])
            .collect(),
    })
}

/// Answers a request the way a service would: a directory, a file's
/// directory, or with `nearest` the closest one that exists.
fn list(request: ListRequest) {
    let mut path = match request.path.trim() {
        "" => HOME.to_owned(),
        path if path.len() > 1 => path.trim_end_matches('/').to_owned(),
        path => path.to_owned(),
    };
    let mut file = None;

    let result = loop {
        if let Some(listing) = listed(&path, file.clone()) {
            break Ok(listing);
        }

        let name = path.rsplit('/').next().unwrap_or_default().to_owned();

        match parent(&path) {
            // A file in a listed directory, or a missing name to save as.
            Some(up) if file.is_none() && listed(&up, None).is_some() => {
                let named =
                    listed(&up, None).is_some_and(|l| l.entries.iter().any(|e| e.name == name));

                if !named && !request.nearest {
                    break Err(format!("No such file or directory: {}", request.path));
                }

                file = named.then_some(name);
                path = up;
            }
            Some(up) if request.nearest => {
                file = None;
                path = up;
            }
            _ => break Err(format!("No such file or directory: {}", request.path)),
        }
    };

    request.reply.emit(result);
}

fn title(mode: FileBrowserMode) -> &'static str {
    match mode {
        FileBrowserMode::Open => "Open a file",
        FileBrowserMode::Save => "Save as",
        FileBrowserMode::Folder => "Choose a folder",
    }
}

#[derive(Properties, PartialEq, Clone)]
struct Setup {
    mode: FileBrowserMode,
    gz: bool,
    start: bool,
}

#[function_component]
fn FileBrowserDemo(props: &Setup) -> Html {
    let open = use_state(|| false);
    let chosen = use_state(|| None::<String>);

    let on_open = {
        let open = open.clone();
        Callback::from(move |_: MouseEvent| open.set(true))
    };
    let on_dismiss = {
        let open = open.clone();
        Callback::from(move |()| open.set(false))
    };
    let on_choose = {
        let (open, chosen) = (open.clone(), chosen.clone());
        Callback::from(move |path: String| {
            chosen.set(Some(path));
            open.set(false);
        })
    };
    let extensions = if props.gz {
        vec![AttrValue::from(".gz")]
    } else {
        Vec::new()
    };
    let path = if props.start {
        "/home/me/Documents/report.gz"
    } else {
        ""
    };
    let title = title(props.mode);

    html! {
        <Stack>
            <Cluster>
                <Button onclick={on_open} testid="file-browser-open">{ "Browse…" }</Button>
            </Cluster>
            <p class={class::SAMPLE}>
                { chosen.as_deref().map_or_else(|| "Nothing chosen yet".to_owned(), |p| format!("Chosen: {p}")) }
            </p>
            <FileBrowser open={*open} mode={props.mode} {title} {path} {extensions}
                list={Callback::from(list)} {on_choose} {on_dismiss} testid="file-browser" />
        </Stack>
    }
}

#[function_component]
pub fn FileBrowserPage() -> Html {
    let mode = use_state(|| FileBrowserMode::Open);
    let gz = use_state(|| false);
    let start = use_state(|| false);

    let controls = html! {
        <>
            <Select<FileBrowserMode> label="mode" options={MODES} value={*mode} on_change={bind(&mode)} />
            <Toggle label=".gz only" value={*gz} on_change={bind(&gz)} />
            <Toggle label="path (a file)" value={*start} on_change={bind(&start)} />
        </>
    };

    let code = snippet(
        &[
            "let list = Callback::from(|request: ListRequest| {\n    // Ask the service, then `request.reply.emit(result)`.\n});"
                .to_owned(),
        ],
        El::new("FileBrowser")
            .expr("open", "open")
            .variant("mode", *mode)
            .text("title", title(*mode))
            .text_if(*start, "path", "/home/me/Documents/report.gz")
            .expr_if(*gz, "extensions", "vec![\".gz\".into()]")
            .expr("list", "list")
            .expr("on_choose", "choose")
            .expr("on_dismiss", "close"),
    );

    html! {
        <Demo {controls} {code}>
            <FileBrowserDemo mode={*mode} gz={*gz} start={*start} />
        </Demo>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    fn ask(path: &str, nearest: bool) -> Result<Listing, String> {
        let out = Rc::new(RefCell::new(None));
        let reply = {
            let out = out.clone();
            Callback::from(move |result| *out.borrow_mut() = Some(result))
        };
        list(ListRequest {
            path: path.into(),
            nearest,
            reply,
        });
        out.take().unwrap()
    }

    #[test]
    fn the_fake_lists_like_a_service() {
        assert_eq!(ask("", false).unwrap().path, HOME);
        let file = ask("/home/me/Documents/report.gz", false).unwrap();
        assert_eq!(file.path, "/home/me/Documents");
        assert_eq!(file.file.as_deref(), Some("report.gz"));
        assert_eq!(ask("/home/me/x/y", true).unwrap().path, HOME);
        assert!(ask("/home/me/x/y", false).is_err());
        assert_eq!(ask("/", false).unwrap().parent, None);
        assert_eq!(ask("/etc", false).unwrap().parent.as_deref(), Some("/"));
    }
}
