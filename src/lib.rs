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
//! - [`Button`] (`.button`): a button or a link styled as one, its
//!   variants and sizes, an icon slot, icon-only and fold, pressed and busy,
//!   all in the same box.
//! - [`Checkbox`] (`.checkbox`) and [`Switch`] (`.switch`): an on or off
//!   toggle in a box of a fixed size.
//! - [`Choice`] (`.choice`): a segmented pick-one group, its options of one
//!   width; [`ChipToggle`] (`.chip`) and [`ChipList`] (`.chip-list`): filter
//!   chips with a count.
//! - [`Field`] (`.field`), [`FormRows`] (`.form-rows`) with [`FormRow`],
//!   [`Fieldset`] and [`FieldCheck`]: a label, control, hint and an error
//!   line that can keep its room.
//! - [`TextInput`] (`.input`), [`Textarea`] (`.textarea`), [`SearchField`]
//!   (`.input-box`) and [`InputGroup`] (`.input-group`): text controls at a
//!   control's height, a unit or a clear button inside the field, controls
//!   joined in a row.
//! - [`IconButton`] (`.iconed`, `.trailing`): a button with an icon, which
//!   the app supplies and styles.
//! - [`Icon`] (`.glyph`), [`IconSet`], [`use_icon`]: plinth's own icons, from
//!   the set an app provides or a built-in one, in a fixed box.
//! - [`Image`] (`.image`): a picture in a box of a fixed ratio, with its
//!   placeholder; [`Avatar`] (`.avatar`): a portrait of a fixed size, its
//!   initials behind it.
//! - [`Notice`] (`.notice`): an inline message in a tone, its action and
//!   close; [`ErrorBox`] (`.error-box`): an error, its cause and a retry;
//!   [`EmptyState`] (`.empty-state`): what stands where there is nothing,
//!   compact for a row.
//! - [`Pill`] (`.pill`), [`Badge`] (`.badge`), [`Dot`] (`.dot`) and [`Count`]
//!   (`.count`): short marks of a fixed box, in a [`Tone`];
//!   [`StatusLabel`] (`.status-label`): a status as wide as its widest state.
//! - [`RowList`] (`.row-list`) with [`RowListItem`]: rows of a name, a detail
//!   and what ends them, divided by lines that add no height; [`PickRow`]
//!   (`.pick-row`): a row that is one press; [`SelectionBar`]
//!   (`.selection-bar`): the picked count and its actions under a list;
//!   [`ActionRow`] (`.action-row`): actions on what stands around them, over
//!   a row's end as an overlay; [`CardGrid`] (`.card-grid`): cards in as many
//!   columns as fit.
//! - [`Toolbar`] (`.toolbar`): one line of controls.
//! - [`Card`] (`.card`): a surface lifted off the page.
//! - [`PageHead`] (`.page-head`) and [`SectionHead`] (`.section-head`): a
//!   page's or a section's title, its help and its actions.
//! - [`Popover`] (`.popover`): a surface beside its anchor in the overlay
//!   layer ([`overlay`]), placed before it shows ([`place`], [`use_popover`]),
//!   dismissed by a press outside or Escape, a sheet on a phone.
//! - [`Dialog`] (`.dialog`): a modal box over a scrim, its head, a body that
//!   scrolls and pinned actions, in three sizes; a sheet on a phone. The page
//!   behind holds still; Escape closes the innermost overlay only.
//! - [`Tooltip`] (`.tooltip`) and [`DataTips`] (`data-tip`): a short tip in
//!   the overlay layer after a pointer's rest or on keyboard focus;
//!   [`HoverCard`] (`.hover-card`): a richer card on hover or focus;
//!   [`ReasonButton`]: a [`Button`] that says why it cannot act, in one node.
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
mod badge;
mod button;
mod card_grid;
mod checkbox;
mod chip;
mod choice;
mod dialog;
mod empty_state;
mod error_box;
mod field;
mod head;
mod icon_button;
mod icons;
mod image;
mod input;
mod layer;
mod layout;
mod library_toolbar;
mod notice;
mod popover;
mod rows;
mod selection;
mod status;
mod styles;
mod tooltip;

pub use action_group::{ActionGroup, ActionGroupProps, action_group};
pub use badge::{
    Badge, BadgeProps, Count, CountProps, Dot, DotProps, Pill, PillProps, Tone, badge, count,
    count_text, dot, pill,
};
pub use button::{Button, ButtonProps, ButtonSize, ButtonVariant, button};
pub use card_grid::{CardGrid, CardGridProps, GridSize, card_grid};
pub use checkbox::{Checkbox, CheckboxProps, Switch, SwitchProps, checkbox, switch};
pub use chip::{ChipList, ChipListProps, ChipToggle, ChipToggleProps, chip_count};
pub use choice::{Choice, ChoiceFit, ChoiceOption, ChoiceProps, choice};
pub use dialog::{Dialog, DialogDismiss, DialogProps, DialogSize, dialog};
pub use empty_state::{EmptyState, EmptyStateProps, empty_state};
pub use error_box::{ErrorBox, ErrorBoxProps};
pub use field::{
    Field, FieldCheck, FieldCheckProps, FieldProps, Fieldset, FieldsetProps, FormRow, FormRows,
    FormRowsProps, field, field_describedby, field_error, form_row, form_rows,
};
pub use head::{Head, HeadLevel, HeadProps, PageHead, SectionHead};
pub use icon_button::{IconButton, IconButtonProps, icon_button};
pub use icons::{Glyph, Icon, IconProps, IconSet, icon, use_icon, use_icons};
pub use image::{
    Avatar, AvatarProps, AvatarSize, Image, ImageProps, Ratio, Status, avatar, image, initials,
};
pub use input::{
    InputGroup, InputGroupProps, InputKind, InputSize, SearchField, SearchFieldProps, TextInput,
    TextInputProps, Textarea, TextareaProps, group_classes, input, input_box, textarea,
};
pub use layout::{
    Align, Card, CardProps, Cluster, ClusterProps, Gap, Inset, Justify, Stack, StackProps, Toolbar,
    ToolbarProps, ToolbarSpacer, ToolbarSpacerProps, card, cluster, stack, toolbar,
};
pub use library_toolbar::{LibraryToolbar, LibraryToolbarProps};
pub use notice::{Notice, NoticeProps, notice};
pub use popover::{
    Dismiss, OVERLAY_ID, Placed, Popover, PopoverAlign, PopoverProps, Rect, Side, overlay, place,
    popover, use_popover,
};
pub use rows::{
    PickRow, PickRowProps, RowList, RowListItem, RowListItemProps, RowListProps, pick_row,
    row_list, row_list_item,
};
pub use selection::{
    ActionRow, ActionRowProps, SelectionBar, SelectionBarProps, action_row, selection_bar,
};
pub use status::{StatusLabel, StatusLabelProps, status_label};
pub use styles::{STYLE_ID, STYLESHEET, Styles, StylesProps};
pub use tooltip::{
    DataTips, HOVER_CARD_DELAY, HoverCard, HoverCardProps, ReasonButton, ReasonButtonProps,
    TOOLTIP_DELAY, Tooltip, TooltipProps, hover_card, reason_text, tooltip,
};
