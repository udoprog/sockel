mod actions;
mod badges;
mod buttons;
mod dialog;
mod fields;
mod foundations;
mod heads;
mod icons;
mod inputs;
mod layout;
mod library;
mod media;
mod notices;
mod popover;
mod rows;
mod styles;
mod toggles;
mod tooltips;

use yew::prelude::*;

use plinth::Head;

use crate::class;
use crate::probe::{Across, Hold, Size};

pub struct Page {
    /// The page's address, after the `#`.
    pub id: &'static str,
    pub title: &'static str,
    /// What the page is about, under its title.
    pub about: &'static str,
    /// The names exported from plinth that the page shows (its component,
    /// props and enums).
    pub covers: &'static [&'static str],
    pub render: fn() -> Html,
    /// The boxes the stability probe checks hold across the page's states.
    pub holds: &'static [Hold],
}

const fn hold(part: &'static str, size: Size, across: Across) -> Hold {
    Hold { part, size, across }
}

/// A page with no states for the probe to cycle.
#[cfg(test)]
const UNPROBED: &[&str] = &["styles"];

/// Every page, in the navigation's order. A name plinth exports must be in
/// some page's `covers`, which the test below checks.
pub const PAGES: &[Page] = &[
    Page {
        id: "stack",
        title: "Stack",
        about: "A column, its blocks a gap apart.",
        covers: &["Stack", "StackProps", "Gap"],
        render: || html! { <layout::StackPage /> },
        holds: &[
            hold(plinth::class::STACK, Size::Width, Across::All),
            hold(plinth::class::STACK, Size::Box, Across::Only(&["tag"])),
            hold(class::SAMPLE, Size::Box, Across::Only(&["tag"])),
        ],
    },
    Page {
        id: "cluster",
        title: "Cluster",
        about: "A row that wraps, its items centred.",
        covers: &["Cluster", "ClusterProps", "Justify", "Align"],
        render: || html! { <layout::ClusterPage /> },
        holds: &[
            hold(
                plinth::class::CLUSTER,
                Size::Width,
                Across::Except(&["inline"]),
            ),
            hold(plinth::class::CLUSTER, Size::Box, Across::Only(&["tag"])),
            hold(class::SAMPLE, Size::Box, Across::Only(&["tag"])),
        ],
    },
    Page {
        id: "action-group",
        title: "ActionGroup",
        about: "The buttons that end a form, a panel or a section, the primary first.",
        covers: &["ActionGroup", "ActionGroupProps", "action_group"],
        render: || html! { <actions::ActionGroupPage /> },
        holds: &[hold(
            plinth::class::ICONED,
            Size::Height,
            Across::Except(&["Buttons"]),
        )],
    },
    Page {
        id: "button",
        title: "Button",
        about: "A button or a link styled as one: variants, sizes, an icon, icon-only, fold, pressed and busy, in the same box. Click it; tab to it; hover it.",
        covers: &[
            "Button",
            "ButtonProps",
            "ButtonVariant",
            "ButtonSize",
            "button",
        ],
        render: || html! { <buttons::ButtonPage /> },
        holds: &[
            hold(
                plinth::class::BUTTON,
                Size::Box,
                Across::Only(&[
                    "pressed",
                    "busy",
                    "disabled",
                    "href (a link)",
                    "focus",
                    "click",
                ]),
            ),
            hold(
                plinth::class::BUTTON,
                Size::Height,
                Across::Except(&["size"]),
            ),
        ],
    },
    Page {
        id: "checkbox",
        title: "Checkbox",
        about: "A native checkbox in a box of a fixed size, with an indeterminate state. Click it; tab to it; press space.",
        covers: &["Checkbox", "CheckboxProps", "checkbox"],
        render: || html! { <toggles::CheckboxPage /> },
        holds: &[hold(plinth::class::CHECKBOX_MARK, Size::Both, Across::All)],
    },
    Page {
        id: "switch",
        title: "Switch",
        about: "An on or off switch (role=switch): its thumb moves by transform, and busy shows a spinner in it. Click it.",
        covers: &["Switch", "SwitchProps", "switch"],
        render: || html! { <toggles::SwitchPage /> },
        holds: &[
            hold(plinth::class::SWITCH_TRACK, Size::Both, Across::All),
            hold(plinth::class::SWITCH_THUMB, Size::Both, Across::All),
        ],
    },
    Page {
        id: "choice",
        title: "Choice",
        about: "A segmented pick-one group: the selected option is marked by its background alone, and the options keep one width. Tab to it, then use the arrow keys.",
        covers: &[
            "Choice",
            "ChoiceProps",
            "ChoiceOption",
            "ChoiceFit",
            "choice",
        ],
        render: || html! { <toggles::ChoicePage /> },
        holds: &[
            hold(
                plinth::class::CHOICE,
                Size::Height,
                Across::Except(&["small"]),
            ),
            hold(
                plinth::class::CHOICE_OPTION,
                Size::Box,
                Across::Only(&["Selected", "invalid", "disabled", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "chip",
        title: "ChipToggle",
        about: "Filter chips with an on state and a count, in a list that keeps one row's height when it is empty. Click a chip.",
        covers: &[
            "ChipToggle",
            "ChipToggleProps",
            "ChipList",
            "ChipListProps",
            "chip_count",
        ],
        render: || html! { <toggles::ChipPage /> },
        holds: &[
            hold(
                plinth::class::CHIP_LIST,
                Size::Height,
                Across::Only(&["Chips=1", "Chips=0 (empty)"]),
            ),
            hold(
                plinth::class::CHIP,
                Size::Both,
                Across::Only(&[
                    "Count=7", "Count=0", "Count=42", "disabled", "focus", "click",
                ]),
            ),
        ],
    },
    Page {
        id: "field",
        title: "Field",
        about: "A label, the control, a hint and an error. With reserve_error the error line keeps its room: set an error and nothing below moves.",
        covers: &[
            "Field",
            "FieldProps",
            "field",
            "field_error",
            "field_describedby",
        ],
        render: || html! { <fields::FieldPage /> },
        holds: &[
            hold(plinth::class::FIELD, Size::Width, Across::All),
            hold(
                plinth::class::FIELD,
                Size::Height,
                Across::Only(&["error", "required", "disabled", "busy", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "form-rows",
        title: "FormRows",
        about: "Rows sharing one label column, the label above its control on a phone; with a Fieldset and a FieldCheck.",
        covers: &[
            "FormRows",
            "FormRowsProps",
            "FormRow",
            "form_row",
            "form_rows",
            "Fieldset",
            "FieldsetProps",
            "FieldCheck",
            "FieldCheckProps",
        ],
        render: || html! { <fields::FormRowsPage /> },
        holds: &[
            hold(plinth::class::FORM_ROWS, Size::Width, Across::All),
            hold(
                plinth::class::FORM_CONTROL,
                Size::Width,
                Across::Except(&["wide_labels", "label"]),
            ),
            hold(
                plinth::class::FORM_ROW,
                Size::Height,
                Across::Only(&["error", "required", "disabled", "busy", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "pill",
        title: "Pill",
        about: "A tinted label of one height, led by a dot in its tone; a button when it has on_click. Change the tone, hover, click.",
        covers: &["Pill", "PillProps", "Tone", "pill"],
        render: || html! { <badges::PillPage /> },
        holds: &[
            hold(plinth::class::PILL, Size::Height, Across::All),
            hold(
                plinth::class::PILL,
                Size::Both,
                Across::Only(&["Tone", "on_click (a button)", "disabled", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "text-input",
        title: "TextInput",
        about: "A text input at a control's height, bare or with a unit inside the field, and a Textarea. Type a long value; tab to it; set it invalid.",
        covers: &[
            "TextInput",
            "TextInputProps",
            "Textarea",
            "TextareaProps",
            "InputKind",
            "InputSize",
            "input",
            "input_box",
            "textarea",
        ],
        render: || html! { <inputs::TextInputPage /> },
        holds: &[
            hold(
                plinth::class::INPUT,
                Size::Both,
                Across::Except(&["Size", "Text"]),
            ),
            hold(
                plinth::class::INPUT_BOX,
                Size::Both,
                Across::Except(&["Size"]),
            ),
            hold(
                plinth::class::INPUT_CONTROL,
                Size::Box,
                Across::Except(&["Size", "Unit"]),
            ),
            hold(
                plinth::class::TEXTAREA,
                Size::Both,
                Across::Except(&["Size"]),
            ),
        ],
    },
    Page {
        id: "badge",
        title: "Badge",
        about: "A short bordered mark, solid or over a corner, and the Dot beside it: filled or a ring, the same box.",
        covers: &["Badge", "BadgeProps", "badge", "Dot", "DotProps", "dot"],
        render: || html! { <badges::BadgePage /> },
        holds: &[
            hold(
                plinth::class::BADGE,
                Size::Height,
                Across::Except(&["corner"]),
            ),
            hold(
                plinth::class::BADGE,
                Size::Both,
                Across::Only(&["Tone", "solid", "focus", "click"]),
            ),
            hold(plinth::class::DOT, Size::Both, Across::All),
        ],
    },
    Page {
        id: "count",
        title: "Count",
        about: "A figure whose width is reserved for its max digits and a +, hidden at none without losing its box.",
        covers: &["Count", "CountProps", "count", "count_text"],
        render: || html! { <badges::CountPage /> },
        holds: &[hold(
            plinth::class::COUNT,
            Size::Box,
            Across::Except(&["Max"]),
        )],
    },
    Page {
        id: "status-label",
        title: "StatusLabel",
        about: "A status as wide as its widest state, so Loading... to Saved does not resize it; its spinner slot is always reserved.",
        covers: &["StatusLabel", "StatusLabelProps", "status_label"],
        render: || html! { <badges::StatusLabelPage /> },
        holds: &[
            hold(
                plinth::class::STATUS_LABEL,
                Size::Box,
                Across::Only(&["State", "focus", "click"]),
            ),
            hold(
                plinth::class::STATUS_SPINNER,
                Size::Box,
                Across::Only(&["State", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "search-field",
        title: "SearchField",
        about: "A search icon, the text and a clear button whose room is always kept: type, then clear it, and the field does not move. Escape clears too.",
        covers: &["SearchField", "SearchFieldProps"],
        render: || html! { <inputs::SearchFieldPage /> },
        holds: &[
            hold(
                plinth::class::INPUT_BOX,
                Size::Box,
                Across::Except(&["Size"]),
            ),
            hold(
                plinth::class::INPUT_CONTROL,
                Size::Box,
                Across::Except(&["Size"]),
            ),
            hold(
                plinth::class::INPUT_CLEAR,
                Size::Box,
                Across::Except(&["Size"]),
            ),
        ],
    },
    Page {
        id: "input-group",
        title: "InputGroup",
        about: "Controls joined in a row, the corners that join squared and one border between them; the members keep one height. A detached member stands apart.",
        covers: &["InputGroup", "InputGroupProps", "group_classes"],
        render: || html! { <inputs::InputGroupPage /> },
        holds: &[
            hold(
                plinth::class::INPUT_GROUP,
                Size::Both,
                Across::Except(&["Size"]),
            ),
            hold(
                plinth::class::INPUT,
                Size::Height,
                Across::Except(&["Size"]),
            ),
        ],
    },
    Page {
        id: "icon-button",
        title: "IconButton",
        about: "A button that leads with (or ends in) an icon the app supplies. Click it; tab to it.",
        covers: &["IconButton", "IconButtonProps", "icon_button"],
        render: || html! { <actions::IconButtonPage /> },
        holds: &[
            hold(
                plinth::class::ICONED,
                Size::Box,
                Across::Only(&["disabled", "dialog", "label (aria-label)", "focus", "click"]),
            ),
            hold(
                plinth::class::ICONED,
                Size::Height,
                Across::Except(&["Small"]),
            ),
        ],
    },
    Page {
        id: "icon",
        title: "Icon",
        about: "plinth's own glyphs, from the app's IconSet or the built-in one, always in the same box. Swap the set; the sizes do not move.",
        covers: &[
            "Icon",
            "IconProps",
            "IconSet",
            "Glyph",
            "icon",
            "use_icon",
            "use_icons",
        ],
        render: || html! { <icons::IconPage /> },
        holds: &[
            hold(
                plinth::class::GLYPH,
                Size::Both,
                Across::Only(&["Icon set", "glyph", "disabled", "focus", "click"]),
            ),
            hold(
                plinth::class::ICONED,
                Size::Height,
                Across::Only(&["Icon set", "glyph", "disabled", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "image",
        title: "Image",
        about: "A picture in a box of a fixed ratio: its placeholder while it loads, when it fails and when there is none is drawn in the same box. Switch the source and the ratio.",
        covers: &["Image", "ImageProps", "Ratio", "Status", "image"],
        render: || html! { <media::ImagePage /> },
        holds: &[
            hold(
                plinth::class::IMAGE,
                Size::Width,
                Across::Except(&["Width"]),
            ),
            hold(
                plinth::class::IMAGE,
                Size::Box,
                Across::Only(&["Source", "alt", "lazy", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "avatar",
        title: "Avatar",
        about: "A round portrait of a fixed size, its initials behind the picture: loading, loaded, failed or missing, it is the same box.",
        covers: &["Avatar", "AvatarProps", "AvatarSize", "avatar", "initials"],
        render: || html! { <media::AvatarPage /> },
        holds: &[
            hold(plinth::class::AVATAR, Size::Both, Across::Except(&["Size"])),
            hold(
                plinth::class::AVATAR,
                Size::Box,
                Across::Only(&["Source", "Name", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "notice",
        title: "Notice",
        about: "An inline message in a tone, with an optional icon, action and close. Closing one collapses its space, by design.",
        covers: &["Notice", "NoticeProps", "notice"],
        render: || html! { <notices::NoticePage /> },
        holds: &[
            hold(
                plinth::class::NOTICE,
                Size::Width,
                Across::Except(&["Shown", "click"]),
            ),
            hold(
                plinth::class::NOTICE,
                Size::Height,
                Across::Only(&["tone", "focus"]),
            ),
        ],
    },
    Page {
        id: "error-box",
        title: "ErrorBox",
        about: "An error with its cause and a retry; a retry in progress keeps the button's box.",
        covers: &["ErrorBox", "ErrorBoxProps"],
        render: || html! { <notices::ErrorBoxPage /> },
        holds: &[
            hold(plinth::class::ERROR_BOX, Size::Width, Across::All),
            hold(
                plinth::class::BUTTON,
                Size::Box,
                Across::Only(&["retrying", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "empty-state",
        title: "EmptyState",
        about: "What stands where there is nothing. Compact, it is one row high, like the row it replaces.",
        covers: &["EmptyState", "EmptyStateProps", "empty_state"],
        render: || html! { <notices::EmptyStatePage /> },
        holds: &[
            hold(plinth::class::EMPTY_STATE, Size::Width, Across::All),
            hold(
                plinth::class::EMPTY_STATE,
                Size::Height,
                Across::Except(&["compact"]),
            ),
        ],
    },
    Page {
        id: "row-list",
        title: "RowList",
        about: "Rows of a name, a detail and what ends them, divided by lines that add no height. Hover or tab to a row.",
        covers: &[
            "RowList",
            "RowListProps",
            "RowListItem",
            "RowListItemProps",
            "row_list",
            "row_list_item",
        ],
        render: || html! { <rows::RowListPage /> },
        holds: &[
            hold(plinth::class::ROW_LIST, Size::Width, Across::All),
            hold(
                plinth::class::ROW_LIST_ITEM,
                Size::Box,
                Across::Only(&["divided", "boxed", "Actions", "overlay", "focus", "click"]),
            ),
            hold(
                plinth::class::ROW_LIST_ITEM,
                Size::Height,
                Across::Except(&["Detail", "Name", "top", "Rows", "State"]),
            ),
        ],
    },
    Page {
        id: "pick-row",
        title: "PickRow",
        about: "A row that is one press, choosing the thing it names: the chosen row is marked by its background alone.",
        covers: &["PickRow", "PickRowProps", "pick_row"],
        render: || html! { <rows::PickRowPage /> },
        holds: &[
            hold(plinth::class::PICK_ROW, Size::Width, Across::All),
            hold(
                plinth::class::PICK_ROW,
                Size::Height,
                Across::Except(&["Detail", "Name"]),
            ),
            hold(
                plinth::class::PICK_ROW,
                Size::Box,
                Across::Only(&["disabled", "mark", "Trailing", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "selection-bar",
        title: "SelectionBar",
        about: "The picked count and its actions under a list. It keeps its room at one height, hidden while nothing is picked, so picking moves nothing.",
        covers: &["SelectionBar", "SelectionBarProps", "selection_bar"],
        render: || html! { <rows::SelectionBarPage /> },
        holds: &[
            hold(plinth::class::SELECTION_BAR, Size::Height, Across::All),
            hold(
                plinth::class::SELECTION_BAR,
                Size::Box,
                Across::Only(&["picked", "Label", "busy", "disabled", "focus", "click"]),
            ),
            hold(
                plinth::class::ROW_LIST,
                Size::Box,
                Across::Only(&["picked", "busy", "sticky", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "action-row",
        title: "ActionRow",
        about: "Actions acting on what stands around them; as an overlay it lies over its row's end, invisible and inert until the row is hovered or focused.",
        covers: &["ActionRow", "ActionRowProps", "action_row"],
        render: || html! { <rows::ActionRowPage /> },
        holds: &[
            hold(
                plinth::class::ROW_LIST_ITEM,
                Size::Box,
                Across::Only(&["Buttons", "Labels", "focus", "click"]),
            ),
            hold(plinth::class::ROW_LIST, Size::Width, Across::All),
        ],
    },
    Page {
        id: "card-grid",
        title: "CardGrid",
        about: "Cards in as many columns as fit, each at least a size's width, sharing the room.",
        covers: &["CardGrid", "CardGridProps", "GridSize", "card_grid"],
        render: || html! { <rows::CardGridPage /> },
        holds: &[
            hold(plinth::class::CARD_GRID, Size::Width, Across::All),
            hold(
                plinth::class::CARD_GRID,
                Size::Box,
                Across::Only(&["focus", "click"]),
            ),
        ],
    },
    Page {
        id: "toolbar",
        title: "Toolbar",
        about: "One line of controls; a ToolbarSpacer pushes the rest to its end. Tab through its buttons.",
        covers: &[
            "Toolbar",
            "ToolbarProps",
            "ToolbarSpacer",
            "ToolbarSpacerProps",
        ],
        render: || html! { <layout::ToolbarPage /> },
        holds: &[
            hold(plinth::class::TOOLBAR, Size::Width, Across::All),
            hold(
                plinth::class::TOOLBAR,
                Size::Height,
                Across::Only(&["Labels", "ToolbarSpacer", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "card",
        title: "Card",
        about: "A surface lifted off the page.",
        covers: &["Card", "CardProps", "Inset"],
        render: || html! { <layout::CardPage /> },
        holds: &[
            hold(plinth::class::CARD, Size::Width, Across::All),
            hold(plinth::class::CARD, Size::Box, Across::Only(&["tag"])),
        ],
    },
    Page {
        id: "page-head",
        title: "PageHead",
        about: "A page's title with its help, and its actions.",
        covers: &["PageHead", "HeadProps", "Head", "HeadLevel"],
        render: || html! { <heads::HeadPage kind={Head::Page} /> },
        holds: &[
            hold(plinth::class::PAGE_HEAD, Size::Width, Across::All),
            hold(
                plinth::class::PAGE_HEAD,
                Size::Height,
                Across::Only(&[
                    "Actions=0",
                    "title_class",
                    "help",
                    "Test ids",
                    "focus",
                    "click",
                ]),
            ),
        ],
    },
    Page {
        id: "section-head",
        title: "SectionHead",
        about: "A section's title with its help, and its actions.",
        covers: &["SectionHead", "HeadProps", "Head", "HeadLevel"],
        render: || html! { <heads::HeadPage kind={Head::Section} /> },
        holds: &[
            hold(plinth::class::SECTION_HEAD, Size::Width, Across::All),
            hold(
                plinth::class::SECTION_HEAD,
                Size::Height,
                Across::Only(&[
                    "Actions=0",
                    "title_class",
                    "help",
                    "Test ids",
                    "focus",
                    "click",
                ]),
            ),
        ],
    },
    Page {
        id: "library-toolbar",
        title: "LibraryToolbar",
        about: "A list's search field and its filters: click a filter to change its text, and the field keeps its width.",
        covers: &["LibraryToolbar", "LibraryToolbarProps"],
        render: || html! { <library::LibraryToolbarPage /> },
        holds: &[
            hold(plinth::class::LIBRARY_TOOLBAR, Size::Width, Across::All),
            hold(
                plinth::class::LIBRARY_SEARCH,
                Size::Box,
                Across::Only(&["icon", "autofocus", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "popover",
        title: "Popover",
        about: "A surface beside its anchor in the overlay layer: placed before it shows, flipped and kept inside the window, following its anchor as the page scrolls; a press outside or Escape closes it, and focus goes back to the anchor. A sheet from the bottom on a phone.",
        covers: &[
            "Popover",
            "PopoverProps",
            "Side",
            "PopoverAlign",
            "Dismiss",
            "Placed",
            "Rect",
            "place",
            "use_popover",
            "overlay",
            "popover",
            "OVERLAY_ID",
        ],
        render: || html! { <popover::PopoverPage /> },
        holds: &[
            hold(plinth::class::BUTTON, Size::Box, Across::All),
            hold(class::SAMPLE, Size::Box, Across::All),
        ],
    },
    Page {
        id: "dialog",
        title: "Dialog",
        about: "A modal box over a scrim: its head, a body that scrolls and actions pinned below it, in three sizes, a sheet from the bottom on a phone. The page behind does not move or lose its scrollbar's room; Tab stays inside; Escape closes the innermost overlay first (open the popover or the second dialog from it), and focus goes back where it was.",
        covers: &[
            "Dialog",
            "DialogProps",
            "DialogSize",
            "DialogDismiss",
            "dialog",
        ],
        render: || html! { <dialog::DialogPage /> },
        holds: &[
            hold(plinth::class::BUTTON, Size::Box, Across::All),
            hold(class::SAMPLE, Size::Box, Across::All),
        ],
    },
    Page {
        id: "tooltip",
        title: "Tooltip",
        about: "A short tip in the overlay layer, shown after the pointer rests on its anchor or at once on keyboard focus, and from a data-tip attribute with DataTips. It never moves the page. Hover it; tab to it; press Escape.",
        covers: &[
            "Tooltip",
            "TooltipProps",
            "DataTips",
            "TOOLTIP_DELAY",
            "tooltip",
        ],
        render: || html! { <tooltips::TooltipPage /> },
        holds: &[
            hold(plinth::class::BUTTON, Size::Box, Across::All),
            hold(class::SAMPLE, Size::Box, Across::All),
        ],
    },
    Page {
        id: "hover-card",
        title: "HoverCard",
        about: "A richer card beside its anchor, opened by hover or keyboard focus and kept open while the pointer is over it, so a link in it can be reached.",
        covers: &[
            "HoverCard",
            "HoverCardProps",
            "HOVER_CARD_DELAY",
            "hover_card",
        ],
        render: || html! { <tooltips::HoverCardPage /> },
        holds: &[
            hold(plinth::class::BUTTON, Size::Box, Across::All),
            hold(class::SAMPLE, Size::Box, Across::All),
        ],
    },
    Page {
        id: "reason-button",
        title: "ReasonButton",
        about: "A Button that says why it cannot act, or what it is doing, in a tooltip. One button across available, unavailable (aria-disabled, so it keeps focus) and working. Tab to it while unavailable.",
        covers: &["ReasonButton", "ReasonButtonProps", "reason_text"],
        render: || html! { <tooltips::ReasonButtonPage /> },
        holds: &[
            hold(
                plinth::class::BUTTON,
                Size::Box,
                Across::Only(&["state", "reason", "focus", "click"]),
            ),
            hold(
                plinth::class::BUTTON,
                Size::Height,
                Across::Except(&["size"]),
            ),
            hold(
                class::SAMPLE,
                Size::Box,
                Across::Only(&["state", "reason", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "classes",
        title: "Class functions",
        about: "The classes of a stack, cluster, toolbar and card, for an element that cannot be the component.",
        covers: &["stack", "cluster", "toolbar", "card"],
        render: || html! { <layout::ClassesPage /> },
        holds: &[hold(plinth::class::STACK, Size::Width, Across::All)],
    },
    Page {
        id: "foundations",
        title: "Foundations",
        about: "The utilities and mixins against jitter: text fitting, tabular figures, a scroll area, the current item, showing by width, the focus ring; and the colours plinth derives.",
        covers: &[],
        render: || html! { <foundations::FoundationsPage /> },
        holds: &[
            hold(
                class::TAB,
                Size::Both,
                Across::Only(&["focus", "click", "current by"]),
            ),
            hold(
                class::READOUT,
                Size::Both,
                Across::Only(&["Figure=8888", "Figure=1010"]),
            ),
            hold(
                class::SCROLL_BOX,
                Size::Width,
                Across::Only(&["Scrolled items", "scroll-area"]),
            ),
        ],
    },
    Page {
        id: "styles",
        title: "Styles",
        about: "plinth's compiled stylesheet, and the component that adds the page's one stylesheet.",
        covers: &["STYLESHEET", "STYLE_ID", "Styles", "StylesProps"],
        render: || html! { <styles::StylesPage /> },
        holds: &[],
    },
];

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{PAGES, UNPROBED};

    /// The names in `src/lib.rs`'s `pub use` items.
    fn exported() -> BTreeSet<&'static str> {
        let lib = include_str!("../../../src/lib.rs");
        let mut names = BTreeSet::new();

        for item in lib.split("pub use ").skip(1) {
            let item = &item[..item.find(';').expect("a `pub use` ends with `;`")];
            let list = match (item.find('{'), item.rfind('}')) {
                (Some(open), Some(close)) => &item[open + 1..close],
                _ => item.rsplit("::").next().unwrap_or(item),
            };

            for name in list.split(',').map(str::trim).filter(|n| !n.is_empty()) {
                // `Name as Alias` exports the alias.
                names.insert(name.rsplit(' ').next().unwrap_or(name));
            }
        }

        names
    }

    #[test]
    fn every_export_has_a_showcase_page() {
        let exported = exported();
        let covered: BTreeSet<&str> = PAGES
            .iter()
            .flat_map(|p| p.covers.iter().copied())
            .collect();

        assert!(exported.contains("Stack"), "the parser found {exported:?}");
        let missing: Vec<_> = exported.difference(&covered).collect();
        assert!(
            missing.is_empty(),
            "exported from plinth without a showcase page: {missing:?}"
        );
        let stale: Vec<_> = covered.difference(&exported).collect();
        assert!(
            stale.is_empty(),
            "showcase pages cover names plinth does not export: {stale:?}"
        );
    }

    #[test]
    fn every_page_declares_the_boxes_it_holds() {
        let bare: Vec<_> = PAGES
            .iter()
            .filter(|p| p.holds.is_empty() != UNPROBED.contains(&p.id))
            .map(|p| p.id)
            .collect();
        assert!(
            bare.is_empty(),
            "pages without holds for the probe (or listed in UNPROBED with some): {bare:?}"
        );
    }

    #[derive(yew::Properties, PartialEq)]
    struct OneProps {
        index: usize,
    }

    #[yew::function_component]
    fn One(props: &OneProps) -> yew::Html {
        (PAGES[props.index].render)()
    }

    #[test]
    fn every_page_shows_its_code() {
        let card = format!(
            "class=\"{} {}\"",
            crate::class::CODE,
            plinth::class::SCROLL_AREA
        );
        let bare: Vec<_> = (0..PAGES.len())
            .filter(|&index| {
                let page = yew::LocalServerRenderer::<One>::with_props(OneProps { index });
                !futures::executor::block_on(page.render()).contains(&card)
            })
            .map(|index| PAGES[index].id)
            .collect();
        assert!(bare.is_empty(), "pages without their code: {bare:?}");
    }

    #[test]
    fn page_ids_are_unique() {
        let ids: BTreeSet<_> = PAGES.iter().map(|p| p.id).collect();
        assert_eq!(ids.len(), PAGES.len());
    }
}
