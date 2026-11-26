//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/sockel-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/sockel)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/sockel.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/sockel)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-sockel-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/sockel)
//! [Yew] components that hold still.
//!
//! A button turning busy, a count gaining a digit, a field showing its error, a
//! menu opening: in most UIs each of these nudges the layout around it. With
//! sockel none of them does. Every component reserves the room its widest state
//! needs, and overlays float in a layer of their own, so an app's pages never
//! jitter.
//!
//! **[See every component live on the documentation site](https://udoprog.github.io/sockel/)**:
//! each one has controls for its props, the code that renders it, and a probe
//! that checks nothing moves.
//!
//! - **Styled out of the box.** Every component ships its Sass, and its variants
//!   are props, so markup and styles never drift apart.
//! - **Typed classes.** Your Sass and sockel's compile into one stylesheet, with a
//!   Rust constant per class: a misspelt class does not compile.
//! - **Pay for what you render.** Each component is a Cargo feature that gates
//!   its Rust and its CSS together.
//! - **Your colours.** sockel brings no palette. It reads seven colours from the
//!   app and derives the rest, in light or dark.
//! - **Translated.** No English in the code: every word comes from a catalog the
//!   app can extend.
//!
//! <br>
//!
//! ## Getting started
//!
//! An app depends on `sockel`, and builds one stylesheet (sockel's CSS and its
//! own Sass) with `sockel-build` in its `build.rs`. The blocks below are the
//! files of a complete app.
//!
//! In `Cargo.toml`, pick the features the app renders (or leave out
//! `default-features` and `features` to get all of them):
//!
//! <!-- check: Cargo.toml -->
//! ```toml
//! [dependencies]
//! sockel = { version = "0.3", default-features = false, features = ["button", "layout", "overlay"] }
//! yew = { version = "0.23", features = ["csr"] }
//!
//! [build-dependencies]
//! sockel-build = "0.3"
//!
//! [dev-dependencies]
//! sockel-build = "0.3"
//! ```
//!
//! The app's Sass lives in `style/`, with an `_index.scss` that loads sockel's
//! tokens and mixins as `"sockel"`. It also sets the colours sockel reads:
//!
//! <!-- check: style/_index.scss -->
//! ```scss
//! @use "sockel" as p;
//!
//! :root {
//!     --bg: #fafafa;
//!     --surface: #fff;
//!     --text: #1d1d1f;
//!     --muted: #6e6e73;
//!     --border: #d2d2d7;
//!     --accent: #0a66c2;
//!     --shadow-1: 0 1px 2px rgb(0 0 0 / 0.08);
//! }
//!
//! .app-sidebar {
//!     padding: var(--space-4);
//!
//!     @include p.below("phone") {
//!         padding: var(--space-2);
//!     }
//! }
//! ```
//!
//! The build is described once, in `style.rs` at the package root, so that the
//! build script and a test can share it:
//!
//! <!-- check: style.rs -->
//! ```rust,ignore
//! fn style() -> sockel_build::Config {
//!     sockel_build::Config::new()
//!         .local("style/")
//!         .prefix("app-")
//! }
//! ```
//!
//! <!-- check: build.rs -->
//! ```rust,ignore
//! include!("style.rs");
//!
//! fn main() {
//!     style().compile();
//! }
//! ```
//!
//! The build writes `style.css` and `classes.rs` to `OUT_DIR`. The app mounts the
//! stylesheet once at its root with
//! [`Styles`](https://docs.rs/sockel/latest/sockel/struct.Styles.html), and wraps
//! itself in an
//! [`OverlayHost`](https://docs.rs/sockel/latest/sockel/struct.OverlayHost.html)
//! so menus and dialogs have somewhere to float:
//!
//! <!-- check: src/main.rs -->
//! ```rust,ignore
//! use yew::prelude::*;
//!
//! mod class {
//!     include!(concat!(env!("OUT_DIR"), "/classes.rs"));
//! }
//!
//! const STYLESHEET: &str = include_str!(concat!(env!("OUT_DIR"), "/style.css"));
//!
//! #[function_component]
//! fn App() -> Html {
//!     html! {
//!         <>
//!             <sockel::Styles css={STYLESHEET} />
//!             <sockel::OverlayHost>
//!                 <sockel::Stack class={classes!(sockel::class::CARD, class::SIDEBAR)}>
//!                     <sockel::Button>{ "Save" }</sockel::Button>
//!                 </sockel::Stack>
//!             </sockel::OverlayHost>
//!         </>
//!     }
//! }
//!
//! fn main() {
//!     yew::Renderer::<App>::new().render();
//! }
//! ```
//!
//! Finally, a test that fails when the Sass defines a class no Rust code writes:
//!
//! <!-- check: tests/style_classes.rs -->
//! ```rust,ignore
//! use sockel_build::Usage;
//!
//! include!("../style.rs");
//!
//! #[test]
//! fn every_styled_class_is_written() {
//!     Usage::new()
//!         .source(concat!(env!("CARGO_MANIFEST_DIR"), "/src"))
//!         .assert_clean(&style().style(env!("SOCKEL_STYLE")));
//! }
//! ```
//!
//! No other Sass step runs: `index.html` links no stylesheet, and `Trunk.toml`
//! has no `[tools] sass`. An app with no Sass of its own needs none of this; it
//! mounts `<sockel::Styles/>` and uses `sockel::class`.
//!
//! To build against a local checkout of sockel, patch both crates in the
//! workspace's `Cargo.toml`:
//!
//! ```toml
//! [patch.crates-io]
//! sockel = { path = "../sockel" }
//! sockel-build = { path = "../sockel/sockel-build" }
//! ```
//!
//! <br>
//!
//! ## Nothing moves
//!
//! The idea sockel is built around is that a page holds still while its state
//! changes. A component never grows or shrinks because it turned busy, showed an
//! error or got a longer number: it reserves room for its widest state up front.
//! Borders are always there and only change colour, focus is an outline rather
//! than a border, numbers use figures of one width, and anything that floats
//! (menus, popovers, dialogs, toasts) sits in a layer above the page instead of
//! pushing it aside.
//!
//! Every component's page on the [documentation
//! site](https://udoprog.github.io/sockel/) runs a probe that steps through its
//! states and reports anything that moved.
//!
//! The same rules are available for the app's own markup as Sass mixins, and
//! most as utility classes: `numeric`, `reserve-ch`, `truncate`, `clamp`,
//! `scroll-area`, `focus-ring`, `visually-hidden` and more. There are also
//! mixins with the exact rules of sockel's components (`card`, `button`,
//! `input`, `stack`, ...), for markup sockel does not render, such as a
//! dependency's sign-in form (the [mixins
//! reference](https://udoprog.github.io/sockel/#mixins) lists every one, with
//! its arguments):
//!
//! ```scss
//! @use "sockel" as p;
//!
//! .auth-card {
//!     @include p.card;
//! }
//! ```
//!
//! <br>
//!
//! ## Your classes, checked by the compiler
//!
//! `sockel-build` compiles sockel's CSS and the app's Sass into one stylesheet.
//! For every class the app defines it writes a constant, named without the
//! prefix: `.app-sidebar` becomes `class::SIDEBAR`. sockel's own classes are in
//! `sockel::class`. Since markup names classes through constants, a typo or a
//! deleted class is a compile error rather than an unstyled element.
//!
//! Every class the app defines must start with its prefix (`app-`), so it never
//! collides with sockel's `sockel-` classes. sockel's classes are stable, so the
//! app may also refine them (`.sockel-card.app-sidebar`).
//!
//! The test from getting started covers the other direction: a class that is
//! styled but never written is dead weight, and the test fails on it.
//!
//! See the [sockel-build](https://docs.rs/sockel-build) documentation for the
//! details.
//!
//! <br>
//!
//! ## Pay for what you render
//!
//! Every component, or family of components that only work together, is a Cargo
//! feature. `full`, the default, enables them all. A feature gates a component's
//! Rust and its CSS together, so a component whose feature is off does not
//! compile, rather than rendering unstyled. Each feature brings in the features
//! it needs; the [features reference](https://udoprog.github.io/sockel/#features)
//! lists them all.
//!
//! `sockel-build` knows which features sockel was built with, and builds only
//! their CSS. It can also prune rules for components the app never reaches, and
//! the Sass variable `$omit` leaves out smaller parts the app never uses (a
//! button variant, a modifier); the parts it can name are listed on the
//! [configuration reference](https://udoprog.github.io/sockel/#reference/parts-omit-can-name).
//!
//! <br>
//!
//! ## Colours and configuration
//!
//! sockel supplies no colours. The app sets seven custom properties, `--bg`,
//! `--surface`, `--text`, `--muted`, `--border`, `--accent` and `--shadow-1`, and
//! sockel derives the rest from them (danger, success, field borders, the dialog
//! scrim and so on). The status colours are mixed toward `--text`, so they keep
//! their contrast on light and dark backgrounds alike.
//!
//! sockel's scales (type sizes, spacing, radii, control heights) are custom
//! properties too, which the app can override in its own CSS, at the root or for
//! part of the page:
//!
//! ```css
//! .dense {
//!     --control-height: 2rem;
//! }
//! ```
//!
//! Every value is also a `!default` Sass variable, which the build can set. This
//! changes values, never class names:
//!
//! <!-- check-variant: build.rs -->
//! ```rust,no_run
//! fn main() {
//!     sockel_build::Config::new()
//!         .local("style/")
//!         .prefix("app-")
//!         .with("text-md", "1.0625rem")
//!         .with("$radius-lg", "12px")
//!         .compile();
//! }
//! ```
//!
//! Every variable, its default and what it sets is listed on the
//! [configuration reference](https://udoprog.github.io/sockel/#reference),
//! generated from sockel's Sass.
//!
//! <br>
//!
//! ## Translations
//!
//! sockel has no English in its code. Every word a component shows or announces
//! is a message in a [phrasebook-catalog] catalog,
//! `sockel::phrasebookcatalog01::L10N`, in English and Swedish. The app adds it
//! after its own catalog, and can reword any of sockel's messages by defining the
//! same id itself:
//!
//! ```rust
//! # use phrasebook02::{Lang, Phrasebook};
//! # use phrasebookcatalog01::Catalog;
//! # static APP: Catalog = Catalog::new("app", &[]);
//! let book = Phrasebook::builder()
//!     .language(Lang::EN_US, "English")
//!     .language(Lang::new("sv-SE"), "Svenska")
//!     .catalog(&APP)
//!     .catalog(&sockel::phrasebookcatalog01::L10N)
//!     .build();
//! ```
//!
//! Components read the `Phrases` provided with `ContextProvider<Phrases>`. Label
//! props (`close_label` and the like) override a single use. A library built on
//! sockel ships its own catalog the same way; see
//! [`phrasebookcatalog01`](https://docs.rs/sockel/latest/sockel/phrasebookcatalog01/index.html).
//!
//! <br>
//!
//! ## Odds and ends
//!
//! - **Test ids.** Every component takes a `testid` prop, written to
//!   `data-testid`. Set `SOCKEL_TEST_ID_ATTR` in the build environment to use
//!   another attribute name.
//! - **Props.** Nearly every props struct implements `Default`, so a struct
//!   literal ending in `..Default::default()` keeps compiling when a release adds
//!   a prop.
//! - **trug.** sockel's Sass can also be built with [trug], which gives the same
//!   stylesheet: `@use "pkg:sockel";` before the app's own Sass.
//!
//! <br>
//!
//! ## Working on sockel
//!
//! The documentation site is built from `showcase/` and served locally with
//! [trug]:
//!
//! ```sh
//! cd showcase
//! trug serve --port 8095
//! ```
//!
//! A new component lands with its page in `showcase/src/pages/mod.rs`, or
//! `cargo test --workspace` fails. Before sending a change, run:
//!
//! ```sh
//! cargo fmt --check
//! cargo clippy --workspace --all-targets -- -D warnings
//! cargo test --workspace
//! bash scripts/check-features.sh
//! bash scripts/check-packaged.sh
//! ```
//!
//! [Yew]: https://yew.rs
//! [phrasebook-catalog]: https://docs.rs/phrasebook-catalog
//! [trug]: https://crates.io/crates/trug

// The build.rs examples are whole files, `fn main` included.
#![allow(clippy::needless_doctest_main)]
// A subset of features leaves some of the shared helpers unused.
#![cfg_attr(not(feature = "full"), allow(dead_code, unused_imports))]

/// A constant per class of [`STYLESHEET`], generated from sockel's Sass by
/// sockel-build: without the `sockel-` prefix, upper case, `-` as `_`
/// (`.sockel-page-head-title` is [`class::PAGE_HEAD_TITLE`]), so a misspelt class does not compile.
///
/// ```
/// assert_eq!(sockel::class::PAGE_HEAD_TITLE, "sockel-page-head-title");
/// ```
///
/// ```compile_fail
/// let _ = sockel::class::STAK;
/// ```
pub mod class {
    include!(concat!(env!("OUT_DIR"), "/classes.rs"));
}

pub mod phrasebookcatalog01;

mod field_context;
#[allow(dead_code)]
mod testid;

pub use field_context::{FieldContext, use_field};

#[cfg(feature = "action-group")]
mod action_group;
#[cfg(feature = "badge")]
mod badge;
#[cfg(feature = "button")]
mod button;
#[cfg(feature = "card-grid")]
mod card_grid;
#[cfg(feature = "checkbox")]
mod checkbox;
#[cfg(feature = "chip")]
mod chip;
#[cfg(feature = "chip-select")]
mod chip_select;
#[cfg(feature = "choice")]
mod choice;
#[cfg(feature = "code-snippet")]
mod code;
#[cfg(feature = "code-editor")]
mod code_editor;
#[cfg(feature = "code-snippet")]
mod code_snippet;
#[cfg(feature = "color-select")]
mod color_select;
#[cfg(feature = "composer")]
mod composer;
#[cfg(feature = "confirm")]
mod confirm;
#[cfg(feature = "overlay")]
mod dialog;
#[cfg(feature = "disclosure")]
mod disclosure;
#[cfg(feature = "drop-zone")]
mod drop_zone;
#[cfg(feature = "notice")]
mod empty_state;
#[cfg(feature = "notice")]
mod error_box;
#[cfg(feature = "field")]
mod field;
#[cfg(feature = "file-browser")]
mod file_browser;
#[cfg(feature = "picker")]
mod filter_picker;
#[cfg(feature = "head")]
mod head;
#[cfg(feature = "button")]
mod icon_button;
#[cfg(feature = "icons")]
mod icons;
#[cfg(feature = "media")]
mod image;
#[cfg(feature = "input")]
mod input;
#[cfg(feature = "lanes")]
mod lanes;
#[cfg(feature = "overlay")]
mod layer;
#[cfg(feature = "layout")]
mod layout;
#[cfg(feature = "library-toolbar")]
mod library_toolbar;
#[cfg(feature = "lightbox")]
mod lightbox;
#[cfg(feature = "select")]
mod listbox;
#[cfg(feature = "loadable")]
mod loadable;
#[cfg(feature = "log-view")]
mod log_view;
#[cfg(feature = "menu")]
mod menu;
#[cfg(feature = "tabs")]
mod nav_list;
#[cfg(feature = "notice")]
mod notice;
#[cfg(feature = "checkbox")]
mod option_card;
#[cfg(feature = "overlay")]
mod overlay;
#[cfg(feature = "pager")]
mod pager;
#[cfg(feature = "panel")]
mod panel;
#[cfg(feature = "picker")]
mod picker;
#[cfg(feature = "overlay")]
mod popover;
#[cfg(feature = "property-list")]
mod property_list;
#[cfg(feature = "quantity")]
mod quantity;
#[cfg(feature = "reorder")]
mod reorder;
#[cfg(feature = "reorder")]
mod reorder_tree;
mod roving;
#[cfg(feature = "rows")]
mod rows;
#[cfg(feature = "save-bar")]
mod save_bar;
#[cfg(feature = "seam")]
mod seam;
#[cfg(feature = "search-select")]
mod search_select;
#[cfg(feature = "input")]
mod secret_input;
#[cfg(feature = "select")]
mod select;
#[cfg(feature = "rows")]
mod selection;
#[cfg(feature = "shell")]
mod shell;
#[cfg(feature = "shortcuts")]
mod shortcuts;
mod size;
#[cfg(feature = "skeleton")]
mod skeleton;
#[cfg(feature = "table")]
mod sort_header;
#[cfg(feature = "skeleton")]
mod spinner;
#[cfg(feature = "split-view")]
mod split_view;
#[cfg(feature = "badge")]
mod status;
#[cfg(feature = "stepper")]
mod stepper;
mod styles;
#[cfg(feature = "switch")]
mod switch;
#[cfg(feature = "table")]
mod table;
#[cfg(feature = "tabs")]
mod tabs;
#[cfg(feature = "time")]
mod time;
mod timer;
#[cfg(feature = "toasts")]
mod toast;
#[cfg(feature = "tooltip")]
mod tooltip;

#[cfg(feature = "action-group")]
pub use action_group::{ActionGroup, ActionGroupProps, action_group};
#[cfg(feature = "badge")]
pub use badge::{
    Badge, BadgeProps, Count, CountProps, Dot, DotProps, Pill, PillProps, Tone, badge, count,
    count_text, dot, pill,
};
#[cfg(feature = "button")]
pub use button::{Button, ButtonProps, ButtonSize, ButtonVariant, button};
#[cfg(feature = "card-grid")]
pub use card_grid::{CardGrid, CardGridProps, GridSize, card_grid};
#[cfg(feature = "checkbox")]
pub use checkbox::{Checkbox, CheckboxProps, checkbox};
#[cfg(feature = "chip")]
pub use chip::{
    Chip, ChipList, ChipListProps, ChipProps, ChipToggle, ChipToggleProps, chip, chip_count,
};
#[cfg(feature = "chip-select")]
pub use chip_select::{ChipSelect, ChipSelectProps, chip_select};
#[cfg(feature = "choice")]
pub use choice::{Choice, ChoiceFit, ChoiceOption, ChoiceProps, ChoiceStack, choice};
#[cfg(feature = "code-snippet")]
pub use code::{CodeSpan, CodeToken, Diagnostic, Severity, char_to_byte};
#[cfg(feature = "code-editor")]
pub use code_editor::{
    CodeEditor, CodeEditorProps, Completion, CompletionQuery, Completions, code_editor,
    code_editor_insert,
};
#[cfg(feature = "code-snippet")]
pub use code_snippet::{CodeSnippet, CodeSnippetProps, code_snippet};
#[cfg(feature = "color-select")]
pub use color_select::{
    ColorOption, ColorSelect, ColorSelectProps, ColorSelectWidth, color_index, color_input_value,
    color_key, color_list, color_name, color_option, color_select, color_swatch,
};
#[cfg(feature = "composer")]
pub use composer::{
    ComposeKey, Composer, ComposerMessage, ComposerProps, compose_key, composed, composer,
    composer_draft, set_composer_draft,
};
#[cfg(feature = "confirm")]
pub use confirm::{
    Answer, Confirm, ConfirmButton, ConfirmButtonProps, ConfirmHost, ConfirmInline,
    ConfirmInlineProps, ConfirmPopover, ConfirmPopoverProps, ConfirmProps, ConfirmRequest, ask,
    confirm_inline, confirm_popover,
};
#[cfg(feature = "overlay")]
pub use dialog::{Dialog, DialogDismiss, DialogProps, DialogSize, dialog};
#[cfg(feature = "disclosure")]
pub use disclosure::{Disclosure, DisclosureProps, disclosure, disclosure_summary};
#[cfg(feature = "drop-zone")]
pub use drop_zone::{DropState, DropZone, DropZoneProps, Refusal, accepts, drop_zone, refusal};
#[cfg(feature = "notice")]
pub use empty_state::{EmptyState, EmptyStateProps, empty_state};
#[cfg(feature = "notice")]
pub use error_box::{ErrorBox, ErrorBoxProps, error_box};
#[cfg(feature = "field")]
pub use field::{
    Field, FieldCheck, FieldCheckProps, FieldInputs, FieldProps, Fieldset, FieldsetProps, FormRow,
    FormRows, FormRowsProps, field, field_describedby, field_error, field_inputs, form_row,
    form_rows,
};
#[cfg(feature = "file-browser")]
pub use file_browser::{
    EntryKind, FileBrowser, FileBrowserMode, FileBrowserProps, ListRequest, Listing, ListingEntry,
    Place, PlaceKind, file_browser_body, file_browser_note,
};
#[cfg(feature = "picker")]
pub use filter_picker::{
    FilterChoice, FilterPicker, FilterPickerProps, filter_picker, filter_summary, toggled,
};
#[cfg(feature = "head")]
pub use head::{Head, HeadLevel, HeadProps, PageHead, SectionHead};
#[cfg(feature = "button")]
pub use icon_button::{IconButton, IconButtonProps, icon_button};
#[cfg(feature = "icons")]
pub use icons::{Glyph, Icon, IconProps, IconSet, icon, use_icon, use_icon_named, use_icons};
#[cfg(feature = "media")]
pub use image::{
    Avatar, AvatarProps, AvatarSize, Image, ImageProps, Ratio, Status, avatar, image, initials,
};
#[cfg(feature = "input")]
pub use input::{
    InputGroup, InputGroupProps, InputKind, InputSize, SearchField, SearchFieldProps, TextInput,
    TextInputProps, Textarea, TextareaProps, group_classes, input, input_box, native_select,
    textarea,
};
#[cfg(feature = "lanes")]
pub use lanes::{Lane, LaneProps, Lanes, LanesProps, lanes};
#[cfg(feature = "layout")]
pub use layout::{
    Align, Card, CardProps, Cluster, ClusterProps, Gap, Inset, Justify, Stack, StackProps, Toolbar,
    ToolbarPart, ToolbarPartProps, ToolbarProps, ToolbarSpacer, ToolbarSpacerProps, card, cluster,
    cluster_with, stack, stack_with, tool_band, toolbar, toolbar_part,
};
#[cfg(feature = "library-toolbar")]
pub use library_toolbar::{LibraryToolbar, LibraryToolbarProps};
#[cfg(feature = "lightbox")]
pub use lightbox::{
    LIGHTBOX_ZOOMS, Lightbox, LightboxItem, LightboxKey, LightboxProps, lightbox, lightbox_key,
    swipe_step,
};
#[cfg(feature = "select")]
pub use listbox::{LISTBOX_PAGE, ListKey, Listbox, listbox, listbox_option, option_id};
#[cfg(feature = "loadable")]
pub use loadable::{LoadState, Loadable, LoadableProps};
#[cfg(feature = "log-view")]
pub use log_view::{
    LOG_END_SLOP, LogGroup, LogGroupProps, LogView, LogViewProps, at_end, log_group, log_view,
};
#[cfg(feature = "menu")]
pub use menu::{
    ContextMenu, ContextMenuProps, Menu, MenuCheck, MenuCheckProps, MenuHeading, MenuItem,
    MenuItemProps, MenuNote, MenuPartProps, MenuPopup, MenuPopupProps, MenuProps, MenuRadio,
    MenuRadioProps, MenuSeparator, MenuWidth, RowAction, RowActions, RowActionsProps, SplitButton,
    SplitButtonProps, menu, menu_item,
};
#[cfg(feature = "tabs")]
pub use nav_list::{NavItem, NavList, NavListProps, nav_list};
#[cfg(feature = "notice")]
pub use notice::{Notice, NoticeProps, notice};
#[cfg(feature = "checkbox")]
pub use option_card::{OptionCard, OptionCardKind, OptionCardProps, option_card};
#[cfg(feature = "overlay")]
pub use overlay::{
    OVERLAY_ID, OverlayHost, OverlayHostProps, Overlays, overlay, overlay_host, use_overlay,
    use_overlays,
};
#[cfg(feature = "pager")]
pub use pager::{
    MIN_SLOTS, Pager, PagerCounts, PagerProps, PagerSummary, Slot, page_count, pager, pager_button,
    pager_slots,
};
#[cfg(feature = "panel")]
pub use panel::{Panel, PanelGround, PanelProps, panel};
#[cfg(feature = "picker")]
pub use picker::{
    ChoiceList, ChoiceListProps, ChoicePopup, ChoicePopupProps, Picker, PickerDialog,
    PickerDialogProps, PickerMenu, PickerMenuProps, PickerOption, PickerProps, PickerVariant,
    SEARCH_FROM, picker, picker_matches,
};
#[cfg(feature = "overlay")]
pub use popover::{
    Dismiss, Fit, Placed, Popover, PopoverAlign, PopoverProps, Rect, Side, place, place_fit,
    placement_bounds, popover, use_popover, use_popover_within,
};
#[cfg(feature = "property-list")]
pub use property_list::{
    EditKey, InlineEdit, InlineEditProps, PropertyList, PropertyListProps, PropertyRow,
    PropertyRowProps, edit_key, inline_edit, property_list, property_row,
};
#[cfg(feature = "quantity")]
pub use quantity::{
    QuantityField, QuantityFieldProps, Unit, byte_sizes, convert, durations, quantity,
    readable_unit, unit_bounds,
};
#[cfg(feature = "reorder")]
pub use reorder::{
    DragHandle, DragHandleProps, Reorder, ReorderHandle, ReorderHandleProps, ReorderItem,
    ReorderItemProps, ReorderProps, ReorderRow, ReorderText, drag_handle, drag_target, edge_step,
    grid_target, key_target, landing, move_item, move_items, reorder_row, reorder_text, slot_at,
    use_reorder_text,
};
#[cfg(feature = "reorder")]
pub use reorder_tree::{
    ReorderList, ReorderListProps, ReorderSpot, ReorderTree, ReorderTreeHandle,
    ReorderTreeHandleProps, ReorderTreeMove, ReorderTreeProps, TreeList, tree_slots,
};
pub use roving::{menu_step, printable, typeahead};
#[cfg(feature = "rows")]
pub use rows::{
    PickRow, PickRowProps, RowList, RowListItem, RowListItemProps, RowListProps, pick_row,
    row_list, row_list_item,
};
#[cfg(feature = "save-bar")]
pub use save_bar::{SaveBar, SaveBarProps, UnloadGuard, save_bar, unsaved, use_unload_guard};
#[cfg(feature = "seam")]
pub use seam::{Seam, SeamAxis, SeamProps, seam};
#[cfg(feature = "search-select")]
pub use search_select::{
    Listed, SearchOption, SearchSelect, SearchSelectProps, listed, offers_new, search_matches,
    search_select, search_select_row,
};
#[cfg(feature = "input")]
pub use secret_input::{REVEAL_MS, Reveal, SecretInput, SecretInputProps};
#[cfg(feature = "select")]
pub use select::{
    Select, SelectOption, SelectProps, SelectWidth, chosen, filter_matches, picked, select,
};
#[cfg(feature = "rows")]
pub use selection::{
    ActionRow, ActionRowProps, SelectionBar, SelectionBarProps, action_row, selection_bar,
};
#[cfg(feature = "shell")]
pub use shell::{
    AppShell, AppShellProps, BottomNav, BottomNavItem, BottomNavProps, app_shell, bottom_nav,
};
#[cfg(feature = "shortcuts")]
pub use shortcuts::{
    Chord, Kbd, KbdProps, Shortcut, ShortcutHelp, ShortcutHelpProps, is_apple, kbd,
    shortcut_applies, use_shortcut, use_shortcut_layer, use_shortcut_list,
};
pub use size::use_width;
#[cfg(feature = "skeleton")]
pub use skeleton::{Skeleton, SkeletonProps, SkeletonShape, skeleton};
#[cfg(feature = "table")]
pub use sort_header::{
    SortDirection, SortHeader, SortHeaderProps, next_sort, sort_header, sort_mark,
};
#[cfg(feature = "skeleton")]
pub use spinner::{Spinner, SpinnerProps};
#[cfg(feature = "split-view")]
pub use split_view::{SplitView, SplitViewProps, split_view};
#[cfg(feature = "badge")]
pub use status::{StatusLabel, StatusLabelProps, status_label};
#[cfg(feature = "stepper")]
pub use stepper::{
    Stepper, StepperProps, format_number, key_step, key_value, parse_decimal, parse_number,
    step_by, step_from, step_within, stepper,
};
pub use styles::{STYLE_ID, STYLESHEET, Styles, StylesProps};
#[cfg(feature = "switch")]
pub use switch::{Switch, SwitchProps, switch};
#[cfg(feature = "table")]
pub use table::{
    Table, TableCell, TableCellProps, TableProps, TableRow, TableRowProps, table, table_bulk,
    table_cell, table_cell_part, table_frame, table_mode, table_wrap,
};
#[cfg(feature = "tabs")]
pub use tabs::{LinkTabs, LinkTabsProps, SubTabs, TabItem, Tabs, TabsProps, tabs};
#[cfg(feature = "time")]
pub use time::{
    JUST_NOW, Time, TimeProps, TimeStyle, TimeText, now_seconds, time, time_next, time_text,
    use_wake_at,
};
#[cfg(feature = "toasts")]
pub use toast::{TOAST_MS, Toast, ToastHost, ToastHostProps, ToastProps, toast, toast_host};
#[cfg(feature = "tooltip")]
pub use tooltip::{
    DataTips, HOVER_CARD_DELAY, HoverCard, HoverCardProps, ReasonButton, ReasonButtonProps,
    TOOLTIP_DELAY, Tooltip, TooltipProps, hover_card, reason_text, tooltip,
};

#[cfg(test)]
mod props_default {
    use super::*;
    use yew::prelude::*;

    #[test]
    fn default_matches_html_defaults() {
        assert!(yew::props!(CardGridProps {}) == CardGridProps::default());
        assert_eq!(CardGridProps::default().tag, "ul");
    }

    #[test]
    fn struct_update_fills_the_rest() {
        let built = AvatarProps {
            name: AttrValue::from("Ada"),
            ..Default::default()
        };
        assert!(built == yew::props!(AvatarProps { name: "Ada" }));
        assert!(built.lazy);

        let wide = CardGridProps {
            size: GridSize::Wide,
            ..Default::default()
        };
        assert!(
            wide == yew::props!(CardGridProps {
                size: GridSize::Wide
            })
        );
    }
}
