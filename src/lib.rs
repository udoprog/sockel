//! Plinth: the layout primitives and components a Yew app is built from,
//! each a component and its Sass (`style/`), its variants modifier classes
//! set by props, so the markup and the styles stay in step.
//!
//! [`Styles`] adds the compiled stylesheet ([`STYLESHEET`]) to the page. An
//! app with Sass of its own builds it with plinth-build after plinth's, into
//! one stylesheet, and passes that to [`Styles`] (README: "Adopting plinth").
//! The colours are the app's: plinth reads `--bg`, `--surface`, `--text`,
//! `--muted`, `--border`, `--accent` and `--shadow-1`, and derives
//! overridable defaults for `--danger`, `--success`, `--warning`,
//! `--on-accent`, `--surface-raised` and `--scrim` from them.
//!
//! The utilities against jitter (`.plinth-numeric`, `.plinth-scroll-area`,
//! `.plinth-truncate`, `.plinth-clamp-*`, `.plinth-visually-hidden`,
//! `.plinth-phone-only`, `.plinth-desktop-only`, `.plinth-focus-ring`) are
//! classes only, in [`class`]; their mixins are in README: "Against jitter".
//!
//! - [`Stack`] (`.stack`): a column, its blocks a gap apart.
//! - [`Cluster`] (`.cluster`): a row that wraps, its items centred.
//! - [`ActionGroup`] (`.action-group`): the buttons ending a form, a panel
//!   or a section.
//! - [`IconButton`] (`.iconed`, `.trailing`): a button with an icon, which
//!   the app supplies.
//! - [`Icon`] (`.glyph`), [`IconSet`], [`use_icon`]: plinth's own icons, from
//!   the set an app provides or a built-in one, in a fixed box.
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
/// plinth-build: without the `plinth-` prefix, upper case, `-` as `_`
/// (`.plinth-page-head-title` is [`class::PAGE_HEAD_TITLE`]), so a misspelt class does not compile.
///
/// ```
/// assert_eq!(plinth::class::PAGE_HEAD_TITLE, "plinth-page-head-title");
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
mod icons;
mod layout;
mod library_toolbar;
mod styles;

pub use action_group::{ActionGroup, ActionGroupProps, action_group};
pub use head::{Head, HeadLevel, HeadProps, PageHead, SectionHead};
pub use icon_button::{IconButton, IconButtonProps, icon_button};
pub use icons::{Glyph, Icon, IconProps, IconSet, icon, use_icon, use_icons};
pub use layout::{
    Align, Card, CardProps, Cluster, ClusterProps, Gap, Inset, Justify, Stack, StackProps, Toolbar,
    ToolbarProps, ToolbarSpacer, ToolbarSpacerProps, card, cluster, stack, toolbar,
};
pub use library_toolbar::{LibraryToolbar, LibraryToolbarProps};
pub use styles::{STYLE_ID, STYLESHEET, Styles, StylesProps};
