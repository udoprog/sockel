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
//! Each component (or small family) is a Cargo feature that gates its Rust,
//! its class constants and its CSS together; `full`, the default, has all.
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
//!   joined in a row; [`native_select`] (`.native-select`) for a native
//!   `<select>` in a field's look.
//! - [`Stepper`] (`.stepper`): a number between a minus and a plus button,
//!   its room that of the longest bound; [`SecretInput`] (`.input-box`): a
//!   password with show, copy and clear slots that keep their room.
//! - [`IconButton`] (`.iconed`, `.trailing`): a button with an icon, which
//!   the app supplies and styles.
//! - [`Icon`] (`.glyph` or `.name`), [`IconSet`], [`use_icon`]: plinth's and the app's icons, from
//!   the set an app provides or a built-in one, in a fixed box.
//! - [`Image`] (`.image`): a picture in a box of a fixed ratio, with its
//!   placeholder; [`Avatar`] (`.avatar`): a portrait of a fixed size, its
//!   initials behind it.
//! - [`Notice`] (`.notice`): an inline message in a tone, its action and
//!   close; [`ErrorBox`] (`.error-box`): an error, its cause and a retry;
//!   [`EmptyState`] (`.empty-state`): what stands where there is nothing,
//!   compact for a row.
//! - [`DropZone`] (`.drop-zone`): a target for dropped or pasted files, or
//!   picked with its button, with over, refused, busy and error states drawn
//!   by an overlay, so its box never changes; [`accepts`] and [`refusal`]
//!   filter by type and size.
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
//! - [`Reorder`] (`.reorder`) with [`ReorderRow`]s or [`ReorderItem`] children
//!   (their grip a [`ReorderHandle`]): rows moved by a drag on
//!   their [`DragHandle`] (`.drag-handle`) or by keyboard, nothing reflowing
//!   until the drop, each step announced.
//! - [`Spinner`] (`.spinner`): a spinning glyph in the icon box;
//!   [`Skeleton`] (`.skeleton`): a placeholder of the box its content will
//!   have; [`Loadable`] (`.loadable`): loading, empty, error and ready in
//!   one box that keeps its height.
//! - [`Table`] (`.table`) with [`TableRow`] and [`TableCell`]: a table in a
//!   scroll area, its bulk actions' room reserved; [`SortHeader`]
//!   (`.sort-header`): a column heading whose sort mark keeps its box.
//! - [`Pager`] (`.pager`): page buttons in a fixed number of slots, first,
//!   previous, numbers with gaps, next and last, so it keeps its width from
//!   the first page to the last; [`Disclosure`] (`.disclosure`): a summary
//!   row that toggles its body, one height shut or open.
//! - [`ToastHost`] (`.toast-host`) and [`Toast`] (`.toast`): short messages
//!   with an undo, fixed over the window's bottom so they move nothing, their
//!   clock held under the pointer or focus; [`SaveBar`] (`.save-bar`): a
//!   form's unsaved changes, save and discard, in the flow at one height,
//!   with [`UnloadGuard`] for the browser's own prompt.
//! - [`Toolbar`] (`.toolbar`): one line of controls; `band` wraps them
//!   instead (`.band`, with `.chrome` and `.search`), in [`ToolbarPart`]s
//!   (`.toolbar-part`, `.ruled`) that each keep to one line.
//! - [`Panel`] (`.panel-box`): a ringed ground over a list or a view holding
//!   its filters or a short form, as a column or a line, tinted or bare.
//! - [`Seam`] (`.seam`): two panes and the divider dragged between them, each
//!   pane keeping its least size.
//! - [`use_width`]: an element's width, at most once a frame.
//! - [`Tabs`] (`.tabs`) and [`SubTabs`]: a tab list, the selected tab marked by
//!   an underline inside its box; [`NavList`] (`.nav-list`): a side or settings
//!   navigation, the current entry marked by `aria-current`.
//! - [`Card`] (`.card`): a surface lifted off the page.
//! - [`PageHead`] (`.page-head`) and [`SectionHead`] (`.section-head`): a
//!   page's or a section's title, its help and its actions.
//! - [`OverlayHost`] (`.overlay-host`): the optional root of the overlay
//!   layer, which [`use_overlays`] finds ([`overlay`] without one);
//!   [`use_overlay`] puts an app's own overlay on its stack.
//! - [`Popover`] (`.popover`): a surface beside its anchor in the overlay
//!   layer, placed before it shows ([`place`], [`place_fit`], [`use_popover`]),
//!   dismissed by a press outside or Escape, a sheet on a phone.
//! - [`Dialog`] (`.dialog`): a modal box over a scrim, its head, a body that
//!   scrolls and pinned actions, in three sizes; a sheet on a phone. The page
//!   behind holds still; Escape closes the innermost overlay only.
//! - [`Confirm`] (`.confirm`): a dialog asking before an action, with
//!   [`ask`] and [`ConfirmHost`] for promise-style confirms;
//!   [`ConfirmPopover`] and [`ConfirmButton`]: the same beside a button;
//!   [`ConfirmInline`] (`.confirm-inline`): No and Yes replacing a button in
//!   its box.
//! - [`Tooltip`] (`.tooltip`) and [`DataTips`] (`data-tip`): a short tip in
//!   the overlay layer after a pointer's rest or on keyboard focus;
//!   [`HoverCard`] (`.hover-card`): a richer card on hover or focus;
//!   [`ReasonButton`]: a [`Button`] that says why it cannot act, in one node.
//! - [`Menu`] (`.menu`): a button opening a list of actions ([`MenuItem`],
//!   [`MenuCheck`], [`MenuRadio`], separators, headings, notes), with
//!   keyboard navigation and type-ahead; [`RowActions`] (`.row-actions`), a
//!   row's icon buttons and overflow menu; [`SplitButton`]
//!   (`.split-button`); [`ContextMenu`], opened at the pointer.
//! - [`Select`] (`.select`): a field's box showing what is chosen, as wide
//!   as its widest option, opening a [`Listbox`] (`.listbox`) of its options
//!   while focus stays on it; [`Listbox`] is the list's keyboard state.
//! - [`ColorSelect`] (`.color-select`): a swatch face of one width showing the
//!   chosen colour, opening a grid of the app's palette (data) and an optional
//!   custom colour, on the select pattern.
//! - [`SearchSelect`] (`.search-select`): a text field filtering a
//!   [`Listbox`] of options in groups, loaded here or elsewhere, the list
//!   one width and `--picker-rows` high, loading or not.
//! - [`FilterPicker`] (`.filter-picker`): a list's filter button,
//!   `--filter-width` wide, the first choice and a reserved count of the
//!   rest, opening a multi-select listbox.
//! - [`Picker`] (`.picker`): a field of one width showing the current
//!   option, opening a [`ChoiceList`] (`.choice-list`), options with icon,
//!   mark, detail and badge slots under a search field, `--picker-rows`
//!   high; [`PickerMenu`], the same from an action's button;
//!   [`PickerDialog`], the searchable shell for a large
//!   set, single or multi-select, in a [`Dialog`]; [`ChoicePopup`], the
//!   list's popup below any opener the app renders.
//! - [`LibraryToolbar`] (`.library-toolbar`): a list's search field and its
//!   filter buttons, one row.
//!
//! Where a component cannot be used (the element is another component's),
//! the class functions ([`stack`], [`cluster`], [`toolbar`], [`card`], ...) give
//! the same classes.

// A subset of features leaves some of the shared helpers unused.
#![cfg_attr(not(feature = "full"), allow(dead_code, unused_imports))]

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

pub mod phrasebookcatalog01;

#[cfg(feature = "action-group")]
mod action_group;
#[cfg(feature = "badge")]
mod badge;
#[cfg(feature = "button")]
mod button;
#[cfg(feature = "card-grid")]
mod card_grid;
#[cfg(feature = "toggles")]
mod checkbox;
#[cfg(feature = "toggles")]
mod chip;
#[cfg(feature = "toggles")]
mod choice;
#[cfg(feature = "color-select")]
mod color_select;
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
#[cfg(feature = "overlay")]
mod layer;
#[cfg(feature = "layout")]
mod layout;
#[cfg(feature = "library-toolbar")]
mod library_toolbar;
#[cfg(feature = "select")]
mod listbox;
#[cfg(feature = "loadable")]
mod loadable;
#[cfg(feature = "menu")]
mod menu;
#[cfg(feature = "tabs")]
mod nav_list;
#[cfg(feature = "notice")]
mod notice;
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
#[cfg(feature = "reorder")]
mod reorder;
mod roving;
#[cfg(feature = "rows")]
mod rows;
#[cfg(feature = "toast")]
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
mod size;
#[cfg(feature = "skeleton")]
mod skeleton;
#[cfg(feature = "table")]
mod sort_header;
#[cfg(feature = "skeleton")]
mod spinner;
#[cfg(feature = "badge")]
mod status;
#[cfg(feature = "stepper")]
mod stepper;
mod styles;
#[cfg(feature = "table")]
mod table;
#[cfg(feature = "tabs")]
mod tabs;
mod timer;
#[cfg(feature = "toast")]
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
#[cfg(feature = "toggles")]
pub use checkbox::{Checkbox, CheckboxProps, Switch, SwitchProps, checkbox, switch};
#[cfg(feature = "toggles")]
pub use chip::{ChipList, ChipListProps, ChipToggle, ChipToggleProps, chip_count};
#[cfg(feature = "toggles")]
pub use choice::{Choice, ChoiceFit, ChoiceOption, ChoiceProps, choice};
#[cfg(feature = "color-select")]
pub use color_select::{
    ColorOption, ColorSelect, ColorSelectProps, ColorSelectWidth, color_index, color_input_value,
    color_key, color_list, color_name, color_option, color_select, color_swatch,
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
#[cfg(feature = "layout")]
pub use layout::{
    Align, Card, CardProps, Cluster, ClusterProps, Gap, Inset, Justify, Stack, StackProps, Toolbar,
    ToolbarPart, ToolbarPartProps, ToolbarProps, ToolbarSpacer, ToolbarSpacerProps, card, cluster,
    stack, stack_with, tool_band, toolbar, toolbar_part,
};
#[cfg(feature = "library-toolbar")]
pub use library_toolbar::{LibraryToolbar, LibraryToolbarProps};
#[cfg(feature = "select")]
pub use listbox::{LISTBOX_PAGE, ListKey, Listbox, listbox, listbox_option, option_id};
#[cfg(feature = "loadable")]
pub use loadable::{LoadState, Loadable, LoadableProps};
#[cfg(feature = "menu")]
pub use menu::{
    ContextMenu, ContextMenuProps, Menu, MenuCheck, MenuCheckProps, MenuHeading, MenuItem,
    MenuItemProps, MenuNote, MenuPartProps, MenuProps, MenuRadio, MenuRadioProps, MenuSeparator,
    MenuWidth, RowAction, RowActions, RowActionsProps, SplitButton, SplitButtonProps, menu,
    menu_item,
};
#[cfg(feature = "tabs")]
pub use nav_list::{NavItem, NavList, NavListProps, nav_list};
#[cfg(feature = "notice")]
pub use notice::{Notice, NoticeProps, notice};
#[cfg(feature = "overlay")]
pub use overlay::{
    OVERLAY_ID, OverlayHost, OverlayHostProps, Overlays, overlay, overlay_host, use_overlay,
    use_overlays,
};
#[cfg(feature = "pager")]
pub use pager::{
    MIN_SLOTS, Pager, PagerProps, PagerSummary, Slot, page_count, pager, pager_button, pager_slots,
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
#[cfg(feature = "reorder")]
pub use reorder::{
    DragHandle, DragHandleProps, Reorder, ReorderHandle, ReorderHandleProps, ReorderItem,
    ReorderItemProps, ReorderProps, ReorderRow, ReorderText, drag_handle, drag_target, key_target,
    landing, move_item, reorder_row, reorder_text, slot_at,
};
pub use roving::{menu_step, printable, typeahead};
#[cfg(feature = "rows")]
pub use rows::{
    PickRow, PickRowProps, RowList, RowListItem, RowListItemProps, RowListProps, pick_row,
    row_list, row_list_item,
};
#[cfg(feature = "toast")]
pub use save_bar::{SaveBar, SaveBarProps, UnloadGuard, save_bar, unsaved, use_unload_guard};
#[cfg(feature = "seam")]
pub use seam::{Seam, SeamAxis, SeamProps, seam};
#[cfg(feature = "search-select")]
pub use search_select::{
    Listed, SearchOption, SearchSelect, SearchSelectProps, listed, search_matches, search_select,
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
pub use size::use_width;
#[cfg(feature = "skeleton")]
pub use skeleton::{Skeleton, SkeletonProps, SkeletonShape, skeleton};
#[cfg(feature = "table")]
pub use sort_header::{
    SortDirection, SortHeader, SortHeaderProps, next_sort, sort_header, sort_mark,
};
#[cfg(feature = "skeleton")]
pub use spinner::{Spinner, SpinnerProps};
#[cfg(feature = "badge")]
pub use status::{StatusLabel, StatusLabelProps, status_label};
#[cfg(feature = "stepper")]
pub use stepper::{Stepper, StepperProps, key_value, parse_number, step_from, stepper};
pub use styles::{STYLE_ID, STYLESHEET, Styles, StylesProps};
#[cfg(feature = "table")]
pub use table::{
    Table, TableCell, TableCellProps, TableProps, TableRow, TableRowProps, table, table_bulk,
    table_cell, table_wrap,
};
#[cfg(feature = "tabs")]
pub use tabs::{SubTabs, TabItem, Tabs, TabsProps, tabs};
#[cfg(feature = "toast")]
pub use toast::{TOAST_MS, Toast, ToastHost, ToastHostProps, ToastProps, toast, toast_host};
#[cfg(feature = "tooltip")]
pub use tooltip::{
    DataTips, HOVER_CARD_DELAY, HoverCard, HoverCardProps, ReasonButton, ReasonButtonProps,
    TOOLTIP_DELAY, Tooltip, TooltipProps, hover_card, reason_text, tooltip,
};
