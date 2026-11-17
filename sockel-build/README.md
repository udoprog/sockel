# sockel-build

Builds an app's one stylesheet and its class constants from a `build.rs`:
sockel's, and a project's own Sass beside it ([`Config::local`]).

sockel's Sass sources are read from the sockel package a build depends on
([`STYLE_VAR`]); local Sass loads them as `@use "sockel" as p;`.
[`Config::compile`] writes `style.css` and `classes.rs` (a `pub const` per
class) to `OUT_DIR`.

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
dependency's markup writes are allowed with [`Config::foreign`] (a trailing
`*` is a prefix), and get no constant. Tokens are configured on the build
with [`Config::with`], not `@use "sockel" with (...)`.

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

## How it finds sockel

sockel's own `build.rs` passes two values to the `build.rs` of every
package depending on it directly (Cargo's `links` metadata):
[`FEATURES_VAR`], its enabled features, and [`STYLE_VAR`], the `style/`
directory of its package. [`Config`] builds those features from those
sources, so the stylesheet matches the sockel the app links. A package that
reaches sockel only through another crate names them with
[`Config::features`] and [`Config::style`].

<br>

## A guard against dead classes

A class the Sass defines and no Rust writes is dead weight. [`Usage`] finds
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

`SOCKEL_STYLE` ([`TEST_STYLE_VAR`]) is sockel's style directory, which
`compile()` hands the package's crates. A class counts as written when a
source names its constant (`class::SIDEBAR`) or the class in a string
literal; comments and `#[cfg(test)]` items do not count. The test fails on
a dead class and on an allowed pattern that matches no class.
