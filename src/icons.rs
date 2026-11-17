//! The icons of plinth's own chrome (`style/_icons.scss`): an [`IconSet`]
//! the app provides once at its root, and a built-in set without one.

use crate::class;
use yew::prelude::*;

/// One of plinth's glyphs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Glyph {
    Close,
    ChevronDown,
    ChevronUp,
    ChevronRight,
    Check,
    Plus,
    Minus,
    Spinner,
    Grip,
    Ellipsis,
    Search,
}

impl Glyph {
    pub const ALL: &[Glyph] = &[
        Glyph::Close,
        Glyph::ChevronDown,
        Glyph::ChevronUp,
        Glyph::ChevronRight,
        Glyph::Check,
        Glyph::Plus,
        Glyph::Minus,
        Glyph::Spinner,
        Glyph::Grip,
        Glyph::Ellipsis,
        Glyph::Search,
    ];
}

/// An icon per [`Glyph`]. Provide one with a `ContextProvider<IconSet>`;
/// without one, plinth uses [`IconSet::default`]. Each should draw in
/// `currentColor` and fill its box (plinth sizes the box, not the icon).
#[derive(Clone, PartialEq)]
pub struct IconSet {
    pub close: Html,
    pub chevron_down: Html,
    pub chevron_up: Html,
    pub chevron_right: Html,
    pub check: Html,
    pub plus: Html,
    pub minus: Html,
    pub spinner: Html,
    pub grip: Html,
    pub ellipsis: Html,
    pub search: Html,
}

fn svg(paths: Html) -> Html {
    html! {
        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"
            stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false">
            { paths }
        </svg>
    }
}

impl Default for IconSet {
    /// The built-in set: inline svg on a 16 unit grid.
    fn default() -> Self {
        Self {
            close: svg(html! { <path d="M4 4l8 8M12 4l-8 8" /> }),
            chevron_down: svg(html! { <path d="M4 6l4 4 4-4" /> }),
            chevron_up: svg(html! { <path d="M4 10l4-4 4 4" /> }),
            chevron_right: svg(html! { <path d="M6 4l4 4-4 4" /> }),
            check: svg(html! { <path d="M3.5 8.5l3 3 6-7" /> }),
            plus: svg(html! { <path d="M8 3v10M3 8h10" /> }),
            minus: svg(html! { <path d="M3 8h10" /> }),
            spinner: svg(html! { <path d="M8 2a6 6 0 1 0 6 6" /> }),
            grip: {
                let cells: Html = [5u8, 11]
                    .iter()
                    .flat_map(|x| [3u8, 8, 13].map(move |y| (*x, y)))
                    .map(|(x, y)| html! { <circle cx={x.to_string()} cy={y.to_string()} r="1" fill="currentColor" stroke="none" /> })
                    .collect();
                svg(cells)
            },
            ellipsis: svg([3u8, 8, 13]
                .iter()
                .map(|x| html! { <circle cx={x.to_string()} cy="8" r="1" fill="currentColor" stroke="none" /> })
                .collect()),
            search: svg(html! { <><circle cx="7" cy="7" r="4.5" /><path d="M10.5 10.5L14 14" /></> }),
        }
    }
}

impl IconSet {
    /// The icon for `glyph`.
    pub fn get(&self, glyph: Glyph) -> Html {
        match glyph {
            Glyph::Close => &self.close,
            Glyph::ChevronDown => &self.chevron_down,
            Glyph::ChevronUp => &self.chevron_up,
            Glyph::ChevronRight => &self.chevron_right,
            Glyph::Check => &self.check,
            Glyph::Plus => &self.plus,
            Glyph::Minus => &self.minus,
            Glyph::Spinner => &self.spinner,
            Glyph::Grip => &self.grip,
            Glyph::Ellipsis => &self.ellipsis,
            Glyph::Search => &self.search,
        }
        .clone()
    }
}

/// The app's [`IconSet`], or the built-in one without a provider.
#[hook]
pub fn use_icons() -> IconSet {
    use_context::<IconSet>().unwrap_or_default()
}

/// The icon for `glyph`, from the app's set or the built-in one.
#[hook]
pub fn use_icon(glyph: Glyph) -> Html {
    use_icons().get(glyph)
}

#[derive(Properties, PartialEq)]
pub struct IconProps {
    pub glyph: Glyph,
    #[prop_or_default]
    pub class: Classes,
}

/// The box's classes: `glyph` always, `spin` for the spinner.
pub fn icon(classes: Classes, glyph: Glyph) -> Classes {
    classes!(
        classes,
        class::GLYPH,
        (glyph == Glyph::Spinner).then_some(class::SPIN)
    )
}

/// A glyph in its fixed `--glyph-size` box: `1em` unless set, so it
/// follows the text, and the same box whatever icon the set holds.
#[function_component]
pub fn Icon(props: &IconProps) -> Html {
    let icon_html = use_icon(props.glyph);
    html! { <span class={icon(props.class.clone(), props.glyph)} aria-hidden="true">{ icon_html }</span> }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn a_glyph_is_boxed_and_only_the_spinner_spins() {
        assert_eq!(names(icon(Classes::new(), Glyph::Close)), [class::GLYPH]);
        assert_eq!(
            names(icon(classes!(class::CARD), Glyph::Spinner)),
            [class::CARD, class::GLYPH, class::SPIN]
        );
    }

    #[test]
    fn the_built_in_set_has_every_glyph() {
        let set = IconSet::default();
        assert!(Glyph::ALL.iter().all(|g| set.get(*g) != html! {}));
    }
}
