//! Image and Avatar (`style/_media.scss`): pictures in a box that is fixed
//! before they load.

use crate::class;
use crate::icons::{Glyph, Icon};
use yew::prelude::*;

/// The box an [`Image`] keeps, whatever it shows. There is no default: an
/// image without a box shifts the page as it loads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ratio {
    /// 2 / 3.
    Poster,
    /// 16 / 9.
    Screenshot,
    /// 21 / 4.
    Banner,
    /// 1 / 1.
    Square,
    /// The picture's own width and height, also set as its attributes.
    Explicit(u32, u32),
}

impl Ratio {
    fn class(self) -> &'static str {
        match self {
            Ratio::Poster => class::POSTER,
            Ratio::Screenshot => class::SCREENSHOT,
            Ratio::Banner => class::BANNER,
            Ratio::Square => class::SQUARE,
            Ratio::Explicit(..) => class::RATIO,
        }
    }
}

/// Where a picture is in loading.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// No `src`.
    Empty,
    Loading,
    Loaded,
    Error,
}

impl Status {
    /// The status of `src`, given the last `(src, loaded)` the browser reported:
    /// a report about another `src` is stale, so a new `src` starts loading.
    fn of(src: Option<&str>, reported: Option<(&str, bool)>) -> Self {
        match (src, reported) {
            (None, _) => Status::Empty,
            (Some(src), Some((last, loaded))) if src == last => {
                if loaded {
                    Status::Loaded
                } else {
                    Status::Error
                }
            }
            (Some(_), _) => Status::Loading,
        }
    }

    fn class(self) -> &'static str {
        match self {
            Status::Empty => class::EMPTY,
            Status::Loading => class::LOADING,
            Status::Loaded => class::LOADED,
            Status::Error => class::ERROR,
        }
    }
}

/// The status of `src`, and the callbacks for its `<img>`'s `load` and `error`.
#[hook]
fn use_status(src: &Option<AttrValue>) -> (Status, Callback<Event>, Callback<Event>) {
    let reported = use_state(|| None::<(AttrValue, bool)>);
    let status = Status::of(
        src.as_deref(),
        reported.as_ref().map(|(src, ok)| (src.as_str(), *ok)),
    );
    let report = |ok| {
        let reported = reported.clone();
        let src = src.clone();
        Callback::from(move |_| reported.set(src.clone().map(|src| (src, ok))))
    };
    (status, report(true), report(false))
}

/// An [`Image`]'s classes: `image`, its ratio and its status.
pub fn image(classes: Classes, ratio: Ratio, status: Status) -> Classes {
    classes!(classes, class::IMAGE, ratio.class(), status.class())
}

/// The picture's classes: `picture`, and `loaded` once it has.
fn picture(status: Status) -> Classes {
    classes!(
        class::PICTURE,
        (status == Status::Loaded).then_some(class::LOADED)
    )
}

#[derive(Properties, PartialEq)]
pub struct ImageProps {
    /// The picture's box, required.
    pub ratio: Ratio,
    /// The picture's address; without one the box shows its empty state.
    #[prop_or_default]
    pub src: Option<AttrValue>,
    #[prop_or_default]
    pub alt: AttrValue,
    /// The picture loads once it nears the viewport (`loading="lazy"`).
    #[prop_or(true)]
    pub lazy: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub onload: Callback<Event>,
    /// Called when the picture could not be loaded, which the box shows.
    #[prop_or_default]
    pub onerror: Callback<Event>,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// A picture in a box of a fixed ratio, filling the width it is given. The
/// box draws a placeholder while the picture loads, when it fails and when
/// there is none, so it never changes size between them.
#[function_component]
pub fn Image(props: &ImageProps) -> Html {
    let (status, loaded, failed) = use_status(&props.src);
    let glyph = match status {
        Status::Loading => Some(Glyph::Spinner),
        Status::Error => Some(Glyph::Close),
        Status::Empty => Some(Glyph::Minus),
        Status::Loaded => None,
    };
    let (width, height) = match props.ratio {
        Ratio::Explicit(width, height) => (Some(width.to_string()), Some(height.to_string())),
        _ => (None, None),
    };
    let style = match props.ratio {
        Ratio::Explicit(width, height) => Some(format!("--ratio: {width} / {height}")),
        _ => None,
    };
    let onload = props.onload.clone();
    let onerror = props.onerror.clone();

    html! {
        <span class={image(props.class.clone(), props.ratio, status)} {style}
            data-testid={props.testid.clone()}>
            if let Some(src) = &props.src {
                <img class={picture(status)} src={src.clone()} alt={props.alt.clone()}
                    {width} {height} loading={props.lazy.then_some("lazy")}
                    onload={Callback::from(move |e: Event| { loaded.emit(e.clone()); onload.emit(e); })}
                    onerror={Callback::from(move |e: Event| { failed.emit(e.clone()); onerror.emit(e); })} />
            }
            if let Some(glyph) = glyph {
                <span class={class::PLACEHOLDER}>
                    <Icon {glyph} />
                </span>
            }
        </span>
    }
}

/// The fixed sizes of an [`Avatar`], from the `--avatar-*` tokens.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AvatarSize {
    /// `--avatar-sm`: beside a line of text.
    Small,
    /// `--avatar-md`: the row of a list.
    #[default]
    Medium,
    /// `--avatar-lg`.
    Large,
    /// `--avatar-xl`: a profile's head.
    Huge,
}

impl AvatarSize {
    fn class(self) -> Option<&'static str> {
        match self {
            AvatarSize::Small => Some(class::AVATAR_SMALL),
            AvatarSize::Medium => None,
            AvatarSize::Large => Some(class::AVATAR_LARGE),
            AvatarSize::Huge => Some(class::AVATAR_HUGE),
        }
    }
}

/// An [`Avatar`]'s classes: `avatar` and its size.
pub fn avatar(classes: Classes, size: AvatarSize) -> Classes {
    classes!(classes, class::AVATAR, size.class())
}

/// Up to two capital letters of `name`: the first of its first and of its
/// last word.
pub fn initials(name: &str) -> String {
    let mut words = name.split_whitespace();
    let first = words.next().and_then(|w| w.chars().next());
    let last = words.next_back().and_then(|w| w.chars().next());
    first
        .into_iter()
        .chain(last)
        .flat_map(char::to_uppercase)
        .collect()
}

#[derive(Properties, PartialEq)]
pub struct AvatarProps {
    /// Whose it is: its initials show until the picture has loaded, and
    /// when there is none or it fails.
    pub name: AttrValue,
    #[prop_or_default]
    pub src: Option<AttrValue>,
    #[prop_or_default]
    pub size: AvatarSize,
    #[prop_or(true)]
    pub lazy: bool,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// A round portrait of a fixed size, its initials behind the picture.
#[function_component]
pub fn Avatar(props: &AvatarProps) -> Html {
    let (status, loaded, failed) = use_status(&props.src);

    html! {
        <span class={avatar(props.class.clone(), props.size)} role="img"
            aria-label={props.name.clone()} data-testid={props.testid.clone()}>
            <span class={class::AVATAR_INITIALS} aria-hidden="true">{ initials(&props.name) }</span>
            if let Some(src) = &props.src {
                <img class={picture(status)} src={src.clone()} alt="" loading={props.lazy.then_some("lazy")}
                    onload={loaded} onerror={failed} />
            }
        </span>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn an_image_has_its_ratio_and_status_classes() {
        let of = |ratio, status| names(image(Classes::new(), ratio, status));
        assert_eq!(
            of(Ratio::Poster, Status::Loading),
            [class::IMAGE, class::POSTER, class::LOADING]
        );
        assert_eq!(
            of(Ratio::Screenshot, Status::Error),
            [class::IMAGE, class::SCREENSHOT, class::ERROR]
        );
        assert_eq!(
            of(Ratio::Banner, Status::Empty),
            [class::IMAGE, class::BANNER, class::EMPTY]
        );
        assert_eq!(
            of(Ratio::Square, Status::Loaded),
            [class::IMAGE, class::SQUARE, class::LOADED]
        );
        assert_eq!(
            of(Ratio::Explicit(4, 3), Status::Loaded),
            [class::IMAGE, class::RATIO, class::LOADED]
        );
    }

    #[test]
    fn the_picture_shows_only_once_loaded() {
        assert_eq!(
            names(picture(Status::Loaded)),
            [class::PICTURE, class::LOADED]
        );
        for status in [Status::Loading, Status::Error] {
            assert_eq!(names(picture(status)), [class::PICTURE]);
        }
    }

    #[test]
    fn an_avatar_has_a_class_per_size() {
        let of = |size| names(avatar(Classes::new(), size));
        assert_eq!(of(AvatarSize::Small), [class::AVATAR, class::AVATAR_SMALL]);
        assert_eq!(of(AvatarSize::Medium), [class::AVATAR]);
        assert_eq!(of(AvatarSize::Large), [class::AVATAR, class::AVATAR_LARGE]);
        assert_eq!(of(AvatarSize::Huge), [class::AVATAR, class::AVATAR_HUGE]);
    }

    #[test]
    fn a_failed_load_is_an_error_until_the_src_changes() {
        assert_eq!(Status::of(None, None), Status::Empty);
        assert_eq!(Status::of(None, Some(("a", true))), Status::Empty);
        assert_eq!(Status::of(Some("a"), None), Status::Loading);
        assert_eq!(Status::of(Some("a"), Some(("a", true))), Status::Loaded);
        assert_eq!(Status::of(Some("a"), Some(("a", false))), Status::Error);
        assert_eq!(Status::of(Some("b"), Some(("a", false))), Status::Loading);
    }

    #[test]
    fn initials_are_the_first_and_last_words() {
        assert_eq!(initials("ada lovelace"), "AL");
        assert_eq!(initials("Ada"), "A");
        assert_eq!(initials("Ada Augusta King"), "AK");
        assert_eq!(initials("  "), "");
        assert_eq!(initials(""), "");
    }
}
