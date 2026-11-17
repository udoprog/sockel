mod actions;
mod badges;
mod buttons;
mod color_select;
mod confirms;
mod dialog;
mod disclosures;
mod drops;
mod fields;
mod foundations;
mod heads;
mod icons;
mod inputs;
mod layout;
mod library;
mod loading;
mod media;
mod menus;
mod notices;
mod pagers;
mod pickers;
mod popover;
mod reorder;
mod rows;
mod select;
mod steppers;
mod styles;
mod tables;
mod tabs;
mod toasts;
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

/// The states of a ConfirmInline that keep its box: not its label, size or icon.
const ASKING: Across = Across::Only(&[
    "danger", "disabled", "busy", "asking", "variant", "focus", "click",
]);

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
        id: "stepper",
        title: "Stepper",
        about: "A number between a minus and a plus button, its room that of the longest bound. Step it, type past the bounds, use the arrow keys.",
        covers: &[
            "Stepper",
            "StepperProps",
            "stepper",
            "step_from",
            "key_value",
            "parse_number",
        ],
        render: || html! { <steppers::StepperPage /> },
        holds: &[
            hold(
                plinth::class::STEPPER,
                Size::Both,
                Across::Except(&["Size", "Max"]),
            ),
            hold(
                plinth::class::STEPPER_CONTROL,
                Size::Box,
                Across::Except(&["Size", "Max"]),
            ),
            hold(
                plinth::class::STEPPER_BUTTON,
                Size::Box,
                Across::Except(&["Size", "Max"]),
            ),
        ],
    },
    Page {
        id: "secret-input",
        title: "SecretInput",
        about: "A password field with show, copy and clear slots that keep their room; the secret hides itself after a while. Show it; clear it.",
        covers: &["SecretInput", "SecretInputProps", "Reveal", "REVEAL_MS"],
        render: || html! { <steppers::SecretInputPage /> },
        holds: &[
            hold(
                plinth::class::INPUT_BOX,
                Size::Both,
                Across::Except(&["Size"]),
            ),
            hold(
                plinth::class::INPUT_CONTROL,
                Size::Box,
                Across::Except(&["Size"]),
            ),
            hold(
                plinth::class::INPUT_ACTION,
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
        id: "color-select",
        title: "ColorSelect",
        about: "A colour picker on the select pattern: a face of one width showing the chosen swatch and name, opening a grid of the app's palette (passed as data) and an optional custom colour. A pick, or a palette still loading, never changes the face's box.",
        covers: &[
            "ColorSelect",
            "ColorSelectProps",
            "ColorSelectWidth",
            "ColorOption",
            "color_select",
            "color_swatch",
            "color_list",
            "color_option",
            "color_index",
            "color_name",
            "color_input_value",
            "color_key",
        ],
        render: || html! { <color_select::ColorSelectPage /> },
        holds: &[
            hold(
                plinth::class::COLOR_SELECT,
                Size::Box,
                Across::Only(&[
                    "value",
                    "Label text",
                    "placeholder",
                    "custom",
                    "palette",
                    "columns",
                    "invalid",
                    "disabled",
                    "busy",
                    "focus",
                    "click",
                ]),
            ),
            hold(
                plinth::class::COLOR_SELECT,
                Size::Height,
                Across::Except(&["size"]),
            ),
            hold(
                plinth::class::COLOR_SWATCH,
                Size::Box,
                Across::Only(&[
                    "value",
                    "Label text",
                    "palette",
                    "invalid",
                    "disabled",
                    "busy",
                ]),
            ),
        ],
    },
    Page {
        id: "select",
        title: "Select",
        about: "A field's box showing what is chosen, opening a listbox in the overlay layer while focus stays on it: a placeholder, disabled options, joined inside an InputGroup. Its face is as wide as its widest option or the placeholder (or its container's, Fill), so a pick never changes its width. Listbox is the list's keyboard state: the active option, PageUp and PageDown, type-ahead.",
        covers: &[
            "Select",
            "SelectProps",
            "SelectOption",
            "SelectWidth",
            "select",
            "chosen",
            "Listbox",
            "ListKey",
            "LISTBOX_PAGE",
            "listbox",
            "listbox_option",
            "option_id",
        ],
        render: || html! { <select::SelectPage /> },
        holds: &[
            hold(
                plinth::class::SELECT,
                Size::Box,
                Across::Only(&[
                    "value",
                    "invalid",
                    "disabled",
                    "busy",
                    "an option disabled",
                    "focus",
                    "click",
                ]),
            ),
            hold(
                plinth::class::SELECT,
                Size::Height,
                Across::Except(&["size"]),
            ),
            hold(
                plinth::class::INPUT_GROUP,
                Size::Height,
                Across::Except(&["size"]),
            ),
        ],
    },
    Page {
        id: "search-select",
        title: "SearchSelect",
        about: "A text field filtering a listbox of options under it, in groups, with details: filtered here, or loaded elsewhere with results for an older query ignored (a loading row stands in). The list is one width and --picker-rows rows high, loading, loaded or empty.",
        covers: &[
            "SearchSelect",
            "SearchSelectProps",
            "SearchOption",
            "Listed",
            "listed",
            "search_matches",
            "search_select",
        ],
        render: || html! { <pickers::SearchSelectPage /> },
        holds: &[
            hold(
                plinth::class::SEARCH_SELECT,
                Size::Box,
                Across::Only(&[
                    "options",
                    "Option text",
                    "invalid",
                    "disabled",
                    "focus",
                    "click",
                ]),
            ),
            hold(
                plinth::class::SEARCH_SELECT,
                Size::Height,
                Across::Except(&["size"]),
            ),
        ],
    },
    Page {
        id: "filter-picker",
        title: "FilterPicker",
        about: "A list's filter button, --filter-width wide: all, or the first choice and a count of the rest in a slot kept with none. It opens a multi-select listbox, an option for all first; a pick leaves it open.",
        covers: &[
            "FilterPicker",
            "FilterPickerProps",
            "FilterChoice",
            "filter_picker",
            "filter_summary",
            "toggled",
        ],
        render: || html! { <pickers::FilterPickerPage /> },
        holds: &[
            hold(plinth::class::FILTER_PICKER, Size::Box, Across::All),
            hold(plinth::class::FILTER_PICKER_COUNT, Size::Box, Across::All),
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
        id: "spinner",
        title: "Spinner",
        about: "A spinning glyph in the icon box, in the colour of the text around it; the busy mark of a Button.",
        covers: &["Spinner", "SpinnerProps"],
        render: || html! { <loading::SpinnerPage /> },
        holds: &[hold(plinth::class::SPINNER, Size::Both, Across::All)],
    },
    Page {
        id: "skeleton",
        title: "Skeleton",
        about: "A placeholder of the box its content will have: a line, a block, a row as high as a RowList row, or a picture of an Image's ratio.",
        covers: &["Skeleton", "SkeletonProps", "SkeletonShape", "skeleton"],
        render: || html! { <loading::SkeletonPage /> },
        holds: &[hold(
            plinth::class::SKELETON,
            Size::Box,
            Across::Only(&["RowList beside", "focus", "click"]),
        )],
    },
    Page {
        id: "loadable",
        title: "Loadable",
        about: "Loading, empty, error and ready content in one box: it keeps the height it last had while it loads again, and the empty state waits a moment so it never flashes before data.",
        covers: &["Loadable", "LoadableProps", "LoadState"],
        render: || html! { <loading::LoadablePage /> },
        holds: &[
            hold(plinth::class::LOADABLE, Size::Width, Across::All),
            hold(
                plinth::class::LOADABLE,
                Size::Height,
                Across::Only(&["state=Loading", "state=Ready", "placeholder", "retrying"]),
            ),
        ],
    },
    Page {
        id: "toast",
        title: "Toast",
        about: "A short message after an action, with an undo, fixed over the window's bottom so it moves nothing; its clock stops under the pointer or focus. Tick Live toast.",
        covers: &[
            "Toast",
            "ToastProps",
            "ToastHost",
            "ToastHostProps",
            "TOAST_MS",
            "toast",
            "toast_host",
        ],
        render: || html! { <toasts::ToastPage /> },
        holds: &[
            hold(
                plinth::class::TOAST,
                Size::Width,
                Across::Only(&["Live toast", "raised", "focus", "click"]),
            ),
            hold(
                plinth::class::TOAST,
                Size::Height,
                Across::Only(&[
                    "tone",
                    "icon",
                    "action",
                    "shortcut",
                    "duration",
                    "raised",
                    "Live toast",
                ]),
            ),
            hold(
                class::SAMPLE,
                Size::Box,
                Across::Only(&[
                    "raised",
                    "duration",
                    "shortcut",
                    "Live toast",
                    "focus",
                    "click",
                ]),
            ),
        ],
    },
    Page {
        id: "save-bar",
        title: "SaveBar",
        about: "A form's unsaved changes: a note, save and discard, always in the flow at one height and only hidden while clean, so the first edit moves nothing.",
        covers: &[
            "SaveBar",
            "SaveBarProps",
            "UnloadGuard",
            "save_bar",
            "unsaved",
            "use_unload_guard",
        ],
        render: || html! { <toasts::SaveBarPage /> },
        holds: &[
            hold(
                plinth::class::SAVE_BAR,
                Size::Box,
                Across::Except(&["status", "label"]),
            ),
            hold(plinth::class::SAVE_BAR, Size::Width, Across::All),
            hold(
                class::SAMPLE,
                Size::Box,
                Across::Except(&["status", "label"]),
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
        id: "reorder",
        title: "Reorder",
        about: "Rows moved by a drag on their handle or by keyboard (Space lifts, the arrows aim, Space drops, Escape cancels; Alt and an arrow moves at once). Nothing reflows until the drop.",
        covers: &[
            "Reorder",
            "ReorderProps",
            "ReorderRow",
            "ReorderText",
            "DragHandle",
            "DragHandleProps",
            "drag_handle",
            "reorder_row",
            "reorder_text",
            "landing",
            "slot_at",
            "move_item",
            "key_target",
        ],
        render: || html! { <reorder::ReorderPage /> },
        holds: &[
            hold(
                plinth::class::REORDER,
                Size::Width,
                Across::Except(&["Rows"]),
            ),
            hold(
                plinth::class::REORDER,
                Size::Box,
                Across::Only(&["Keys", "divided", "end", "disabled", "focus", "click"]),
            ),
            hold(
                plinth::class::REORDER_ROW,
                Size::Box,
                Across::Only(&["Keys", "divided", "end", "disabled", "focus", "click"]),
            ),
            hold(
                plinth::class::DRAG_HANDLE,
                Size::Box,
                Across::Only(&["Keys", "divided", "end", "disabled", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "table",
        title: "Table",
        about: "A table in a scroll area with a sticky head, hover rows and numeric columns; the bulk actions' room is reserved above it. Press a heading to sort.",
        covers: &[
            "Table",
            "TableProps",
            "TableRow",
            "TableRowProps",
            "TableCell",
            "TableCellProps",
            "table",
            "table_wrap",
            "table_bulk",
            "table_cell",
        ],
        render: || html! { <tables::TablePage /> },
        holds: &[
            hold(plinth::class::TABLE_FRAME, Size::Width, Across::All),
            hold(
                plinth::class::SORT_HEADER,
                Size::Box,
                Across::Only(&["Sort", "Picked", "disabled", "focus", "click"]),
            ),
            hold(
                plinth::class::TABLE_BULK,
                Size::Box,
                Across::Only(&["Picked", "Sort", "focus", "click"]),
            ),
            hold(
                plinth::class::TABLE_ROW,
                Size::Height,
                Across::Only(&["Sort", "Picked", "fixed", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "sort-header",
        title: "SortHeader",
        about: "A column heading that sorts: unsorted, first way, the other way, unsorted. The mark has its box in every state, so sorting never changes the heading's width.",
        covers: &[
            "SortHeader",
            "SortHeaderProps",
            "SortDirection",
            "next_sort",
            "sort_header",
            "sort_mark",
        ],
        render: || html! { <tables::SortHeaderPage /> },
        holds: &[
            hold(
                plinth::class::SORT_HEADER,
                Size::Box,
                Across::Only(&["Sort", "disabled", "Descending first", "focus", "click"]),
            ),
            hold(plinth::class::TABLE_FRAME, Size::Width, Across::All),
        ],
    },
    Page {
        id: "pager",
        title: "Pager",
        about: "Page buttons in a fixed number of slots: first, previous, numbers with gaps, next and last. The slot count and each button's width never change with the page, and the current page is marked without bold.",
        covers: &[
            "Pager",
            "PagerProps",
            "Slot",
            "MIN_SLOTS",
            "page_count",
            "pager",
            "pager_button",
            "pager_slots",
        ],
        render: || html! { <pagers::PagerPage /> },
        holds: &[
            hold(
                plinth::class::PAGER,
                Size::Width,
                Across::Only(&["disabled", "busy", "focus", "click"]),
            ),
            hold(plinth::class::PAGER, Size::Height, Across::All),
        ],
    },
    Page {
        id: "disclosure",
        title: "Disclosure",
        about: "A summary row that toggles its body (aria-expanded). The row is one height shut or open: the mark turns inside its box and the summary is cut to one line.",
        covers: &["Disclosure", "DisclosureProps", "disclosure"],
        render: || html! { <disclosures::DisclosurePage /> },
        holds: &[
            hold(plinth::class::DISCLOSURE, Size::Width, Across::All),
            hold(
                plinth::class::DISCLOSURE_HEAD,
                Size::Box,
                Across::Only(&[
                    "Title", "Summary", "open", "actions", "disabled", "focus", "click",
                ]),
            ),
            hold(
                plinth::class::DISCLOSURE_MARK,
                Size::Box,
                Across::Only(&[
                    "Title", "Summary", "open", "actions", "disabled", "focus", "click",
                ]),
            ),
        ],
    },
    Page {
        id: "drop-zone",
        title: "DropZone",
        about: "A target for dropped or pasted files, or picked with its button. Over, refused, busy and error are drawn by an overlay inside a border the zone always has, so no state or message changes its size. Drop a file on it to see its states.",
        covers: &[
            "DropState",
            "DropZone",
            "DropZoneProps",
            "Refusal",
            "accepts",
            "drop_zone",
            "refusal",
        ],
        render: || html! { <drops::DropZonePage /> },
        holds: &[
            hold(
                plinth::class::DROP_ZONE,
                Size::Box,
                Across::Except(&["Text", "pick"]),
            ),
            hold(plinth::class::DROP_ZONE, Size::Width, Across::All),
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
        id: "confirm",
        title: "Confirm",
        about: "A dialog asking before an action: the explanation, then Cancel and the accepting button pinned below, the focus on Cancel when accepting destroys. Escape and the scrim cancel. ConfirmHost and ask put the same question from any code and resolve with the answer: press ask(..).await.",
        covers: &[
            "Confirm",
            "ConfirmProps",
            "ConfirmHost",
            "ConfirmRequest",
            "Answer",
            "ask",
        ],
        render: || html! { <confirms::ConfirmPage /> },
        holds: &[
            hold(plinth::class::BUTTON, Size::Box, Across::All),
            hold(class::SAMPLE, Size::Box, Across::All),
        ],
    },
    Page {
        id: "confirm-popover",
        title: "ConfirmPopover",
        about: "A confirm beside the button that asked, in a popover of a fixed width: the focus on Cancel when accepting destroys, Escape and a press outside cancel and focus returns to the button. ConfirmButton is a Button that opens it. A sheet on a phone.",
        covers: &[
            "ConfirmPopover",
            "ConfirmPopoverProps",
            "ConfirmButton",
            "ConfirmButtonProps",
        ],
        render: || html! { <confirms::ConfirmPopoverPage /> },
        holds: &[
            hold(plinth::class::BUTTON, Size::Box, Across::All),
            hold(class::SAMPLE, Size::Box, Across::All),
        ],
    },
    Page {
        id: "confirm-inline",
        title: "ConfirmInline",
        about: "No and Yes replacing a button in its own box: both are always laid out in one cell, so the row is as wide as the wider and as high as the button whether it asks or not. No takes the focus; Escape goes back.",
        covers: &["ConfirmInline", "ConfirmInlineProps", "confirm_inline"],
        render: || html! { <confirms::ConfirmInlinePage /> },
        holds: &[
            hold(plinth::class::CONFIRM_INLINE, Size::Box, ASKING),
            hold(plinth::class::CONFIRM_IDLE, Size::Box, ASKING),
            hold(class::SAMPLE, Size::Box, ASKING),
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
        id: "menu",
        title: "Menu",
        about: "A button opening a list of actions in the overlay layer: items with an icon, a detail and a hint, checks and radios, headings, separators and notes. Arrows, Home, End and type-ahead move between items; a pick, Tab, Escape or a press outside closes it, and Escape returns focus to the opener. Its width is set once on open.",
        covers: &[
            "Menu",
            "MenuProps",
            "MenuWidth",
            "MenuItem",
            "MenuItemProps",
            "MenuCheck",
            "MenuCheckProps",
            "MenuRadio",
            "MenuRadioProps",
            "MenuSeparator",
            "MenuHeading",
            "MenuNote",
            "MenuPartProps",
            "menu",
            "menu_item",
            "menu_step",
            "typeahead",
        ],
        render: || html! { <menus::MenuPage /> },
        holds: &[
            hold(
                plinth::class::BUTTON,
                Size::Box,
                Across::Except(&["variant", "size", "icon only"]),
            ),
            hold(
                plinth::class::BUTTON,
                Size::Height,
                Across::Except(&["size"]),
            ),
            hold(class::SAMPLE, Size::Box, Across::Except(&["size"])),
        ],
    },
    Page {
        id: "row-actions",
        title: "RowActions",
        about: "A row's actions: small icon buttons of one size, always shown, the danger ones last and muted until pointed at, then the overflow menu behind its ellipsis.",
        covers: &["RowActions", "RowActionsProps", "RowAction"],
        render: || html! { <menus::RowActionsPage /> },
        holds: &[
            hold(plinth::class::ROW_ACTIONS, Size::Height, Across::All),
            hold(
                plinth::class::ROW_ACTIONS,
                Size::Width,
                Across::Except(&["more (a menu)"]),
            ),
        ],
    },
    Page {
        id: "split-button",
        title: "SplitButton",
        about: "A main action joined to a menu of its variants: one control of one height, the two parts sharing a border.",
        covers: &["SplitButton", "SplitButtonProps"],
        render: || html! { <menus::SplitButtonPage /> },
        holds: &[
            hold(
                plinth::class::SPLIT_BUTTON,
                Size::Height,
                Across::Except(&["size"]),
            ),
            hold(
                plinth::class::SPLIT_BUTTON,
                Size::Box,
                Across::Only(&["busy", "disabled", "variant", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "context-menu",
        title: "ContextMenu",
        about: "A menu opened at the pointer by a right-click, or a long press on a touch screen. What it wraps is laid out as if it were not there.",
        covers: &["ContextMenu", "ContextMenuProps"],
        render: || html! { <menus::ContextMenuPage /> },
        holds: &[hold(class::SAMPLE, Size::Box, Across::All)],
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
        id: "tabs",
        title: "Tabs",
        about: "A tab list: the selected tab is marked by its colour and an underline inside its box, and tabs past the width scroll sideways. Tab to it, then use the arrow keys.",
        covers: &["Tabs", "SubTabs", "TabsProps", "TabItem", "tabs"],
        render: || html! { <tabs::TabsPage /> },
        holds: &[
            hold(
                plinth::class::TABS,
                Size::Height,
                Across::Except(&["small"]),
            ),
            hold(
                plinth::class::TAB,
                Size::Box,
                Across::Only(&["Selected", "disabled", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "nav-list",
        title: "NavList",
        about: "A side or settings navigation: the current entry is marked by aria-current, its colour and a tint, never its weight.",
        covers: &["NavList", "NavListProps", "NavItem", "nav_list"],
        render: || html! { <tabs::NavListPage /> },
        holds: &[hold(
            plinth::class::NAV_ITEM,
            Size::Box,
            Across::Only(&["Current", "disabled", "links", "icons", "focus", "click"]),
        )],
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
