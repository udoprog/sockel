//! The icons of sockel's own chrome (`style/_icons.scss`): an [`IconSet`]
//! the app provides once at its root, and a built-in set without one.

use crate::class;
use std::collections::HashMap;
use std::rc::Rc;

use yew::prelude::*;

/// One of sockel's glyphs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
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
    Eye,
    EyeOff,
    Copy,
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
        Glyph::Eye,
        Glyph::EyeOff,
        Glyph::Copy,
    ];
}

/// The icons sockel draws with and the app's own, by [`Glyph`] or by name.
/// Provide one with a `ContextProvider<IconSet>`; without one, sockel uses
/// [`IconSet::default`]. Each icon should draw in `currentColor` and fill its
/// box (sockel sizes the box, not the icon). Cloning shares the icons.
#[derive(Clone, Default)]
pub struct IconSet {
    inner: Rc<Icons>,
}

#[derive(Clone, Default)]
struct Icons {
    glyphs: HashMap<Glyph, Html>,
    named: HashMap<String, Html>,
}

impl PartialEq for IconSet {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
            || (self.inner.glyphs == other.inner.glyphs && self.inner.named == other.inner.named)
    }
}

fn svg(paths: Html) -> Html {
    html! {
        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"
            stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false">
            { paths }
        </svg>
    }
}

fn dots(points: impl Iterator<Item = (u8, u8)>) -> Html {
    svg(points
        .map(|(x, y)| html! { <circle cx={x.to_string()} cy={y.to_string()} r="1" fill="currentColor" stroke="none" /> })
        .collect())
}

/// The built-in icon: inline svg on a 16 unit grid, built when asked for.
fn built_in(glyph: Glyph) -> Html {
    match glyph {
        Glyph::Close => svg(html! { <path d="M4 4l8 8M12 4l-8 8" /> }),
        Glyph::ChevronDown => svg(html! { <path d="M4 6l4 4 4-4" /> }),
        Glyph::ChevronUp => svg(html! { <path d="M4 10l4-4 4 4" /> }),
        Glyph::ChevronRight => svg(html! { <path d="M6 4l4 4-4 4" /> }),
        Glyph::Check => svg(html! { <path d="M3.5 8.5l3 3 6-7" /> }),
        Glyph::Plus => svg(html! { <path d="M8 3v10M3 8h10" /> }),
        Glyph::Minus => svg(html! { <path d="M3 8h10" /> }),
        Glyph::Spinner => svg(html! { <path d="M8 2a6 6 0 1 0 6 6" /> }),
        Glyph::Grip => dots(
            [5u8, 11]
                .into_iter()
                .flat_map(|x| [3u8, 8, 13].map(move |y| (x, y))),
        ),
        Glyph::Ellipsis => dots([3u8, 8, 13].into_iter().map(|x| (x, 8))),
        Glyph::Search => {
            svg(html! { <><circle cx="7" cy="7" r="4.5" /><path d="M10.5 10.5L14 14" /></> })
        }
        Glyph::Eye => svg(
            html! { <><path d="M1.5 8S4 3.5 8 3.5 14.5 8 14.5 8 12 12.5 8 12.5 1.5 8 1.5 8z" /><circle cx="8" cy="8" r="2" /></> },
        ),
        Glyph::EyeOff => svg(
            html! { <><path d="M1.5 8S4 3.5 8 3.5 14.5 8 14.5 8 12 12.5 8 12.5 1.5 8 1.5 8z" /><circle cx="8" cy="8" r="2" /><path d="M3 13L13 3" /></> },
        ),
        Glyph::Copy => svg(
            html! { <><rect x="5.5" y="5.5" width="8" height="8" rx="1.5" /><path d="M10.5 5.5v-2a1 1 0 0 0-1-1h-6a1 1 0 0 0-1 1v6a1 1 0 0 0 1 1h2" /></> },
        ),
    }
}

impl IconSet {
    /// Replaces one of sockel's glyphs; the others stay built in.
    pub fn with(mut self, glyph: Glyph, icon: Html) -> Self {
        Rc::make_mut(&mut self.inner).glyphs.insert(glyph, icon);
        self
    }

    /// Adds an app icon under `name`, shown by `<Icon name=.../>`.
    pub fn with_named(mut self, name: impl Into<String>, icon: Html) -> Self {
        Rc::make_mut(&mut self.inner)
            .named
            .insert(name.into(), icon);
        self
    }

    /// The icon for `glyph`.
    pub fn get(&self, glyph: Glyph) -> Html {
        self.inner
            .glyphs
            .get(&glyph)
            .cloned()
            .unwrap_or_else(|| built_in(glyph))
    }

    /// The app icon `name`, if the set has it.
    pub fn get_named(&self, name: &str) -> Option<Html> {
        self.inner.named.get(name).cloned()
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

/// The app icon `name`; empty, with a debug warning, when the set lacks it.
#[hook]
pub fn use_icon_named(name: &str) -> Html {
    let icons = use_icons();
    icons.get_named(name).unwrap_or_else(|| missing(name))
}

fn missing(name: &str) -> Html {
    #[cfg(all(debug_assertions, target_arch = "wasm32"))]
    web_sys::console::warn_1(&format!("sockel: no icon named {name:?} in the IconSet").into());
    #[cfg(not(all(debug_assertions, target_arch = "wasm32")))]
    let _ = name;
    html! {}
}

#[derive(Properties, PartialEq)]
pub struct IconProps {
    /// One of sockel's glyphs; `name` is used when it is not set.
    #[prop_or_default]
    pub glyph: Option<Glyph>,
    /// An app icon added with [`IconSet::with_named`].
    #[prop_or_default]
    pub name: Option<AttrValue>,
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

/// A glyph or named app icon in its fixed `--glyph-size` box: `1em` unless
/// set, so it follows the text, and the same box whatever the set holds,
/// empty for an icon it lacks.
#[function_component]
pub fn Icon(props: &IconProps) -> Html {
    let icons = use_icons();
    let (classes, icon_html) = match (props.glyph, &props.name) {
        (Some(glyph), _) => (icon(props.class.clone(), glyph), icons.get(glyph)),
        (None, Some(name)) => (
            classes!(props.class.clone(), class::GLYPH),
            icons.get_named(name).unwrap_or_else(|| missing(name)),
        ),
        (None, None) => (classes!(props.class.clone(), class::GLYPH), html! {}),
    };
    html! { <span class={classes} aria-hidden="true">{ icon_html }</span> }
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

    #[test]
    fn the_builder_overrides_one_glyph_and_keeps_the_rest() {
        let custom = html! { <b>{ "x" }</b> };
        let set = IconSet::default().with(Glyph::Close, custom.clone());
        assert_eq!(set.get(Glyph::Close), custom);
        assert_eq!(set.get(Glyph::Check), IconSet::default().get(Glyph::Check));
    }

    #[test]
    fn a_named_icon_round_trips_and_a_missing_one_is_none() {
        let custom = html! { <i>{ "bin" }</i> };
        let set = IconSet::default().with_named("trash", custom.clone());
        assert_eq!(set.get_named("trash"), Some(custom));
        assert_eq!(set.get_named("nope"), None);
    }

    #[test]
    fn a_clone_shares_the_icons_until_it_is_changed() {
        let set = IconSet::default().with_named("a", html! { <i /> });
        let copy = set.clone();
        assert!(Rc::ptr_eq(&set.inner, &copy.inner));
        let changed = copy.with_named("b", html! { <i /> });
        assert!(!Rc::ptr_eq(&set.inner, &changed.inner));
        assert_eq!(set.get_named("b"), None);
    }
}
