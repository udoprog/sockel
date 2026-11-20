# sockel-build

[<img alt="github" src="https://img.shields.io/badge/github-udoprog/sockel-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/sockel)
[<img alt="crates.io" src="https://img.shields.io/crates/v/sockel-build.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/sockel-build)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-sockel--build-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/sockel-build)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/udoprog/sockel/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/udoprog/sockel/actions?query=branch%3Amain)

Builds an app's one stylesheet and its class constants from a `build.rs`:
sockel's, and a project's own Sass beside it ([`Config::local`](https://docs.rs/sockel-build/latest/sockel_build/struct.Config.html#method.local)).

sockel's Sass sources are read from the sockel package a build depends on
([`STYLE_VAR`](https://docs.rs/sockel-build/latest/sockel_build/constant.STYLE_VAR.html)); local Sass loads them as `@use "sockel" as p;`.
[`Config::compile`](https://docs.rs/sockel-build/latest/sockel_build/struct.Config.html#method.compile) writes `style.css` and `classes.rs` (a `pub const` per
class) to `OUT_DIR`, so a misspelt class does not compile.

It is the build half of [sockel](https://docs.rs/sockel), Yew components
that hold still; see them in the
[showcase](https://udoprog.github.io/sockel/).

<br>

## `build.rs`

Add sockel-build as a build dependency, then in the `main` of the package's
`build.rs`:

```rust
sockel_build::Config::new()
    .local("style/")
    .prefix("app-")
    .compile();
```

The app's Sass lives in `style/`, with an `_index.scss` (or `index.scss`)
that `@use`s its partials and loads sockel's tokens and mixins as
`"sockel"`:

```scss
@use "sockel" as p;

.app-sidebar {
    padding: var(--space-4);

    @include p.below("phone") {
        padding: var(--space-2);
    }
}
```

The build reruns when `build.rs`, anything under `style/` or sockel's Sass
changes. It emits no second copy of sockel's CSS: `style.css` holds
sockel's and the app's together, so the page links no other stylesheet.

Every class the app's Sass defines that is not sockel's must start with the
prefix (`app-`); one that does not is a build error. Classes a
dependency's markup writes are allowed with [`Config::foreign`](https://docs.rs/sockel-build/latest/sockel_build/struct.Config.html#method.foreign) (a trailing
`*` is a prefix), and get no constant. Tokens are configured on the build
with [`Config::with`](https://docs.rs/sockel-build/latest/sockel_build/struct.Config.html#method.with), not `@use "sockel" with (...)`. The prefix is in
every selector naming a local class, so a short one (`ch-` rather than
`characterize-`) keeps the stylesheet smaller; the constants do not change.

<br>

## The class constants and the stylesheet

`classes.rs` holds the app's classes, named without the prefix. For the
Sass above it is:

```rust
/// `.app-sidebar`
pub const SIDEBAR: &str = "app-sidebar";
```

Include it in a module, and the stylesheet as a string:

```rust
mod class {
    include!(concat!(env!("OUT_DIR"), "/classes.rs"));
}

const STYLESHEET: &str = include_str!(concat!(env!("OUT_DIR"), "/style.css"));
```

Pass the constants to a sockel component's `class` prop, so a class the
Sass does not define does not compile, and mount the stylesheet once with
`<sockel::Styles css={STYLESHEET} />`.

<br>

## A size budget

[`Report::css_bytes`](https://docs.rs/sockel-build/latest/sockel_build/struct.Report.html#structfield.css_bytes) is the stylesheet's size in plain bytes. [`Config::css_budget`](https://docs.rs/sockel-build/latest/sockel_build/struct.Config.html#method.css_budget) fails the build with [`Error::OverBudget`](https://docs.rs/sockel-build/latest/sockel_build/enum.Error.html#variant.OverBudget) when the sheet is larger, so growth is caught where it happens. There is no limit unless one is set.

<br>

## How it finds sockel

sockel's own `build.rs` passes two values to the `build.rs` of every
package depending on it directly (Cargo's `links` metadata):
[`FEATURES_VAR`](https://docs.rs/sockel-build/latest/sockel_build/constant.FEATURES_VAR.html), its enabled features, and [`STYLE_VAR`](https://docs.rs/sockel-build/latest/sockel_build/constant.STYLE_VAR.html), the `style/`
directory of its package. [`Config`](https://docs.rs/sockel-build/latest/sockel_build/struct.Config.html) builds those features from those
sources, so the stylesheet matches the sockel the app links. A package that
reaches sockel only through another crate names them with
[`Config::features`](https://docs.rs/sockel-build/latest/sockel_build/struct.Config.html#method.features) and [`Config::style`](https://docs.rs/sockel-build/latest/sockel_build/struct.Config.html#method.style).

<br>

## A guard against dead classes

A class the Sass defines and no Rust writes is dead weight. [`Usage`](https://docs.rs/sockel-build/latest/sockel_build/struct.Usage.html) finds
them in a test:

```rust
use sockel_build::{Config, Usage};

#[test]
fn every_styled_class_is_written() {
    Usage::new()
        .source(concat!(env!("CARGO_MANIFEST_DIR"), "/src"))
        .assert_clean(
            &Config::new()
                .style(env!("SOCKEL_STYLE"))
                .local("style/")
                .prefix("app-"),
        );
}
```

`SOCKEL_STYLE` ([`TEST_STYLE_VAR`](https://docs.rs/sockel-build/latest/sockel_build/constant.TEST_STYLE_VAR.html)) is sockel's style directory, which
`compile()` hands the package's crates. A class counts as written when a
source names its constant (`class::SIDEBAR`) or the class in a string
literal; comments and `#[cfg(test)]` items do not count. The test fails on
a dead class and on an allowed pattern that matches no class.

It also fails on a class written without its prefix: a literal word that,
prefixed, is a styled class (`"button"` beside `.app-button`), in a class
position. That is a `class` attribute, field or binding, `classes!`, or a
parameter of the sources' own `fn` named for a class, reached through
`Some(..)`, tuples, slices and `match` or `if` arms. A word the sources
write on purpose is allowed with [`Usage::unprefixed`](https://docs.rs/sockel-build/latest/sockel_build/struct.Usage.html#method.unprefixed); one a
[`Usage::dynamic`](https://docs.rs/sockel-build/latest/sockel_build/struct.Usage.html#method.dynamic) pattern covers needs nothing.

It fails as well on a sockel class the build leaves out (`$omit`) that a
source writes, unless the local styles define it. Sockel's own components
that emit an omitted class are not seen: check those against the parts table
in [sockel's docs](https://docs.rs/sockel/latest/sockel/#leaving-parts-out).

[`Usage::one_file_per_component`](https://docs.rs/sockel-build/latest/sockel_build/struct.Usage.html#method.one_file_per_component) also fails on a class with a top-level
rule in more than one local Sass file, so a component's rules stay in its
own file.

<br>

## One Config for the build and the guard

So the guard checks the stylesheet the app builds, define the [`Config`](https://docs.rs/sockel-build/latest/sockel_build/struct.Config.html)
once, in a file beside `build.rs` (`style.rs`):

```rust
fn style() -> sockel_build::Config {
    sockel_build::Config::new()
        .local("style/")
        .prefix("app-")
        .foreign(["auth-*"])
        .with("radius-lg", "12px")
}
```

`build.rs` includes and compiles it:

```rust
include!("style.rs");

fn main() {
    style().compile();
}
```

and the guard includes it as well, adding sockel's style directory, which
only `build.rs` is told:

```rust
use sockel_build::Usage;

include!("../style.rs");

#[test]
fn every_styled_class_is_written() {
    Usage::new()
        .source(concat!(env!("CARGO_MANIFEST_DIR"), "/src"))
        .assert_clean(&style().style(env!("SOCKEL_STYLE")));
}
```

A relative path in it (`style/`) is the package root's in both, where Cargo
runs build scripts and tests. [`FEATURES_VAR`](https://docs.rs/sockel-build/latest/sockel_build/constant.FEATURES_VAR.html) is set only in `build.rs`,
so the guard builds all of sockel's features unless the Config names them
with [`Config::features`](https://docs.rs/sockel-build/latest/sockel_build/struct.Config.html#method.features).
