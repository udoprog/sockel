//! Plinth: the layout primitives and components a Yew app is built from,
//! each a component and its Sass (`style/`), its variants modifier classes
//! set by props, so the markup and the styles stay in step.
//!
//! The Sass is used from the app's stylesheet: `@use "<path>/plinth/style"`
//! for every primitive's rules, and `@use "<path>/plinth/style/tokens"` for
//! the tokens and mixins (breakpoints, page widths, nesting). The colours
//! are the app's: plinth reads `--bg`, `--surface`, `--text`, `--muted`,
//! `--border`, `--accent` and `--shadow-1`.
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

mod action_group;
mod head;
mod icon_button;
mod layout;
mod library_toolbar;

pub use action_group::{ActionGroup, ActionGroupProps, action_group};
pub use head::{Head, HeadProps, PageHead, SectionHead};
pub use icon_button::{IconButton, IconButtonProps, icon_button};
pub use layout::{
    Align, Card, CardProps, Cluster, ClusterProps, Gap, Inset, Justify, Stack, StackProps, Toolbar,
    ToolbarProps, ToolbarSpacer, card, cluster, stack, toolbar,
};
pub use library_toolbar::{LibraryToolbar, LibraryToolbarProps};
