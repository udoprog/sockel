# sockel-build

[<img alt="github" src="https://img.shields.io/badge/github-udoprog/sockel-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/sockel)
[<img alt="crates.io" src="https://img.shields.io/crates/v/sockel-build.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/sockel-build)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-sockel--build-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/sockel-build)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/udoprog/sockel/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/udoprog/sockel/actions?query=branch%3Amain)

The build half of [sockel](https://docs.rs/sockel): compiles sockel's CSS and
an app's own Sass into one stylesheet, with a Rust constant for every class.

Because markup names classes through constants, a misspelt or deleted class
is a compile error, and a test finds classes that are styled but never
written. See sockel's components on the
[documentation site](https://udoprog.github.io/sockel/).

<br>

## Building the stylesheet

Add sockel-build as a build dependency, and in `build.rs`:

```rust
sockel_build::Config::new()
    .local("style/")
    .prefix("app-")
    .compile();
```

The app's Sass lives in `style/`, with an `_index.scss` that loads sockel's
tokens and mixins as `"sockel"`:

```scss
@use "sockel" as p;

.app-sidebar {
    padding: var(--space-4);

    @include p.below("phone") {
        padding: var(--space-2);
    }
}
```

The build writes `style.css` (sockel's CSS, then the app's) and `classes.rs`
to `OUT_DIR`, and reruns when the Sass changes. Every class the app defines
must start with the prefix, so it never collides with sockel's.

`classes.rs` holds a constant per class, named without the prefix:

```rust
pub const SIDEBAR: &str = "app-sidebar";
```

The app includes both and passes the constants to sockel's `class` props:

```rust
mod class {
    include!(concat!(env!("OUT_DIR"), "/classes.rs"));
}

const STYLESHEET: &str = include_str!(concat!(env!("OUT_DIR"), "/style.css"));
```

<br>

## Finding dead classes

A class the Sass defines and no Rust writes is dead weight.
[`Usage`](https://docs.rs/sockel-build/latest/sockel_build/struct.Usage.html)
finds them in a test. So that the test checks the stylesheet the app builds,
describe the build once, in a `style.rs` beside `build.rs`:

```rust
fn style() -> sockel_build::Config {
    sockel_build::Config::new()
        .local("style/")
        .prefix("app-")
}
```

`build.rs` runs `include!("style.rs");` and `style().compile()`, and the test
includes it too:

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

A class counts as written when the sources name its constant, or the class in
a string literal. The test also catches a class written without its prefix,
and a sockel class the build leaves out but the sources still use.

<br>

## Keeping the stylesheet small

The stylesheet only holds the CSS of the sockel features the app enables. On
top of that:

- [`Config::prune`](https://docs.rs/sockel-build/latest/sockel_build/struct.Config.html#method.prune)
  drops sockel's rules for components the app's sources never reach.
- [`Config::css_budget`](https://docs.rs/sockel-build/latest/sockel_build/struct.Config.html#method.css_budget)
  fails the build when the stylesheet grows past a size.

<br>

## How it finds sockel

sockel's own build script tells every package that depends on it directly
which features it was built with and where its Sass is (Cargo's `links`
metadata). That is how the stylesheet always matches the sockel the app links.
A package that only reaches sockel through another crate names them itself,
with
[`Config::features`](https://docs.rs/sockel-build/latest/sockel_build/struct.Config.html#method.features)
and
[`Config::style`](https://docs.rs/sockel-build/latest/sockel_build/struct.Config.html#method.style).

<br>

## Configuring sockel

[`Config::with`](https://docs.rs/sockel-build/latest/sockel_build/struct.Config.html#method.with)
sets one of sockel's Sass variables, such as `.with("radius-lg", "12px")`.
Every variable, its default and what it sets is on the
[configuration reference](https://udoprog.github.io/sockel/#reference).
