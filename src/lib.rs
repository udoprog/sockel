//! Plinth: the layout primitives and components a Yew app is built from,
//! each a component and its Sass (`style/`), its variants modifier classes
//! set by props, so the markup and the styles stay in step.
//!
//! [`Styles`] adds the compiled stylesheet ([`STYLESHEET`]) to the page; an
//! app with its own Sass build can `@use "<path>/plinth/style"` (and
//! `.../style/tokens` for the tokens and mixins) instead. The colours are the
//! app's: plinth reads `--bg`, `--surface`, `--text`, `--muted`, `--border`,
//! `--accent` and `--shadow-1`.
//!
//! - [`Stack`] (`.stack`): a column, its blocks a gap apart.
//! - [`Cluster`] (`.cluster`): a row that wraps, its items centred.
//! - [`ActionGroup`] (`.action-group`): the buttons ending a form, a panel
//!   or a section.
//! - [`IconButton`] (`.iconed`, `.trailing`): a button with an icon, which
//!   the app supplies.
//! - [`Toolbar`] (`.toolbar`): one line of controls.
//! - [`Card`] (`.card`): a surface lifted off the page.
//! - [`PageHead`] (`.page-head`) and [`SectionHead`] (`.section-head`): a
//!   page's or a section's title, its help and its actions.
//! - [`LibraryToolbar`] (`.library-toolbar`): a list's search field and its
//!   filter buttons, one row.
//!
//! Where a component cannot be used (the element is another component's),
//! the class functions ([`stack`], [`cluster`], [`toolbar`], [`card`]) give
//! the same classes.

/// A constant per class of [`STYLESHEET`], generated from plinth's Sass by
/// plinth-build: upper case, `-` as `_` (`.page-head-title` is
/// [`class::PAGE_HEAD_TITLE`]), so a misspelt class does not compile.
///
/// ```
/// assert_eq!(plinth::class::PAGE_HEAD_TITLE, "page-head-title");
/// ```
///
/// ```compile_fail
/// let _ = plinth::class::STAK;
/// ```
pub mod class {
    include!(concat!(env!("OUT_DIR"), "/classes.rs"));
}

mod action_group;
mod head;
mod icon_button;
mod layout;
mod library_toolbar;
mod styles;

pub use action_group::{ActionGroup, ActionGroupProps, action_group};
pub use head::{Head, HeadLevel, HeadProps, PageHead, SectionHead};
pub use icon_button::{IconButton, IconButtonProps, icon_button};
pub use layout::{
    Align, Card, CardProps, Cluster, ClusterProps, Gap, Inset, Justify, Stack, StackProps, Toolbar,
    ToolbarProps, ToolbarSpacer, ToolbarSpacerProps, card, cluster, stack, toolbar,
};
pub use library_toolbar::{LibraryToolbar, LibraryToolbarProps};
pub use styles::{STYLESHEET, Styles};
