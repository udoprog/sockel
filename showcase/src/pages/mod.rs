mod actions;
mod buttons;
mod foundations;
mod heads;
mod icons;
mod layout;
mod library;
mod media;
mod styles;
mod toggles;

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

    #[test]
    fn page_ids_are_unique() {
        let ids: BTreeSet<_> = PAGES.iter().map(|p| p.id).collect();
        assert_eq!(ids.len(), PAGES.len());
    }
}
