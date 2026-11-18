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
mod overlays;
mod pagers;
mod picker;
mod pickers;
mod popover;
mod reorder;
mod reorder_tree;
mod rows;
mod select;
mod steppers;
mod structures;
mod styles;
mod tables;
mod tabs;
mod toasts;
mod toggles;
mod tooltips;
mod translations;

use yew::prelude::*;

use sockel::Head;

use crate::class;
use crate::probe::{Across, Hold, Size};

/// The kinds the navigation groups pages by, in display order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Group {
    Foundations,
    Actions,
    Feedback,
    Inputs,
    Layout,
    Lists,
    Media,
    Navigation,
    Overlays,
    Pickers,
}

impl Group {
    pub const ALL: [Group; 10] = [
        Group::Foundations,
        Group::Actions,
        Group::Feedback,
        Group::Inputs,
        Group::Layout,
        Group::Lists,
        Group::Media,
        Group::Navigation,
        Group::Overlays,
        Group::Pickers,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Group::Foundations => "Foundations",
            Group::Actions => "Actions",
            Group::Feedback => "Feedback & status",
            Group::Inputs => "Inputs & forms",
            Group::Layout => "Layout",
            Group::Lists => "Lists & tables",
            Group::Media => "Media",
            Group::Navigation => "Navigation",
            Group::Overlays => "Overlays & menus",
            Group::Pickers => "Pickers & selects",
        }
    }
}

/// The page shown for an unknown address.
pub const FALLBACK: &str = "stack";

/// The indexes into `PAGES` of a group's pages, alphabetical by title.
pub fn in_group(group: Group) -> Vec<usize> {
    let mut found: Vec<usize> = (0..PAGES.len())
        .filter(|&index| PAGES[index].group == group)
        .collect();
    found.sort_by_key(|&index| PAGES[index].title.to_lowercase());
    found
}

pub struct Page {
    /// The page's address, after the `#`.
    pub id: &'static str,
    pub title: &'static str,
    pub group: Group,
    /// What the page is about, under its title.
    pub about: &'static str,
    /// The names exported from sockel that the page shows (its component,
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
const UNPROBED: &[&str] = &["styles", "translations"];

/// Every page, in the navigation's order. A name sockel exports must be in
/// some page's `covers`, which the test below checks.
pub const PAGES: &[Page] = &[
    Page {
        id: "stack",
        title: "Stack",
        group: Group::Layout,
        about: "A column, its blocks a gap apart.",
        covers: &["Stack", "StackProps", "Gap"],
        render: || html! { <layout::StackPage /> },
        holds: &[
            hold(sockel::class::STACK, Size::Width, Across::All),
            hold(sockel::class::STACK, Size::Box, Across::Only(&["tag"])),
            hold(class::SAMPLE, Size::Box, Across::Only(&["tag"])),
        ],
    },
    Page {
        id: "cluster",
        title: "Cluster",
        group: Group::Layout,
        about: "A row that wraps, its items centred.",
        covers: &["Cluster", "ClusterProps", "Justify", "Align"],
        render: || html! { <layout::ClusterPage /> },
        holds: &[
            hold(
                sockel::class::CLUSTER,
                Size::Width,
                Across::Except(&["inline"]),
            ),
            hold(sockel::class::CLUSTER, Size::Box, Across::Only(&["tag"])),
            hold(class::SAMPLE, Size::Box, Across::Only(&["tag"])),
        ],
    },
    Page {
        id: "action-group",
        title: "ActionGroup",
        group: Group::Actions,
        about: "The buttons that end a form, a panel or a section, the primary first.",
        covers: &["ActionGroup", "ActionGroupProps", "action_group"],
        render: || html! { <actions::ActionGroupPage /> },
        holds: &[hold(
            sockel::class::ICONED,
            Size::Height,
            Across::Except(&["Buttons"]),
        )],
    },
    Page {
        id: "button",
        title: "Button",
        group: Group::Actions,
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
                sockel::class::BUTTON,
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
                sockel::class::BUTTON,
                Size::Height,
                Across::Except(&["size"]),
            ),
        ],
    },
    Page {
        id: "checkbox",
        title: "Checkbox",
        group: Group::Inputs,
        about: "A native checkbox in a box of a fixed size, with an indeterminate state. Click it; tab to it; press space.",
        covers: &["Checkbox", "CheckboxProps", "checkbox"],
        render: || html! { <toggles::CheckboxPage /> },
        holds: &[hold(sockel::class::CHECKBOX_MARK, Size::Both, Across::All)],
    },
    Page {
        id: "switch",
        title: "Switch",
        group: Group::Inputs,
        about: "An on or off switch (role=switch): its thumb moves by transform, and busy shows a spinner in it. Click it.",
        covers: &["Switch", "SwitchProps", "switch"],
        render: || html! { <toggles::SwitchPage /> },
        holds: &[
            hold(sockel::class::SWITCH_TRACK, Size::Both, Across::All),
            hold(sockel::class::SWITCH_THUMB, Size::Both, Across::All),
        ],
    },
    Page {
        id: "choice",
        title: "Choice",
        group: Group::Inputs,
        about: "A segmented pick-one group: the selected option is marked by its background alone, and the options keep one width. Tab to it, then use the arrow keys.",
        covers: &[
            "Choice",
            "ChoiceProps",
            "ChoiceOption",
            "ChoiceFit",
            "ChoiceStack",
            "choice",
        ],
        render: || html! { <toggles::ChoicePage /> },
        holds: &[
            hold(
                sockel::class::CHOICE,
                Size::Height,
                Across::Except(&["small"]),
            ),
            hold(
                sockel::class::CHOICE_OPTION,
                Size::Box,
                Across::Only(&["Selected", "invalid", "disabled", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "chip",
        title: "ChipToggle",
        group: Group::Inputs,
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
                sockel::class::CHIP_LIST,
                Size::Height,
                Across::Only(&["Chips=1", "Chips=0 (empty)"]),
            ),
            hold(
                sockel::class::CHIP,
                Size::Both,
                Across::Only(&[
                    "Count=7", "Count=0", "Count=42", "disabled", "focus", "click",
                ]),
            ),
        ],
    },
    Page {
        id: "chip-item",
        title: "Chip",
        group: Group::Inputs,
        about: "A chip that is not a toggle: a link, a button or an item, led by an avatar or an icon, an item removable by its cross. Its height holds whatever it holds.",
        covers: &["Chip", "ChipProps", "chip"],
        render: || html! { <toggles::ChipItemPage /> },
        holds: &[hold(sockel::class::CHIP, Size::Height, Across::All)],
    },
    Page {
        id: "field",
        title: "Field",
        group: Group::Inputs,
        about: "A label, the control, a hint and an error. With reserve_error the error line keeps its room: set an error and nothing below moves.",
        covers: &[
            "Field",
            "FieldProps",
            "field",
            "field_error",
            "field_describedby",
            "FieldInputs",
            "field_inputs",
        ],
        render: || html! { <fields::FieldPage /> },
        holds: &[
            hold(sockel::class::FIELD, Size::Width, Across::All),
            hold(
                sockel::class::FIELD,
                Size::Height,
                Across::Only(&["error", "required", "disabled", "busy", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "form-rows",
        title: "FormRows",
        group: Group::Inputs,
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
            hold(sockel::class::FORM_ROWS, Size::Width, Across::All),
            hold(
                sockel::class::FORM_CONTROL,
                Size::Width,
                Across::Except(&["wide_labels", "label"]),
            ),
            hold(
                sockel::class::FORM_ROW,
                Size::Height,
                Across::Only(&["error", "required", "disabled", "busy", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "pill",
        title: "Pill",
        group: Group::Feedback,
        about: "A tinted label of one height, led by a dot in its tone; a button when it has on_click. Change the tone, hover, click.",
        covers: &["Pill", "PillProps", "Tone", "pill"],
        render: || html! { <badges::PillPage /> },
        holds: &[
            hold(sockel::class::PILL, Size::Height, Across::All),
            hold(
                sockel::class::PILL,
                Size::Both,
                Across::Only(&["Tone", "on_click (a button)", "disabled", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "text-input",
        title: "TextInput",
        group: Group::Inputs,
        about: "A text input at a control's height, bare or with a unit inside the field, a native select in the field's look, and a Textarea. Type a long value; tab to it; set it invalid.",
        covers: &[
            "TextInput",
            "TextInputProps",
            "Textarea",
            "TextareaProps",
            "InputKind",
            "InputSize",
            "input",
            "input_box",
            "native_select",
            "textarea",
        ],
        render: || html! { <inputs::TextInputPage /> },
        holds: &[
            hold(
                sockel::class::INPUT,
                Size::Both,
                Across::Except(&["Size", "Text"]),
            ),
            hold(
                sockel::class::INPUT_BOX,
                Size::Both,
                Across::Except(&["Size"]),
            ),
            hold(
                sockel::class::INPUT_CONTROL,
                Size::Box,
                Across::Except(&["Size", "Unit"]),
            ),
            hold(
                sockel::class::TEXTAREA,
                Size::Both,
                Across::Except(&["Size"]),
            ),
        ],
    },
    Page {
        id: "badge",
        title: "Badge",
        group: Group::Feedback,
        about: "A short bordered mark, solid or over a corner, and the Dot beside it: filled or a ring, the same box.",
        covers: &["Badge", "BadgeProps", "badge", "Dot", "DotProps", "dot"],
        render: || html! { <badges::BadgePage /> },
        holds: &[
            hold(
                sockel::class::BADGE,
                Size::Height,
                Across::Except(&["corner"]),
            ),
            hold(
                sockel::class::BADGE,
                Size::Both,
                Across::Only(&["Tone", "solid", "focus", "click"]),
            ),
            hold(sockel::class::DOT, Size::Both, Across::All),
        ],
    },
    Page {
        id: "count",
        title: "Count",
        group: Group::Feedback,
        about: "A figure whose width is reserved for its max digits and a +, hidden at none without losing its box.",
        covers: &["Count", "CountProps", "count", "count_text"],
        render: || html! { <badges::CountPage /> },
        holds: &[hold(
            sockel::class::COUNT,
            Size::Box,
            Across::Except(&["Max"]),
        )],
    },
    Page {
        id: "status-label",
        title: "StatusLabel",
        group: Group::Feedback,
        about: "A status as wide as its widest state, so Loading... to Saved does not resize it; its spinner slot is always reserved.",
        covers: &["StatusLabel", "StatusLabelProps", "status_label"],
        render: || html! { <badges::StatusLabelPage /> },
        holds: &[
            hold(
                sockel::class::STATUS_LABEL,
                Size::Box,
                Across::Only(&["State", "focus", "click"]),
            ),
            hold(
                sockel::class::STATUS_SPINNER,
                Size::Box,
                Across::Only(&["State", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "search-field",
        title: "SearchField",
        group: Group::Inputs,
        about: "A search icon, the text and a clear button whose room is always kept: type, then clear it, and the field does not move. Escape clears too, unless escape clears is off (a combobox keeps escape to close its list).",
        covers: &["SearchField", "SearchFieldProps"],
        render: || html! { <inputs::SearchFieldPage /> },
        holds: &[
            hold(
                sockel::class::INPUT_BOX,
                Size::Box,
                Across::Except(&["Size"]),
            ),
            hold(
                sockel::class::INPUT_CONTROL,
                Size::Box,
                Across::Except(&["Size"]),
            ),
            hold(
                sockel::class::INPUT_CLEAR,
                Size::Box,
                Across::Except(&["Size"]),
            ),
        ],
    },
    Page {
        id: "stepper",
        title: "Stepper",
        group: Group::Inputs,
        about: "A number between a minus and a plus button, its room that of the longest bound. Step it, type past the bounds, use the arrow keys.",
        covers: &[
            "Stepper",
            "StepperProps",
            "stepper",
            "step_from",
            "step_by",
            "step_within",
            "key_value",
            "key_step",
            "parse_number",
            "parse_decimal",
            "format_number",
        ],
        render: || html! { <steppers::StepperPage /> },
        holds: &[
            hold(
                sockel::class::STEPPER,
                Size::Both,
                Across::Except(&["Size", "Max"]),
            ),
            hold(
                sockel::class::STEPPER_CONTROL,
                Size::Box,
                Across::Except(&["Size", "Max"]),
            ),
            hold(
                sockel::class::STEPPER_BUTTON,
                Size::Box,
                Across::Except(&["Size", "Max"]),
            ),
        ],
    },
    Page {
        id: "secret-input",
        title: "SecretInput",
        group: Group::Inputs,
        about: "A password field with show, copy and clear slots that keep their room; the secret hides itself after a while. Show it; clear it.",
        covers: &["SecretInput", "SecretInputProps", "Reveal", "REVEAL_MS"],
        render: || html! { <steppers::SecretInputPage /> },
        holds: &[
            hold(
                sockel::class::INPUT_BOX,
                Size::Both,
                Across::Except(&["Size"]),
            ),
            hold(
                sockel::class::INPUT_CONTROL,
                Size::Box,
                Across::Except(&["Size"]),
            ),
            hold(
                sockel::class::INPUT_ACTION,
                Size::Box,
                Across::Except(&["Size"]),
            ),
        ],
    },
    Page {
        id: "input-group",
        title: "InputGroup",
        group: Group::Inputs,
        about: "Controls joined in a row, the corners that join squared and one border between them; the members keep one height. A detached member stands apart.",
        covers: &["InputGroup", "InputGroupProps", "group_classes"],
        render: || html! { <inputs::InputGroupPage /> },
        holds: &[
            hold(
                sockel::class::INPUT_GROUP,
                Size::Both,
                Across::Except(&["Size"]),
            ),
            hold(
                sockel::class::INPUT,
                Size::Height,
                Across::Except(&["Size"]),
            ),
        ],
    },
    Page {
        id: "color-select",
        title: "ColorSelect",
        group: Group::Pickers,
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
                sockel::class::COLOR_SELECT,
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
                sockel::class::COLOR_SELECT,
                Size::Height,
                Across::Except(&["size"]),
            ),
            hold(
                sockel::class::COLOR_SWATCH,
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
        group: Group::Pickers,
        about: "A field's box showing what is chosen, opening a listbox in the overlay layer while focus stays on it: a placeholder, disabled options, joined inside an InputGroup. Its face is as wide as its widest option or the placeholder (or its container's, Fill), so a pick never changes its width. Listbox is the list's keyboard state: the active option, PageUp and PageDown, type-ahead.",
        covers: &[
            "Select",
            "SelectProps",
            "SelectOption",
            "SelectWidth",
            "select",
            "chosen",
            "filter_matches",
            "picked",
            "Listbox",
            "ListKey",
            "printable",
            "LISTBOX_PAGE",
            "listbox",
            "listbox_option",
            "option_id",
        ],
        render: || html! { <select::SelectPage /> },
        holds: &[
            hold(
                sockel::class::SELECT,
                Size::Box,
                Across::Only(&[
                    "value",
                    "invalid",
                    "disabled",
                    "busy",
                    "an option disabled",
                    "descriptions",
                    "groups",
                    "filter",
                    "sheet",
                    "focus",
                    "click",
                ]),
            ),
            hold(
                sockel::class::SELECT,
                Size::Height,
                Across::Except(&["size"]),
            ),
            hold(
                sockel::class::INPUT_GROUP,
                Size::Height,
                Across::Except(&["size"]),
            ),
        ],
    },
    Page {
        id: "search-select",
        title: "SearchSelect",
        group: Group::Pickers,
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
                sockel::class::SEARCH_SELECT,
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
                sockel::class::SEARCH_SELECT,
                Size::Height,
                Across::Except(&["size"]),
            ),
        ],
    },
    Page {
        id: "filter-picker",
        title: "FilterPicker",
        group: Group::Pickers,
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
            hold(sockel::class::FILTER_PICKER, Size::Box, Across::All),
            hold(sockel::class::FILTER_PICKER_COUNT, Size::Box, Across::All),
        ],
    },
    Page {
        id: "picker",
        title: "Picker",
        group: Group::Pickers,
        about: "A field of one width showing the current option, its icon and label, opening a ChoiceList below it: options with icon, mark, detail and badge slots, a search field over them past eight (ranked: labels starting with the text first), --picker-rows rows high however many match.",
        covers: &[
            "Picker",
            "PickerProps",
            "PickerOption",
            "PickerVariant",
            "SEARCH_FROM",
            "picker",
            "picker_matches",
        ],
        render: || html! { <picker::PickerPage /> },
        holds: &[
            hold(
                sockel::class::PICKER,
                Size::Box,
                Across::Only(&[
                    "options",
                    "Option text",
                    "icons",
                    "always_search",
                    "invalid",
                    "disabled",
                    "focus",
                    "click",
                ]),
            ),
            hold(
                sockel::class::PICKER,
                Size::Height,
                Across::Except(&["variant"]),
            ),
            hold(
                sockel::class::CHOICE_LIST_OPTIONS,
                Size::Flush(sockel::class::PICKER_POPUP),
                Across::Only(&["click"]),
            ),
        ],
    },
    Page {
        id: "picker-menu",
        title: "PickerMenu",
        group: Group::Pickers,
        about: "A button for an action that takes an option, such as adding one, opening the same searchable ChoiceList as a Picker; a pick closes it.",
        covers: &["PickerMenu", "PickerMenuProps"],
        render: || html! { <picker::PickerMenuPage /> },
        holds: &[
            hold(
                sockel::class::BUTTON,
                Size::Box,
                Across::Except(&["Option text", "chevron (PickerMenu, with an icon)"]),
            ),
            hold(
                sockel::class::CHOICE_LIST_SEARCH,
                Size::Flush(sockel::class::PICKER_POPUP),
                Across::Only(&["click"]),
            ),
            hold(
                sockel::class::CHOICE_LIST_OPTIONS,
                Size::Flush(sockel::class::PICKER_POPUP),
                Across::Only(&["click"]),
            ),
        ],
    },
    Page {
        id: "choice-popup",
        title: "ChoicePopup",
        group: Group::Pickers,
        about: "A ChoiceList's popup below an opener the app renders itself (here a toggle showing how many are chosen), with a note and a Done button beside the list; Escape, Tab or a press outside closes it.",
        covers: &["ChoicePopup", "ChoicePopupProps"],
        render: || html! { <picker::ChoicePopupPage /> },
        holds: &[hold(sockel::class::BUTTON, Size::Height, Across::All)],
    },
    Page {
        id: "picker-dialog",
        title: "PickerDialog",
        group: Group::Pickers,
        about: "The searchable picker shell for a large set (languages, countries): a dialog holding a ChoiceList, single or multi-select, its search field focused. Shown inline too: filtering, no matches and multi never change its box.",
        covers: &[
            "PickerDialog",
            "PickerDialogProps",
            "ChoiceList",
            "ChoiceListProps",
        ],
        render: || html! { <picker::PickerDialogPage /> },
        holds: &[
            hold(
                sockel::class::CHOICE_LIST_OPTIONS,
                Size::Box,
                Across::Except(&["always_search"]),
            ),
            hold(sockel::class::CHOICE_LIST, Size::Width, Across::All),
        ],
    },
    Page {
        id: "icon-button",
        title: "IconButton",
        group: Group::Actions,
        about: "A button that leads with (or ends in) an icon the app supplies. Click it; tab to it.",
        covers: &["IconButton", "IconButtonProps", "icon_button"],
        render: || html! { <actions::IconButtonPage /> },
        holds: &[
            hold(
                sockel::class::ICONED,
                Size::Box,
                Across::Only(&["disabled", "dialog", "label (aria-label)", "focus", "click"]),
            ),
            hold(
                sockel::class::ICONED,
                Size::Height,
                Across::Except(&["Small"]),
            ),
        ],
    },
    Page {
        id: "icon",
        title: "Icon",
        group: Group::Media,
        about: "sockel's glyphs and the app's own icons by name, from the app's IconSet or the built-in one, always in the same box. A missing icon is the empty box; swap the set and the sizes do not move.",
        covers: &[
            "Icon",
            "IconProps",
            "IconSet",
            "Glyph",
            "icon",
            "use_icon",
            "use_icon_named",
            "use_icons",
        ],
        render: || html! { <icons::IconPage /> },
        holds: &[
            hold(
                sockel::class::GLYPH,
                Size::Both,
                Across::Only(&["Icon set", "icon", "disabled", "focus", "click"]),
            ),
            hold(
                sockel::class::ICONED,
                Size::Height,
                Across::Only(&["Icon set", "icon", "disabled", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "image",
        title: "Image",
        group: Group::Media,
        about: "A picture in a box of a fixed ratio: its placeholder while it loads, when it fails and when there is none is drawn in the same box. Switch the source and the ratio.",
        covers: &["Image", "ImageProps", "Ratio", "Status", "image"],
        render: || html! { <media::ImagePage /> },
        holds: &[
            hold(
                sockel::class::IMAGE,
                Size::Width,
                Across::Except(&["Width"]),
            ),
            hold(
                sockel::class::IMAGE,
                Size::Box,
                Across::Only(&["Source", "alt", "lazy", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "avatar",
        title: "Avatar",
        group: Group::Media,
        about: "A round portrait of a fixed size, its initials behind the picture: loading, loaded, failed or missing, it is the same box.",
        covers: &["Avatar", "AvatarProps", "AvatarSize", "avatar", "initials"],
        render: || html! { <media::AvatarPage /> },
        holds: &[
            hold(sockel::class::AVATAR, Size::Both, Across::Except(&["Size"])),
            hold(
                sockel::class::AVATAR,
                Size::Box,
                Across::Only(&["Source", "Name", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "notice",
        title: "Notice",
        group: Group::Feedback,
        about: "An inline message in a tone, with an optional icon, action and close. Closing one collapses its space, by design.",
        covers: &["Notice", "NoticeProps", "notice"],
        render: || html! { <notices::NoticePage /> },
        holds: &[
            hold(
                sockel::class::NOTICE,
                Size::Width,
                Across::Except(&["Shown", "click"]),
            ),
            hold(
                sockel::class::NOTICE,
                Size::Height,
                Across::Only(&["tone", "focus"]),
            ),
        ],
    },
    Page {
        id: "error-box",
        title: "ErrorBox",
        group: Group::Feedback,
        about: "An error with its cause and a retry; a retry in progress keeps the button's box.",
        covers: &["ErrorBox", "ErrorBoxProps", "error_box"],
        render: || html! { <notices::ErrorBoxPage /> },
        holds: &[
            hold(sockel::class::ERROR_BOX, Size::Width, Across::All),
            hold(
                sockel::class::BUTTON,
                Size::Box,
                Across::Only(&["retrying", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "empty-state",
        title: "EmptyState",
        group: Group::Feedback,
        about: "What stands where there is nothing. Compact, it is one row high, like the row it replaces.",
        covers: &["EmptyState", "EmptyStateProps", "empty_state"],
        render: || html! { <notices::EmptyStatePage /> },
        holds: &[
            hold(sockel::class::EMPTY_STATE, Size::Width, Across::All),
            hold(
                sockel::class::EMPTY_STATE,
                Size::Height,
                Across::Except(&["compact"]),
            ),
        ],
    },
    Page {
        id: "spinner",
        title: "Spinner",
        group: Group::Feedback,
        about: "A spinning glyph in the icon box, in the colour of the text around it; the busy mark of a Button.",
        covers: &["Spinner", "SpinnerProps"],
        render: || html! { <loading::SpinnerPage /> },
        holds: &[hold(sockel::class::SPINNER, Size::Both, Across::All)],
    },
    Page {
        id: "skeleton",
        title: "Skeleton",
        group: Group::Feedback,
        about: "A placeholder of the box its content will have: a line, a block, a row as high as a RowList row, or a picture of an Image's ratio.",
        covers: &["Skeleton", "SkeletonProps", "SkeletonShape", "skeleton"],
        render: || html! { <loading::SkeletonPage /> },
        holds: &[hold(
            sockel::class::SKELETON,
            Size::Box,
            Across::Only(&["RowList beside", "focus", "click"]),
        )],
    },
    Page {
        id: "loadable",
        title: "Loadable",
        group: Group::Feedback,
        about: "Loading, empty, error and ready content in one box: it keeps the height it last had while it loads again, and the empty state waits a moment so it never flashes before data.",
        covers: &["Loadable", "LoadableProps", "LoadState"],
        render: || html! { <loading::LoadablePage /> },
        holds: &[
            hold(sockel::class::LOADABLE, Size::Width, Across::All),
            hold(
                sockel::class::LOADABLE,
                Size::Height,
                Across::Only(&["state=Loading", "state=Ready", "placeholder", "retrying"]),
            ),
        ],
    },
    Page {
        id: "toast",
        title: "Toast",
        group: Group::Feedback,
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
                sockel::class::TOAST,
                Size::Width,
                Across::Only(&["Live toast", "raised", "focus", "click"]),
            ),
            hold(
                sockel::class::TOAST,
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
        group: Group::Feedback,
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
                sockel::class::SAVE_BAR,
                Size::Box,
                Across::Except(&["status", "label"]),
            ),
            hold(sockel::class::SAVE_BAR, Size::Width, Across::All),
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
        group: Group::Lists,
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
            hold(sockel::class::ROW_LIST, Size::Width, Across::All),
            hold(
                sockel::class::ROW_LIST_ITEM,
                Size::Box,
                Across::Only(&["divided", "boxed", "Actions", "overlay", "focus", "click"]),
            ),
            hold(
                sockel::class::ROW_LIST_ITEM,
                Size::Height,
                Across::Except(&["Detail", "Name", "top", "Rows", "State"]),
            ),
        ],
    },
    Page {
        id: "pick-row",
        title: "PickRow",
        group: Group::Lists,
        about: "A row that is one press, choosing the thing it names: the chosen row is marked by its background alone.",
        covers: &["PickRow", "PickRowProps", "pick_row"],
        render: || html! { <rows::PickRowPage /> },
        holds: &[
            hold(sockel::class::PICK_ROW, Size::Width, Across::All),
            hold(
                sockel::class::PICK_ROW,
                Size::Height,
                Across::Except(&["Detail", "Name"]),
            ),
            hold(
                sockel::class::PICK_ROW,
                Size::Box,
                Across::Only(&["disabled", "mark", "Trailing", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "selection-bar",
        title: "SelectionBar",
        group: Group::Lists,
        about: "The picked count and its actions under a list. It keeps its room at one height, hidden while nothing is picked, so picking moves nothing.",
        covers: &["SelectionBar", "SelectionBarProps", "selection_bar"],
        render: || html! { <rows::SelectionBarPage /> },
        holds: &[
            hold(sockel::class::SELECTION_BAR, Size::Height, Across::All),
            hold(
                sockel::class::SELECTION_BAR,
                Size::Box,
                Across::Only(&["picked", "Label", "busy", "disabled", "focus", "click"]),
            ),
            hold(
                sockel::class::ROW_LIST,
                Size::Box,
                Across::Only(&["picked", "busy", "sticky", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "action-row",
        title: "ActionRow",
        group: Group::Lists,
        about: "Actions acting on what stands around them; as an overlay it lies over its row's end, invisible and inert until the row is hovered or focused.",
        covers: &["ActionRow", "ActionRowProps", "action_row"],
        render: || html! { <rows::ActionRowPage /> },
        holds: &[
            hold(
                sockel::class::ROW_LIST_ITEM,
                Size::Box,
                Across::Only(&["Buttons", "Labels", "focus", "click"]),
            ),
            hold(sockel::class::ROW_LIST, Size::Width, Across::All),
        ],
    },
    Page {
        id: "card-grid",
        title: "CardGrid",
        group: Group::Layout,
        about: "Cards in as many columns as fit, each at least a size's width, sharing the room.",
        covers: &["CardGrid", "CardGridProps", "GridSize", "card_grid"],
        render: || html! { <rows::CardGridPage /> },
        holds: &[
            hold(sockel::class::CARD_GRID, Size::Width, Across::All),
            hold(
                sockel::class::CARD_GRID,
                Size::Box,
                Across::Only(&["focus", "click"]),
            ),
        ],
    },
    Page {
        id: "reorder-tree",
        title: "ReorderTree",
        group: Group::Lists,
        about: "Rows moved within and between nested lists: a drag aims at the innermost list under the pointer that takes the row; the arrows step through every place in reading order, into a nested list and back out. accepts keeps groups among groups and entries in groups.",
        covers: &[
            "ReorderTree",
            "ReorderTreeProps",
            "ReorderList",
            "ReorderListProps",
            "ReorderTreeHandle",
            "ReorderTreeHandleProps",
            "ReorderTreeMove",
            "ReorderSpot",
            "TreeList",
            "tree_slots",
        ],
        render: || html! { <reorder_tree::ReorderTreePage /> },
        holds: &[hold(
            sockel::class::REORDER,
            Size::Box,
            Across::Only(&["disabled", "titles", "focus", "click"]),
        )],
    },
    Page {
        id: "reorder",
        title: "Reorder",
        group: Group::Lists,
        about: "Rows moved by a drag on their handle or by keyboard (Space lifts, the arrows aim, Space drops, Escape cancels; Alt and an arrow moves at once). Nothing reflows until the drop. In children mode the rows are the app's own ReorderItems, a ReorderHandle in each.",
        covers: &[
            "Reorder",
            "ReorderProps",
            "ReorderRow",
            "ReorderText",
            "ReorderItem",
            "ReorderItemProps",
            "ReorderHandle",
            "ReorderHandleProps",
            "DragHandle",
            "DragHandleProps",
            "drag_handle",
            "drag_target",
            "edge_step",
            "grid_target",
            "reorder_row",
            "reorder_text",
            "landing",
            "slot_at",
            "move_item",
            "move_items",
            "key_target",
        ],
        render: || html! { <reorder::ReorderPage /> },
        holds: &[
            hold(
                sockel::class::REORDER,
                Size::Width,
                Across::Except(&["Rows"]),
            ),
            hold(
                sockel::class::REORDER,
                Size::Box,
                Across::Only(&[
                    "Keys",
                    "divided",
                    "end",
                    "disabled",
                    "quick_keys",
                    "titles",
                    "focus",
                    "click",
                ]),
            ),
            hold(
                sockel::class::REORDER_ROW,
                Size::Box,
                Across::Only(&[
                    "Keys",
                    "divided",
                    "end",
                    "disabled",
                    "quick_keys",
                    "titles",
                    "focus",
                    "click",
                ]),
            ),
            hold(
                sockel::class::DRAG_HANDLE,
                Size::Box,
                Across::Only(&[
                    "Keys",
                    "divided",
                    "end",
                    "disabled",
                    "quick_keys",
                    "titles",
                    "focus",
                    "click",
                ]),
            ),
        ],
    },
    Page {
        id: "table",
        title: "Table",
        group: Group::Lists,
        about: "A table in a scroll area with a sticky head, hover rows and numeric columns; the bulk actions' room is reserved above it. Press a heading to sort.",
        covers: &[
            "Table",
            "TableProps",
            "TableRow",
            "TableRowProps",
            "TableCell",
            "TableCellProps",
            "table",
            "table_frame",
            "table_wrap",
            "table_bulk",
            "table_cell",
            "table_cell_part",
            "table_mode",
        ],
        render: || html! { <tables::TablePage /> },
        holds: &[
            hold(sockel::class::TABLE_FRAME, Size::Width, Across::All),
            hold(
                sockel::class::SORT_HEADER,
                Size::Box,
                Across::Only(&["Sort", "Picked", "disabled", "focus", "click"]),
            ),
            hold(
                sockel::class::TABLE_BULK,
                Size::Box,
                Across::Only(&["Picked", "Sort", "focus", "click"]),
            ),
            hold(
                sockel::class::TABLE_ROW,
                Size::Height,
                Across::Only(&["Sort", "Picked", "fixed", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "sort-header",
        title: "SortHeader",
        group: Group::Lists,
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
                sockel::class::SORT_HEADER,
                Size::Box,
                Across::Only(&["Sort", "disabled", "Descending first", "focus", "click"]),
            ),
            hold(sockel::class::TABLE_FRAME, Size::Width, Across::All),
        ],
    },
    Page {
        id: "pager",
        title: "Pager",
        group: Group::Navigation,
        about: "Page buttons in a fixed number of slots: first, previous, numbers with gaps, next and last. The slot count and each button's width never change with the page, and the current page is marked without bold. A summary pager says which items the page shows and where it stands, then previous and next.",
        covers: &[
            "Pager",
            "PagerProps",
            "PagerSummary",
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
                sockel::class::PAGER,
                Size::Width,
                Across::Only(&["disabled", "busy", "focus", "click"]),
            ),
            hold(sockel::class::PAGER, Size::Height, Across::All),
        ],
    },
    Page {
        id: "disclosure",
        title: "Disclosure",
        group: Group::Navigation,
        about: "A summary row that toggles its body (aria-expanded). The row is one height shut or open: the mark turns inside its box and the summary is cut to one line.",
        covers: &[
            "Disclosure",
            "DisclosureProps",
            "disclosure",
            "disclosure_summary",
        ],
        render: || html! { <disclosures::DisclosurePage /> },
        holds: &[
            hold(sockel::class::DISCLOSURE, Size::Width, Across::All),
            hold(
                sockel::class::DISCLOSURE_HEAD,
                Size::Box,
                Across::Only(&[
                    "Title", "Summary", "open", "actions", "disabled", "focus", "click",
                ]),
            ),
            hold(
                sockel::class::DISCLOSURE_MARK,
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
        group: Group::Inputs,
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
                sockel::class::DROP_ZONE,
                Size::Box,
                Across::Except(&["Text", "pick"]),
            ),
            hold(sockel::class::DROP_ZONE, Size::Width, Across::All),
        ],
    },
    Page {
        id: "toolbar",
        title: "Toolbar",
        group: Group::Layout,
        about: "One line of controls; a ToolbarSpacer pushes the rest to its end. Tab through its buttons.",
        covers: &[
            "Toolbar",
            "ToolbarProps",
            "ToolbarSpacer",
            "ToolbarSpacerProps",
        ],
        render: || html! { <layout::ToolbarPage /> },
        holds: &[
            hold(sockel::class::TOOLBAR, Size::Width, Across::All),
            hold(
                sockel::class::TOOLBAR,
                Size::Height,
                Across::Only(&["Labels", "ToolbarSpacer", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "tool-band",
        title: "ToolBand",
        group: Group::Layout,
        about: "A Toolbar with band: its controls wrap onto further lines instead of scrolling, in ToolbarParts that each keep to one line, ruled off from the part before from the phone breakpoint up; chrome sets it on --band-chrome-ground with a rule under it, --band-chrome-gap apart; search sets it in --band-search-text. tool_band(chrome, search) gives these classes to markup that is not the component. Its buttons keep one width when disabled, so the band never rewraps. Tab through it.",
        covers: &[
            "ToolbarPart",
            "ToolbarPartProps",
            "toolbar_part",
            "tool_band",
        ],
        render: || html! { <structures::ToolBandPage /> },
        holds: &[
            hold(sockel::class::TOOLBAR, Size::Width, Across::All),
            hold(
                sockel::class::TOOLBAR,
                Size::Height,
                Across::Only(&["disabled", "focus", "click"]),
            ),
            hold(
                sockel::class::TOOLBAR_PART,
                Size::Box,
                Across::Only(&["disabled", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "panel",
        title: "Panel",
        group: Group::Layout,
        about: "A ringed ground over a list or a view holding its filters or a short form: a column, or a line that wraps; tinted or bare; small sets it in --text-sm. Its ring, type and tint text come from --panel-border, --panel-text and --panel-tint-text, the room under it from --panel-space-after. As wide as what it holds, never wider than its column; its ring is always there, so the ground changes nothing of its box.",
        covers: &["Panel", "PanelProps", "PanelGround", "panel"],
        render: || html! { <structures::PanelPage /> },
        holds: &[hold(
            sockel::class::PANEL_BOX,
            Size::Box,
            Across::Only(&["ground", "disabled", "focus", "click"]),
        )],
    },
    Page {
        id: "seam",
        title: "Seam",
        group: Group::Layout,
        about: "Two panes and the divider between them: drag it (one change a frame), focus it and use the arrow keys, Home and End, or double-click it to even the panes. Each pane keeps --seam-min, so neither collapses, and the divider keeps its box.",
        covers: &["Seam", "SeamProps", "SeamAxis", "seam"],
        render: || html! { <structures::SeamPage /> },
        holds: &[
            hold(sockel::class::SEAM, Size::Box, Across::Except(&["axis"])),
            hold(
                sockel::class::SEAM_GRIP,
                Size::Both,
                Across::Except(&["axis"]),
            ),
            hold(
                sockel::class::SEAM_PANE,
                Size::Width,
                Across::Only(&["Text", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "use-width",
        title: "use_width",
        group: Group::Layout,
        about: "An element's width as a hook, from a ResizeObserver: at most one new value a frame, and only when it changed. Drag the narrow frame's corner.",
        covers: &["use_width"],
        render: || html! { <structures::WidthPage /> },
        holds: &[hold(class::READOUT, Size::Height, Across::All)],
    },
    Page {
        id: "card",
        title: "Card",
        group: Group::Layout,
        about: "A surface lifted off the page.",
        covers: &["Card", "CardProps", "Inset"],
        render: || html! { <layout::CardPage /> },
        holds: &[
            hold(sockel::class::CARD, Size::Width, Across::All),
            hold(sockel::class::CARD, Size::Box, Across::Only(&["tag"])),
        ],
    },
    Page {
        id: "page-head",
        title: "PageHead",
        group: Group::Layout,
        about: "A page's title with its help, and its actions.",
        covers: &["PageHead", "HeadProps", "Head", "HeadLevel"],
        render: || html! { <heads::HeadPage kind={Head::Page} /> },
        holds: &[
            hold(sockel::class::PAGE_HEAD, Size::Width, Across::All),
            hold(
                sockel::class::PAGE_HEAD,
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
        group: Group::Layout,
        about: "A section's title with its help, and its actions.",
        covers: &["SectionHead", "HeadProps", "Head", "HeadLevel"],
        render: || html! { <heads::HeadPage kind={Head::Section} /> },
        holds: &[
            hold(sockel::class::SECTION_HEAD, Size::Width, Across::All),
            hold(
                sockel::class::SECTION_HEAD,
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
        group: Group::Lists,
        about: "A list's search field and its filters: click a filter to change its text, and the field keeps its width.",
        covers: &["LibraryToolbar", "LibraryToolbarProps"],
        render: || html! { <library::LibraryToolbarPage /> },
        holds: &[
            hold(sockel::class::LIBRARY_TOOLBAR, Size::Width, Across::All),
            hold(
                sockel::class::LIBRARY_SEARCH,
                Size::Box,
                Across::Only(&["icon", "autofocus", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "popover",
        title: "Popover",
        group: Group::Overlays,
        about: "A surface beside its anchor in the overlay layer: placed before it shows, flipped and kept inside the window, following its anchor as the page scrolls; a press outside or Escape closes it, and focus goes back to the anchor. A sheet from the bottom on a phone.",
        covers: &[
            "Popover",
            "PopoverProps",
            "Side",
            "PopoverAlign",
            "Dismiss",
            "Placed",
            "Rect",
            "Fit",
            "place",
            "place_fit",
            "use_popover",
            "use_popover_within",
            "placement_bounds",
            "overlay",
            "popover",
            "OVERLAY_ID",
            "OverlayHost",
            "OverlayHostProps",
            "Overlays",
            "overlay_host",
            "use_overlays",
        ],
        render: || html! { <popover::PopoverPage /> },
        holds: &[
            // Widened by match_anchor, to show the popover matching it.
            hold(
                sockel::class::BUTTON,
                Size::Box,
                Across::Except(&["match_anchor"]),
            ),
            hold(class::SAMPLE, Size::Box, Across::All),
        ],
    },
    Page {
        id: "dialog",
        title: "Dialog",
        group: Group::Overlays,
        about: "A modal box over a scrim: its head, a body that scrolls (or a flush one that lays itself out) and actions pinned below it, in four sizes, a sheet from the bottom on a phone. The page behind does not move or lose its scrollbar's room; Tab stays inside; Escape closes the innermost overlay first (open the popover or the second dialog from it), and focus goes back where it was.",
        covers: &[
            "Dialog",
            "DialogProps",
            "DialogSize",
            "DialogDismiss",
            "dialog",
        ],
        render: || html! { <dialog::DialogPage /> },
        holds: &[
            hold(sockel::class::BUTTON, Size::Box, Across::All),
            hold(class::SAMPLE, Size::Box, Across::All),
        ],
    },
    Page {
        id: "overlays",
        title: "Overlays",
        group: Group::Overlays,
        about: "Overlays nested in one another, with and without an OverlayHost. Open a dialog, then a menu or a toast from it (the toast's Undo still works), a panel of the app's own (use_overlay) with a menu in it, or a popover with a tooltip on its button: Escape and a press outside close the innermost first and focus goes back. Turn the host off to see the fallback at the end of <body>; set the frame's colours to see a host inside it take them and the fallback not.",
        covers: &["OverlayHost", "use_overlays", "use_overlay"],
        render: || html! { <overlays::OverlaysPage /> },
        holds: &[
            hold(sockel::class::BUTTON, Size::Box, Across::All),
            hold(class::SAMPLE, Size::Box, Across::All),
        ],
    },
    Page {
        id: "confirm",
        title: "Confirm",
        group: Group::Overlays,
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
            hold(sockel::class::BUTTON, Size::Box, Across::All),
            hold(class::SAMPLE, Size::Box, Across::All),
        ],
    },
    Page {
        id: "confirm-popover",
        title: "ConfirmPopover",
        group: Group::Overlays,
        about: "A confirm beside the button that asked, in a popover of a fixed width: the focus on Cancel when accepting destroys, Escape and a press outside cancel and focus returns to the button. Compact asks on one line with two named icons, as wide as it says. ConfirmButton is a Button that opens it. A sheet on a phone.",
        covers: &[
            "ConfirmPopover",
            "ConfirmPopoverProps",
            "confirm_popover",
            "ConfirmButton",
            "ConfirmButtonProps",
        ],
        render: || html! { <confirms::ConfirmPopoverPage /> },
        holds: &[
            hold(sockel::class::BUTTON, Size::Box, Across::All),
            hold(class::SAMPLE, Size::Box, Across::All),
        ],
    },
    Page {
        id: "confirm-inline",
        title: "ConfirmInline",
        group: Group::Actions,
        about: "No and Yes replacing a button in its own box: both are always laid out in one cell, so the row is as wide as the wider and as high as the button whether it asks or not. No takes the focus; Escape goes back.",
        covers: &["ConfirmInline", "ConfirmInlineProps", "confirm_inline"],
        render: || html! { <confirms::ConfirmInlinePage /> },
        holds: &[
            hold(sockel::class::CONFIRM_INLINE, Size::Box, ASKING),
            hold(sockel::class::CONFIRM_IDLE, Size::Box, ASKING),
            hold(class::SAMPLE, Size::Box, ASKING),
        ],
    },
    Page {
        id: "tooltip",
        title: "Tooltip",
        group: Group::Overlays,
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
            hold(sockel::class::BUTTON, Size::Box, Across::All),
            hold(class::SAMPLE, Size::Box, Across::All),
        ],
    },
    Page {
        id: "hover-card",
        title: "HoverCard",
        group: Group::Overlays,
        about: "A richer card beside its anchor, opened by hover or keyboard focus and kept open while the pointer is over it, so a link in it can be reached.",
        covers: &[
            "HoverCard",
            "HoverCardProps",
            "HOVER_CARD_DELAY",
            "hover_card",
        ],
        render: || html! { <tooltips::HoverCardPage /> },
        holds: &[
            hold(sockel::class::BUTTON, Size::Box, Across::All),
            hold(class::SAMPLE, Size::Box, Across::All),
        ],
    },
    Page {
        id: "reason-button",
        title: "ReasonButton",
        group: Group::Actions,
        about: "A Button that says why it cannot act, or what it is doing, in a tooltip. One button across available, unavailable (aria-disabled, so it keeps focus) and working. Tab to it while unavailable.",
        covers: &["ReasonButton", "ReasonButtonProps", "reason_text"],
        render: || html! { <tooltips::ReasonButtonPage /> },
        holds: &[
            hold(
                sockel::class::BUTTON,
                Size::Box,
                Across::Only(&["state", "reason", "focus", "click"]),
            ),
            hold(
                sockel::class::BUTTON,
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
        group: Group::Overlays,
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
                sockel::class::BUTTON,
                Size::Box,
                Across::Except(&["variant", "size", "icon only"]),
            ),
            hold(
                sockel::class::BUTTON,
                Size::Height,
                Across::Except(&["size"]),
            ),
            hold(class::SAMPLE, Size::Box, Across::Except(&["size"])),
        ],
    },
    Page {
        id: "row-actions",
        title: "RowActions",
        group: Group::Overlays,
        about: "A row's actions: small icon buttons of one size, always shown, the danger ones last and muted until pointed at, then the overflow menu behind its ellipsis.",
        covers: &["RowActions", "RowActionsProps", "RowAction"],
        render: || html! { <menus::RowActionsPage /> },
        holds: &[
            hold(sockel::class::ROW_ACTIONS, Size::Height, Across::All),
            hold(
                sockel::class::ROW_ACTIONS,
                Size::Width,
                Across::Except(&["more (a menu)"]),
            ),
        ],
    },
    Page {
        id: "split-button",
        title: "SplitButton",
        group: Group::Actions,
        about: "A main action joined to a menu of its variants: one control of one height, the two parts sharing a border.",
        covers: &["SplitButton", "SplitButtonProps"],
        render: || html! { <menus::SplitButtonPage /> },
        holds: &[
            hold(
                sockel::class::SPLIT_BUTTON,
                Size::Height,
                Across::Except(&["size"]),
            ),
            hold(
                sockel::class::SPLIT_BUTTON,
                Size::Box,
                Across::Only(&["busy", "disabled", "variant", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "context-menu",
        title: "ContextMenu",
        group: Group::Overlays,
        about: "A menu opened at the pointer by a right-click, or a long press on a touch screen. What it wraps is laid out as if it were not there.",
        covers: &["ContextMenu", "ContextMenuProps"],
        render: || html! { <menus::ContextMenuPage /> },
        holds: &[hold(class::SAMPLE, Size::Box, Across::All)],
    },
    Page {
        id: "classes",
        title: "Class functions",
        group: Group::Foundations,
        about: "The classes of a stack, cluster, toolbar and card, for an element that cannot be the component.",
        covers: &["stack", "stack_with", "cluster", "toolbar", "card"],
        render: || html! { <layout::ClassesPage /> },
        holds: &[hold(sockel::class::STACK, Size::Width, Across::All)],
    },
    Page {
        id: "foundations",
        title: "Foundations",
        group: Group::Foundations,
        about: "The utilities and mixins against jitter: text fitting, tabular figures, a scroll area, the current item, showing by width, the focus ring; and the colours sockel derives.",
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
        group: Group::Navigation,
        about: "A tab list: the selected tab is marked by its colour and an underline inside its box, and tabs past the width scroll sideways. Tab to it, then use the arrow keys.",
        covers: &[
            "Tabs",
            "SubTabs",
            "LinkTabs",
            "LinkTabsProps",
            "TabsProps",
            "TabItem",
            "tabs",
        ],
        render: || html! { <tabs::TabsPage /> },
        holds: &[
            hold(
                sockel::class::TABS,
                Size::Height,
                Across::Except(&["small", "segmented"]),
            ),
            hold(
                sockel::class::TAB,
                Size::Box,
                Across::Only(&["Selected", "disabled", "focus", "click"]),
            ),
        ],
    },
    Page {
        id: "nav-list",
        title: "NavList",
        group: Group::Navigation,
        about: "A side or settings navigation: the current entry is marked by aria-current, its colour and a tint, never its weight.",
        covers: &["NavList", "NavListProps", "NavItem", "nav_list"],
        render: || html! { <tabs::NavListPage /> },
        holds: &[hold(
            sockel::class::NAV_ITEM,
            Size::Box,
            Across::Only(&["Current", "disabled", "links", "icons", "focus", "click"]),
        )],
    },
    Page {
        id: "styles",
        title: "Styles",
        group: Group::Foundations,
        about: "sockel's compiled stylesheet, and the component that adds the page's one stylesheet.",
        covers: &["STYLESHEET", "STYLE_ID", "Styles", "StylesProps"],
        render: || html! { <styles::StylesPage /> },
        holds: &[],
    },
    Page {
        id: "translations",
        title: "Translations",
        group: Group::Foundations,
        about: "sockel's words come from its phrasebook catalog, in the language the app provides; pick one under Language.",
        covers: &[],
        render: || html! { <translations::TranslationsPage /> },
        holds: &[],
    },
];

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{FALLBACK, Group, PAGES, UNPROBED, in_group};

    #[test]
    fn each_group_is_alphabetical_and_each_page_is_in_one() {
        let mut seen = Vec::new();
        for group in Group::ALL {
            let titles: Vec<String> = in_group(group)
                .iter()
                .map(|&i| PAGES[i].title.to_lowercase())
                .collect();
            assert!(titles.windows(2).all(|w| w[0] <= w[1]), "{group:?}");
            seen.extend(in_group(group));
        }
        seen.sort();
        assert_eq!(seen, (0..PAGES.len()).collect::<Vec<_>>());
    }

    #[test]
    fn the_fallback_is_a_page() {
        assert!(PAGES.iter().any(|p| p.id == FALLBACK));
    }

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
            "exported from sockel without a showcase page: {missing:?}"
        );
        let stale: Vec<_> = covered.difference(&exported).collect();
        assert!(
            stale.is_empty(),
            "showcase pages cover names sockel does not export: {stale:?}"
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
            sockel::class::SCROLL_AREA
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
