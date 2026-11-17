# plinth

Opinionated layout primitives and components for [Yew] apps, built so
components keep their size and don't jitter as their content changes.

Each component comes with its Sass under `style/`, and its variants are
modifier classes set by props.

```toml
[dependencies]
plinth = { path = "../plinth" }
```

## Including the styles

plinth compiles `style/` with its defaults when the crate builds (with
[grass], through `plinth-build`), so an app needs no Sass step of its own.
An app without styles of its own mounts `<plinth::Styles/>` once at its
root:

```rust
html! {
    <>
        <plinth::Styles/>
        <App/>
    </>
}
```

It adds the stylesheet to `<head>` as `<style id="plinth-styles">`, once,
however many times it is mounted. The CSS is also `plinth::STYLESHEET`, for
an app that serves or inlines it itself. Its classes are constants in
`plinth::class` (`.page-head-title` is `plinth::class::PAGE_HEAD_TITLE`), so
a misspelt class does not compile.

## Adopting plinth with styles of your own

An app with its own Sass builds it with `plinth-build` in its `build.rs`,
beside plinth's, into **one stylesheet**: plinth's CSS first, unchanged,
then the app's. The app's classes become constants as plinth's are. Step by
step:

1. **Dependencies.** In the app's `Cargo.toml`:

   ```toml
   [dependencies]
   plinth = { path = "../plinth" }

   [build-dependencies]
   plinth-build = { path = "../plinth/plinth-build" }
   ```

2. **Styles.** Put the app's Sass in a directory, say `style/`, with an
   `_index.scss` (or `index.scss`) that `@use`s its partials. Load plinth's
   tokens and mixins as `plinth`:

   ```scss
   @use "plinth" as p;

   .army-sidebar {
       padding: var(--space-4);

       @include p.below("phone") {
           padding: var(--space-2);
       }
   }

   // plinth's classes are stable, so the app may refine them.
   .card.army-sidebar {
       --radius-lg: 12px;
   }
   ```

   `"plinth"` is served from the sources embedded in `plinth-build` (no path
   into plinth's checkout), so they always match the plinth-build version.
   It emits no second copy of plinth's CSS: plinth is compiled first in the
   same build. plinth's Sass cannot be configured from here (`@use "plinth"
   with (...)` fails); retune it through the custom properties (see
   [Theming](#theming)).

   Every class the app's Sass defines that is not plinth's must start with
   the app's prefix (`army-`); one that does not is a build error, so the
   app's classes never collide with plinth's.

3. **`build.rs`**, at the package root:

   ```rust
   fn main() {
       plinth_build::Config::new()
           .local("style/")
           .prefix("army-")
           .compile();
   }
   ```

   It writes `style.css` and `classes.rs` to `OUT_DIR`, and reruns when
   `build.rs` or anything under `style/` changes (partials and new files
   included).

4. **The class constants.** Somewhere in the crate (`src/main.rs` or
   `src/lib.rs`):

   ```rust
   mod class {
       include!(concat!(env!("OUT_DIR"), "/classes.rs"));
   }
   ```

   `classes.rs` holds only the app's classes, named without the prefix:
   `.army-sidebar` is `class::SIDEBAR`. plinth's stay `plinth::class::...`.
   Every plinth component takes a `class` prop, so pass the constants there:

   ```rust
   html! { <Stack class={class::SIDEBAR}>{ children }</Stack> }
   ```

   A class the Sass does not define, or a misspelt one, does not compile.

5. **The stylesheet.** Include it and mount it once at the root, with
   `Styles`'s `css`:

   ```rust
   const STYLESHEET: &str = include_str!(concat!(env!("OUT_DIR"), "/style.css"));

   html! {
       <>
           <plinth::Styles css={STYLESHEET} />
           <App/>
       </>
   }
   ```

6. **No Sass elsewhere.** Remove the app's own Sass step (trunk's
   `<link data-trunk rel="scss">` and `[tools] sass`); `index.html` links no
   stylesheet.

**One stylesheet per app.** Both `Styles` fill the same element,
`<style id="plinth-styles">` (`plinth::STYLE_ID`): with `css` it sets the
element's text to the app's stylesheet, which holds plinth's; without it, it
adds plinth's only when the element is missing. So whichever mounts first
(the app's, or a `<plinth::Styles/>` inside a component), the page ends up
with the app's stylesheet, once. Never link or inject plinth's CSS another
way beside it.

The showcase (`showcase/build.rs`, `showcase/style/`) is set up this way.

## Theming

The app supplies the colour tokens: `--bg`, `--surface`, `--text`, `--muted`,
`--border`, `--accent` and `--shadow-1`. plinth's own scales are custom
properties on `:root` too (`--text-*`, `--space-*`, `--radius-*`,
`--control-height`, `--page-*` and the rest, listed under Configuration), and
every rule reads them, so an app retunes the embedded stylesheet by
overriding them in its own CSS, loaded after it or with a more specific
selector:

```css
:root {
    --bg: #fafafa;
    --surface: #fff;
    --text: #1d1d1f;
    --muted: #6e6e73;
    --border: #d2d2d7;
    --accent: #0a66c2;
    --shadow-1: 0 1px 2px rgb(0 0 0 / 0.08);
    --radius-lg: 12px;
}

.dense {
    --control-height: 2rem;
}
```

Breakpoints and the component settings (`$card-padding`,
`$card-padding-phone`, `$toolbar-gap`) are Sass-only: an app built with
`plinth-build` keeps plinth's defaults, since plinth's CSS is included
unchanged.

## Configuration

This applies only to an app that compiles plinth's Sass itself, outside
`plinth-build`. plinth's values are defaults. Every token is a `!default` variable of
`style/_tokens.scss`, emitted as its custom property on `:root`, so a project
sets its own with `with (...)`, either through the index:

```scss
@use "../plinth/style" with ($text-md: 1.0625rem, $radius-lg: 12px);
@use "../plinth/style/tokens";   // tokens and mixins, as configured
```

or on the tokens module first, then the rules:

```scss
@use "../plinth/style/tokens" with ($text-md: 1.0625rem);
@use "../plinth/style";
```

Sass configures a module only on its first load, so `with` goes on whichever
of the two comes first. The rules keep reading the custom properties, so a
page can still override them at runtime.

| Variable | Default |
| --- | --- |
| `$text-3xs`, `$text-2xs`, `$text-xs`, `$text-sm` | `0.625rem`, `0.6875rem`, `0.75rem`, `0.8125rem` |
| `$text-md`, `$text-base`, `$text-heading`, `$text-lg` | `0.875rem`, `0.9375rem`, `1.125rem`, `1.25rem` |
| `$text-title`, `$text-xl`, `$text-2xl` | `1.4rem`, `1.5rem`, `1.75rem` |
| `$space-0`, `$space-px`, `$space-1`, `$space-2`, `$space-3` | `0.125rem`, `1px`, `0.25rem`, `0.5rem`, `0.75rem` |
| `$space-4`, `$space-5`, `$space-6`, `$space-7`, `$space-8`, `$space-9` | `1rem`, `1.25rem`, `1.5rem`, `2rem`, `2.5rem`, `3rem` |
| `$radius-xs`, `$radius-sm`, `$radius-md`, `$radius-lg`, `$radius-pill` | `4px`, `8px`, `10px`, `16px`, `999px` |
| `$nest-inset` | `var(--space-1)` |
| `$control-height`, `$control-height-small` | `2.25rem`, `1.75rem` |
| `$mark-height`, `$touch-height` | `1.5rem`, `2.75rem` |
| `$filter-width` | `10.5rem` |
| `$page-workspace`, `$page-reading`, `$page-wide`, `$page-gutter` | `64rem`, `40rem`, `90rem`, `var(--space-8)` |
| `$card-padding` | `var(--space-4)` |
| `$card-padding-phone` | `null` (a card keeps `$card-padding` on a phone) |
| `$toolbar-gap` | `var(--space-1)` |
| `$breakpoints` | `()`, merged over plinth's: `narrow` 30rem, `small` 36rem, `phone` 40rem, `medium` 48rem, `compact` 52rem, `regular` 56rem, `large` 60rem, `tablet` 64rem, `wide` 84rem, `broad` 90rem, `desktop` 100rem |

`$breakpoints` moves or adds named breakpoints without restating the rest:
`$breakpoints: ("phone": 42rem)`.

## Showcase

`showcase/` is an interactive gallery of every component: a page for
each, with controls for its props and its content, shown in a narrow
frame (drag its corner to widen it) beside a wide one. Run it with
[trunk]:

```sh
cd showcase
trunk serve --port 8095 --open
```

Pick a free port. A new component lands with its showcase page: add it to
`PAGES` in `showcase/src/pages/mod.rs`, listing the names it covers, or
`cargo test --workspace` fails.

## Checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo clippy --target wasm32-unknown-unknown --workspace -- -D warnings
cargo test --workspace
(cd showcase && trunk build)
```

[Yew]: https://yew.rs
[grass]: https://crates.io/crates/grass
[trunk]: https://trunkrs.dev
