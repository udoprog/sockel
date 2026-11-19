//! Skeleton (`style/_skeleton.scss`).

use crate::testid::with_testid;
use crate::{Ratio, class};
use yew::prelude::*;

/// The box a [`Skeleton`] stands in for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SkeletonShape {
    /// One line of the text around it (`1lh`).
    Line,
    /// One control high (`--control-height`), or `--skeleton-height`.
    #[default]
    Block,
    /// One `RowListItem` high.
    Row,
    /// A picture of the ratio an [`Image`](crate::Image) has.
    Image(Ratio),
}

#[derive(Properties, PartialEq)]
pub struct SkeletonProps {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub shape: SkeletonShape,
    /// How many, in a column; the last of several lines is shorter.
    #[prop_or(1)]
    pub count: usize,
    /// Its width (a CSS length, such as the text it stands for).
    #[prop_or_default]
    pub width: Option<AttrValue>,
    #[prop_or_default]
    pub testid: Option<AttrValue>,
}

/// A skeleton's classes: `skeleton`, its shape, and `short` for a line that
/// ends a paragraph.
pub fn skeleton(shape: SkeletonShape, short: bool) -> Classes {
    let shape = match shape {
        SkeletonShape::Line => Some(class::LINE),
        SkeletonShape::Block => None,
        SkeletonShape::Row => Some(class::ROW),
        SkeletonShape::Image(ratio) => Some(ratio.class()),
    };
    classes!(class::SKELETON, shape, short.then_some(class::SHORT))
}

/// A placeholder of the box its content will have, so nothing moves when the
/// content arrives. Several (`count`) stand in a column; a group of rows
/// reserves the border a `RowList` does. It is hidden from a screen reader:
/// say what is loading with a status ([`Loadable`](crate::Loadable)).
#[function_component]
pub fn Skeleton(props: &SkeletonProps) -> Html {
    let ratio = match props.shape {
        SkeletonShape::Image(Ratio::Explicit(width, height)) => {
            Some(format!("--ratio: {width} / {height}"))
        }
        _ => None,
    };
    let width = props.width.as_ref().map(|width| format!("width: {width}"));
    let style = match (ratio, width) {
        (Some(ratio), Some(width)) => Some(format!("{ratio}; {width}")),
        (ratio, width) => ratio.or(width),
    };
    let one = |short: bool, extra: Classes, testid: Option<AttrValue>| {
        with_testid(
            html! {
                <span class={classes!(skeleton(props.shape, short), extra)} style={style.clone()}
                    aria-hidden="true" />
            },
            testid,
        )
    };

    if props.count == 1 {
        return one(false, props.class.clone(), props.testid.clone());
    }

    let last = props.count.saturating_sub(1);
    with_testid(
        html! {
            <span class={classes!(class::SKELETON_GROUP, props.class.clone())} aria-hidden="true">
                { for (0..props.count).map(|n| {
                    one(props.shape == SkeletonShape::Line && n == last, Classes::new(), None)
                }) }
            </span>
        },
        props.testid.clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(classes: Classes) -> Vec<String> {
        classes.into_iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn a_shape_is_a_modifier() {
        let of = |shape| names(skeleton(shape, false));
        assert_eq!(of(SkeletonShape::Block), [class::SKELETON]);
        assert_eq!(of(SkeletonShape::Line), [class::SKELETON, class::LINE]);
        assert_eq!(of(SkeletonShape::Row), [class::SKELETON, class::ROW]);
        assert_eq!(
            of(SkeletonShape::Image(Ratio::Poster)),
            [class::SKELETON, class::POSTER]
        );
        assert_eq!(
            of(SkeletonShape::Image(Ratio::Explicit(4, 3))),
            [class::SKELETON, class::RATIO]
        );
    }

    #[test]
    fn short_is_a_modifier() {
        assert_eq!(
            names(skeleton(SkeletonShape::Line, true)),
            [class::SKELETON, class::LINE, class::SHORT]
        );
    }
}
