# plinth

Opinionated layout primitives and components for [Yew] apps, built so
components keep their size and don't jitter as their content changes.

Each component comes with its Sass under `style/`, and its variants are
modifier classes set by props.

```toml
[dependencies]
plinth = { path = "../plinth" }
```

Or as a registry-style requirement, pointed at the checkout once in the
workspace root:

```toml
[dependencies]
plinth = "0.1"

[patch.crates-io]
plinth = { path = "../plinth" }
plinth-build = { path = "../plinth/plinth-build" }
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
`plinth::class`, so a misspelt class does not compile.

Every class plinth styles starts with `plinth-`, modifiers included
(`.plinth-stack.plinth-tight`), so none collides with an app's classes. A
constant is named after its class without the prefix, as an app's are:
`.plinth-page-head-title` is `plinth::class::PAGE_HEAD_TITLE`,
`.plinth-tight` is `plinth::class::TIGHT`.

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
   .plinth-card.army-sidebar {
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

7. **A guard against dead classes.** A class the Sass defines and no Rust
   writes is dead weight. `plinth_build::Usage` finds them in a test, so an
   app needs no guard of its own (`tests/style_classes.rs`):

   ```rust
   use plinth_build::{Config, Usage};

   #[test]
   fn every_styled_class_is_written() {
       Usage::new()
           .source(concat!(env!("CARGO_MANIFEST_DIR"), "/src"))
           .dynamic(["army-tone-*"]) // built at runtime; a trailing `*` is a prefix
           .foreign(["army-fc-row"]) // written by a dependency
           .assert_clean(&Config::new().local("style/").prefix("army-"));
   }
   ```

   Add `plinth-build` to `[dev-dependencies]`. A class counts as written when
   a source names its constant (`class::SIDEBAR`) or the class in a string
   literal; comments and code after a file's first `#[cfg(test)]` do not
   count. The test fails on a dead class and on an allowed pattern that
   matches no class, and prints the class, single-use and dead counts.
   `STYLE_INVENTORY=PATH` also writes `class<TAB>uses` lines to `PATH`.
   It is a test, not a build warning: a warning scrolls past in a build
   log, a test fails where the project already looks. plinth runs it on
   itself and on the showcase.

**One stylesheet per app.** Both `Styles` fill the same element,
`<style id="plinth-styles">` (`plinth::STYLE_ID`): with `css` it sets the
element's text to the app's stylesheet, which holds plinth's; without it, it
adds plinth's only when the element is missing. So whichever mounts first
(the app's, or a `<plinth::Styles/>` inside a component), the page ends up
with the app's stylesheet, once. Never link or inject plinth's CSS another
way beside it.

**Configuring plinth's defaults.** Local Sass cannot `@use "plinth" with
(...)`, since plinth compiles first. Pass the values to the build instead,
before `compile()`; they are plinth's `!default` tokens (see Configuration),
and only values change, never classes:

```rust
plinth_build::Config::new()
    .local("style")
    .prefix("army-")
    .with("text-md", "1.0625rem")
    .with("$radius-lg", "12px")
    .compile();
```

The `$` is optional and the value is any Sass expression. Without `.local`,
the same call configures plinth's own stylesheet.

The showcase (`showcase/build.rs`, `showcase/style/`) is set up this way.

## Adopting plinth with an app's own Sass compiler

An app that keeps its own Sass step (trunk's `<link data-trunk rel="scss">`
with dart-sass) and its own unprefixed classes can load plinth's sources
from the checkout instead, first in its root stylesheet, so its own rules
come after plinth's and refine them:

```scss
@use "../../../plinth/style";
```

Each partial `@use`s only `tokens` (and the menu's users `menu`), so an app
may load just the tokens and the partials of the components it renders, in
the order of `style/_index.scss`. Its stylesheet then carries none of the
rest:

```scss
@use "../../../plinth/style/tokens" as plinth-tokens;
@use "../../../plinth/style/layout" as plinth-layout;
@use "../../../plinth/style/buttons" as plinth-buttons;
```

The `as` names keep them clear of an app partial of the same name. A
component whose rules are missing renders unstyled, so add a partial when
the app starts rendering one of its components, and the partials of the
components it renders inside (a dialog's buttons and icons).

## Theming

The app supplies seven colour tokens: `--bg`, `--surface`, `--text`,
`--muted`, `--border`, `--accent` and `--shadow-1`. plinth derives eight more
from them, each a default the app overrides by name (or with `Config::with`,
as `$danger` and the rest):

| Token | Default | For |
| --- | --- | --- |
| `--danger` | `#d92d20` mixed 70% with `--text` | errors, destructive actions |
| `--success` | `#12a150` mixed 65% with `--text` | confirmations |
| `--warning` | `#dc8a00` mixed 70% with `--text` | cautions |
| `--on-accent` | `--bg` | text and icons on an accent background |
| `--surface-raised` | `--surface` | menus, popovers, dialogs |
| `--scrim` | `rgb(0 0 0 / 0.45)` | what dims the page behind a dialog |
| `--field-border` | `--border` | a field's edge (inputs, select and picker faces) and a checkbox's, switch's and choice's |
| `--field-bg` | `--surface` | a field's fill |

The status hues are mixed toward `--text`, so they keep their contrast on a
light and a dark background alike. The defaults are computed on `:root`: an
app that sets its colours on a narrower element (a dark panel) sets these
there too. `--focus-ring-color` (default `--accent`) colours the focus ring.

plinth's own scales are custom
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
`$card-padding-phone`, `$toolbar-gap`, `$visibility-breakpoint`) are Sass-only: an app built with
`plinth-build` sets them with `Config::with`.

## Configuration

An app built with `plinth-build` sets these with `Config::with` (see
Adopting plinth); an app that compiles plinth's Sass itself uses `with (...)`.
plinth's values are defaults. Every token is a `!default` variable of
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
| `$dot-size`, `$corner-size` | `0.5rem`, `1.125rem` |
| `$glyph-size` | `1em` |
| `$skeleton-duration` | `1.4s` |
| `$check-size`, `$switch-width`, `$switch-height`, `$choice-width` | `1.25rem`, `2.5rem`, `1.5rem`, `6rem` |
| `$filter-width` | `10.5rem` |
| `$field-label-width`, `$field-message-line`, `$field-error-lines` | `12rem`, `1rem`, `1` |
| `$avatar-sm`, `$avatar-md`, `$avatar-lg`, `$avatar-xl` | `1.75rem`, `2.5rem`, `3.5rem`, `7rem` |
| `$page-workspace`, `$page-reading`, `$page-wide`, `$page-gutter` | `64rem`, `40rem`, `90rem`, `var(--space-8)` |
| `$card-padding` | `var(--space-4)` |
| `$card-padding-phone` | `null` (a card keeps `$card-padding` on a phone) |
| `$toolbar-gap` | `var(--space-1)` |
| `$focus-ring-color`, `$focus-ring-width`, `$focus-ring-offset` | `var(--accent)`, `2px`, `2px` |
| `$tooltip-max-width` | `24rem` |
| `$layer-sticky`, `$layer-overlay`, `$layer-dialog`, `$layer-toast`, `$layer-tooltip` | `10`, `100`, `200`, `300`, `400` (`$layer-dialog` is kept for compatibility and unused: dialogs sit at `$layer-overlay`) |
| `$menu-width`, `$menu-width-wide`, `$menu-width-wider`, `$menu-width-panel` | `12rem`, `16rem`, `20rem`, `34rem` |
| `$danger`, `$success`, `$warning`, `$on-accent`, `$surface-raised`, `$scrim`, `$field-border`, `$field-bg` | see [Theming](#theming) |
| `$visibility-breakpoint` | `"phone"` (where `phone-only` and `desktop-only` switch) |
| `$breakpoints` | `()`, merged over plinth's: `narrow` 30rem, `small` 36rem, `phone` 40rem, `medium` 48rem, `compact` 52rem, `regular` 56rem, `large` 60rem, `tablet` 64rem, `wide` 84rem, `broad` 90rem, `desktop` 100rem |

`$breakpoints` moves or adds named breakpoints without restating the rest:
`$breakpoints: ("phone": 42rem)`.

## Button

`Button` is a `<button>`, or with `href` an `<a>` styled as one. Its label
is its children; `icon` is the app's icon (`trailing` puts it after the
label). plinth styles only `.plinth-button`, never a bare `<button>`.

- `variant` (`ButtonVariant`): `Secondary` (the plain button, the
  default), `Primary` (`plinth::class::PRIMARY`), `Ghost` (`GHOST`, no
  fill), `Quiet` (`QUIET`, no fill or border until hovered), `Danger`
  (`DANGER`), `Success` (`SUCCESS`).
- `size` (`ButtonSize`): `Regular` (`--control-height`), `Small` (`SMALL`,
  `--control-height-small`), `Touch` (`TOUCH`, `--touch-height`).
- An icon without a label is the icon alone (`ICON`), square, named by
  `label` (`aria-label`). `fold` (`FOLD`) makes a button with both the icon
  alone below the phone breakpoint, its label kept for a screen reader.
- `pressed: Some(on)` is a toggle (`aria-pressed`), marked by its fill and
  border colour. `busy` sets `aria-busy`, keeps the label in the DOM and
  lays the IconSet's spinner over it; a busy button ignores clicks.
  `disabled` disables a button; a disabled link loses its `href` and gets
  `aria-disabled="true"`.
- `submit`, `onclick`, `title`, `testid` as on `IconButton`; `class` adds
  the app's classes. `button(variant, size, icon_only, fold)` gives the
  classes for an element that cannot be the component.

Size guarantees: the height is the size's token, whatever the label (one
line, cut with an ellipsis; the button never grows past its container).
The border is always there (`reserved-border`), so hover, pressed and the
variants change colours only; focus is an outline. Pressed never changes
the font weight. Busy hides the icon and label with `visibility`, so the
box keeps their width. Fold swaps at a fixed breakpoint, and the icon's
box does not change with it.

`IconButton` stays as it was, for an app that styles its own buttons (its
children are the icon and label together); a new button should be a
`Button`.

## Stack, Cluster and Card

`Stack` and `Cluster` (and `Card`) take a `tag` (`div` by default). A list
tag, `ul` or `ol`, renders with no bullets, margin or padding
(`plinth::class::PLAIN_LIST` is added for it), so it lays out exactly as a
`div` would; put `li`s in it. An app that adds `PLAIN_LIST` itself keeps
working. `stack_with` and `cluster_with` return the same classes for any
tag, so a list built from them adds `PLAIN_LIST` itself.

A page's column is `WORKSPACE_PAGE` or `LIBRARY_PAGE` (an index, whose
blocks lose their own margins); `LIBRARY_PAGE` + `READING` keeps its blocks
to the reading column (`--page-reading`), as a long entry's text.

## Checkbox, Switch, Choice and chips

All four mark a state by colour and background, never by weight, and
draw it in a box of a fixed size. A `Field` supplies a label, hint and
error; give the control `label` (an `aria-label`) when the field names it
and the control has no text of its own. Every component takes `class`.

- `Checkbox` (`.plinth-checkbox`): a native checkbox drawn in a
  `--check-size` box with the IconSet's `Check` (or `Minus`) over it.
  `checked`, `indeterminate` (set as the input's DOM property, drawn as a
  minus), `disabled`, `invalid` (`aria-invalid`), `on_change(bool)`,
  children as its text. `checkbox(disabled)` gives its classes.
- `Switch` (`.plinth-switch`): a `<button role="switch">`, its track
  `--switch-width` by `--switch-height` and its thumb moved by transform.
  `on` (`ON`), `busy` (a spinner in the thumb, clicks ignored),
  `disabled`, `invalid` (`INVALID`), `on_change(bool)`; `switch(on,
  invalid)` gives its classes.
- `Choice` (`.plinth-choice`): a segmented pick-one `radiogroup` of
  `ChoiceOption`s (`new(value, label)`, `.icon(html)`, `.icon_only(html)`,
  `.disabled(bool)`). `value`, `on_change(value)`, `label` (the group's
  name), `fit` (`ChoiceFit`: `Equal`, `Stretch` to fill the container
  (`STRETCH`), `Fixed` at `--choice-width` (`FIXED`)), `small` (`SMALL`),
  `invalid`, `disabled`; `choice(fit, small, invalid, disabled)`. Roving
  tabindex: the selected option, or the first enabled, is the one tab
  stop; the arrows (with wrap), Home and End select and focus the next
  enabled option.
- `ChipToggle` (`.plinth-chip`) and `ChipList` (`.plinth-chip-list`):
  filter chips. `on` (`aria-pressed`), `count` (`chip_count(n)` marks a
  zero with `ZERO`), `disabled`, `on_toggle(bool)`, children as the
  label; a list takes `label` and the chips as children.

Size guarantees: the checkbox box, the switch's track and thumb, and a
chip's height are tokens (`--check-size`, `--switch-width` and
`--switch-height`, `--control-height-small`), so no state, hover or
focus changes them (borders are reserved, focus is an outline). A
choice's options all take the width of the widest (or the container's, or
`--choice-width`), whichever is selected, and its height is the control
height; the selected one changes background and colour only, through the
`current` mixin, which now also matches `aria-checked="true"` and
`aria-pressed="true"`. A chip's count has room for two figures (tabular);
a list is at least one chip high, so an empty list does not collapse and
grow when chips arrive. New tokens: `$check-size`, `$switch-width`,
`$switch-height`, `$choice-width`.

## Tabs, SubTabs and NavList

The current tab or entry is marked by colour, background or an inset
shadow, never by weight, through the `current` mixin, so selecting one
changes no box. Every component takes `class`.

- `Tabs` (`.plinth-tabs`): a `tablist` of `TabItem`s (`new(value, label)`,
  `.icon(html)`, `.disabled(bool)`). `value`, `on_change(value)`, `label`
  (the list's name); `tabs(small)` gives its classes. Roving tabindex: the
  selected tab, or the first enabled, is the one tab stop; Left and Right
  (with wrap), Home and End select and focus the next enabled tab, from
  the focused one. Up and Down are left to the page. The selected
  tab takes the accent colour and an underline of `--tab-underline` drawn
  as an inset shadow. Icons are hidden below the phone breakpoint. Tabs
  past the width scroll sideways with the scrollbar hidden and the edge
  that has hidden tabs faded (the `scroll-fade` mixin, shared with
  `Toolbar`); no tab is hidden or reflowed.
- `SubTabs`: the same props, at the small control height (`SMALL`).
- `NavList` (`.plinth-nav-list`): a `nav` of a column of `NavItem`s
  (`new(value, label)`, `.icon(html)`, `.href(url)`, `.disabled(bool)`), a
  link with an `href` and a button without. `value` is the current entry
  (`aria-current="page"`; colour and a tint), `on_select(value)`, `label`;
  `nav_list()` gives its classes.

Size guarantees: a tab is `--tab-height` high (the control height, or the
small one) and as wide as its content whichever is selected; the underline
is a shadow, hover and focus add no border (focus is an inset outline). An
empty tab list keeps its height. A nav entry is at least the control
height with a reserved border, so the current one keeps its box. New
tokens: `--tab-height` and `--tab-underline` (custom properties on
`.plinth-tabs`); new mixin `scroll-fade`.

## Field and FormRows

`Field` is a label, the control (its `children`), a hint and an error,
stacked. `id` is the control's `id`, which the label's `for` names; the
control takes `aria-describedby={field_describedby(&id, has_hint,
has_error)}`, which names the hint's and the error's ids (`{id}-hint`,
`{id}-error`). plinth has no input of its own here, so the control is the
app's.

- `label`, `hint`, `error` and `help` are Html (`help` ends the label's
  row); `required` adds a `*` after the label (`aria-hidden`; the control
  sets `required` itself).
- `disabled` mutes the field (`plinth::class::DISABLED`) and `busy` sets
  `aria-busy` with a progress cursor (`BUSY`); neither changes the box.
  `class`, `testid` as elsewhere. `field(disabled, busy)` gives the classes.
- `reserve_error` (`RESERVE`, on `FIELD_ERROR`) keeps the error line's room
  while there is no error. The trade-off: the field is one line taller
  (`--field-message-line`) when it has no error, and an error longer than
  `--field-error-lines` (1) is cut with an ellipsis. Without it the error
  appears under the control and pushes what follows down.

`FormRows` is a grid of `FormRow`s (the same props as `Field`) sharing one
label column: as wide as the widest label up to half the grid, or with
`wide_labels` (`WIDE_LABELS`) `--field-label-width` (12rem) so every section's
controls start on one line. Below the phone breakpoint the label sits above
its control. `Fieldset` is a `<fieldset>` with a `legend` and an optional
`hint`; `FieldCheck` is a checkbox (its `children`) with its `label` after it,
as high as an input.

Size guarantees: with `reserve_error` the field's height does not change
when an error appears, nor does anything else (disabled, busy, focus,
required). A form row's label cell is at least `--control-height` high, so
the label is centred on a one-line control. The hint wraps (long words
break) and never widens the field.

## Pill, Badge, Dot, Count and StatusLabel

Short marks whose box is fixed, so a state, a figure or a word changing does
not move what is around them. Every component takes `class` and `testid`.
`Tone` (`Neutral`, `Accent`, `Success`, `Warning`, `Danger`, `Busy`, `Unset`)
is the one tone vocabulary, also `Notice`'s: a modifier (`ACCENT`, `SUCCESS`,
`WARNING`, `DANGER`, `BUSY`, `UNSET`; neutral has none) that only sets
`--tone`, from `--accent`, `--success`, `--warning` and `--danger`.

- `Pill` (`.plinth-pill`): a tinted label `--mark-height` tall, led by a dot
  of `--dot-size` in its tone. `tone`, `plain` (`PLAIN`, no dot), `on_click`
  (makes it a `<button>`, `PILL_BUTTON`; hover changes its fill only),
  `disabled`, children as the label, cut with an ellipsis in a narrow
  container. `pill(tone, plain, button)` gives its classes.
- `Badge` (`.plinth-badge`): a bordered short text, `--mark-height` tall and
  at least as wide. `tone`, `solid` (`SOLID`, filled, the accent by
  default), `corner` (`CORNER`, `--corner-size`, over the top end corner of
  its positioned parent by transform, so it takes no room);
  `badge(tone, solid, corner)`.
- `Dot` (`.plinth-dot`): a `--dot-size` circle. `tone`, `on` (`ON`; off is an
  empty ring in the same box), `label` (its accessible name; without one it
  is decoration); `dot(tone, on)`.
- `Count` (`.plinth-count`): a figure on a soft fill. `value`, `max` (99:
  above it the count reads `99+`), `hide_zero` (`HIDDEN`: `visibility`, so
  the box stays; a zero shown is dimmed by `ZERO`). Its `min-width` is the
  digits of `max` and one more for the `+`, in tabular figures, so the
  count holds across `0..=max+1` and a hidden one does not move its
  neighbours. The figure is `value.to_string()`; koine has no number
  formatting. `count(value, hide_zero)` and `count_text(value, max)`.
- `StatusLabel` (`.plinth-status-label`, `role="status"`): `states` (the
  text of every state), `current` (an index, or none), `tone`, `spinner`
  (reserves the slot), `busy` (spins in it), `pill`. Every state's text sits
  in one grid cell and the others are `HIDDEN` (and `aria-hidden`), so the
  label is as wide as its widest state and "Loading..." to "Saved" does not
  resize it. The spinner slot is a `--glyph-size` square that is there
  whether busy or not. `status_label(tone, pill)`.

Sizes: the height is `--mark-height` whatever the tone, hover or text; only
a pill's, badge's or status label's own text sets its width. A busy dot
pulses by opacity (still under `prefers-reduced-motion`).

## TextInput, Textarea, SearchField and InputGroup

Text controls at a control's height. All take `class`, `size`
(`InputSize`: `Regular` is `--control-height`, `Small` (`SMALL`) is
`--control-height-small`, `Touch` (`TOUCH`) is `--touch-height`; the height
never changes with the viewport), `invalid` (`INVALID` and `aria-invalid`),
`disabled`, `busy` (`aria-busy`, progress cursor) and `testid`. Inside a
`Field`, pass the field's `id` and `describedby={field_describedby(..)}`.

- `TextInput` (`.plinth-input`): `value` and `on_input` (the new text), `on_change`,
  `on_blur`, `on_keydown`, `kind` (`InputKind`: text, email, url, tel),
  `maxlength`, `list` (a `<datalist>`'s id), `inputmode` and `enterkeyhint`
  (a touch keyboard's kind and enter key),
  `placeholder`, `label` (the accessible name without a `Field`), `readonly`,
  `required`, `autofocus`, `node_ref`. With a `unit` (Html, `aria-hidden`)
  the input sits in a box (`INPUT_BOX`) with the unit after the text inside
  the field; the box takes the border and the focus ring, so the unit is
  part of the field's box and the text keeps its start.
- `Textarea` (`.plinth-textarea`): `rows` (default 3), at least `--input-height`
  high, resized vertically by the user only.
- `SearchField` (`.plinth-input-box`): a search icon, the text and a clear
  button. `label` is the placeholder and the accessible name, `clear_label`
  names the button, `busy` swaps the icon for a spinner in the same box.
  Clearing (the button or escape) calls `on_input("")` and returns the focus
  to the text. `input_ref` is the text input's ref; `on_keydown` sees each
  key first (preventing escape's default keeps the text). `LibraryToolbar` is a `SearchField` (class `LIBRARY_SEARCH`)
  and its filters, passing `input_ref` and `on_keydown` on; its `icon` prop is gone, the icon is the `IconSet`'s
  search glyph.
- `InputGroup` (`.plinth-input-group`): controls joined in a row. Give each
  member `group_classes(&detached)[i]` as its `class`: `GROUP_FIRST`,
  `GROUP_MIDDLE` or `GROUP_LAST` square the corners that join (a lone member
  takes none), and `detached[i]` starts a new run after a `--space-2` gap
  (`GROUP_DETACHED`). The members overlap by their one-pixel border, the
  hovered or focused one on top. No rule uses `:has()`, a sibling selector
  or `!important`. A `TextInput` in a group takes the room the buttons leave.
- Bare helpers: `input(size, invalid, busy)`, `input_box(size, invalid,
  disabled, busy)`, `textarea(size, invalid, busy)`.

Size guarantees: a field's box is the same empty or filled, long text or
short (the text is cut with an ellipsis, never wrapped or widened); hover,
focus and invalid change colours only (the border is reserved, the focus ring
is an outline); the SearchField's icon and clear slot are always laid out, the
button only hidden while empty, so typing moves nothing; a group's members of
one size are one height. New tokens: none. New partial: `_inputs.scss`.

## Stepper and SecretInput

- `Stepper` (`.plinth-stepper`, worn with `.plinth-input-box`): a number
  between a minus and a plus button. `value`, `on_change(i64)`, `min`, `max`,
  `step` (default 1), `label` (the number's accessible name), `less_label`
  ("Decrease") and `more_label` ("Increase"), `id`, `size` (`InputSize`),
  `invalid`, `disabled`, `busy`, `class`, `testid`. The buttons step onto
  multiples of `step` (7 by 5 goes to 10 or 5) and stop at the bounds; the
  arrow keys do the same, Page Up and Down ten steps, Home and End go to the
  bounds, Enter or leaving the field commits what is typed (clamped; text that
  is no number goes back). What is typed outside `min..=max` is marked invalid
  until committed. The buttons are out of the tab order. Helpers: `stepper(size,
  invalid, disabled, busy)`, `step_from`, `key_value`, `parse_number`.
- `SecretInput` (`.plinth-input-box`): a password field with show or hide,
  copy and clear slots (`.plinth-input-action`, the `INPUT_CLEAR` button's
  look). `value`, `on_input`, `label`, `on_copy` (also writes the clipboard),
  `show_label` ("Show"), `hide_label` ("Hide"), `copy_label` ("Copy"),
  `clear_label` ("Clear"), `placeholder`, `reveal_ms` (`REVEAL_MS`, 3000: a
  shown secret hides itself, showing again restarts the time; clearing
  hides it), `id`, `size`, `invalid`, `disabled`, `busy`, `autofocus`,
  `class`, `testid`. The strings are props; an app passes its own words.

Size guarantees: the Stepper's number has the room of the longest of `min`
and `max` in `ch` with `numeric` figures, so its box is the same from the
least value to the most, the buttons only disabled at a bound; the
SecretInput's box is the container's width, dots or text, and its three slots
are always laid out (hidden, not removed, while empty). New classes:
`STEPPER`, `STEPPER_CONTROL`, `STEPPER_BUTTON`, `INPUT_ACTION`. New glyphs
`Eye`, `EyeOff` and `Copy`. New partial: `_stepper.scss`. New tokens: none (`--stepper-chars` is
set per Stepper).

## Heads

`PageHead` and `SectionHead` take one set of props (`HeadProps`):

- `title` is Html; `icon` goes before its text in the heading, `subtitle` is a
  muted line under the row, `crumbs` (a list of links) stack above it, and
  `back` is the inline "back / title" crumb. Each is its own prop.
- `help` follows the heading as its sibling, in `.plinth-head-heading` with
  the `suffix`, so the heading's text and test id exclude the help button.
  The group stays on one line: a long title wraps inside its own box, with
  the help beside it.
- `SectionHead` takes the modifiers `plinth::class::LABEL` (small-caps label),
  `LIST` and `LIST` + `NESTED` (smaller titles) in `class`; `HEAD_END` pushes a
  child to the row's end.
- A head is at least `--control-height` high, so actions coming and going
  never move what follows; a `LABEL` head holds none and is its line's
  height. Neither has outer margins, also when its class is on a heading
  element (`<h3 class="plinth-section-head plinth-label">`).
- `PageHead` takes `COMPACT` (actions as icons, one row on a phone) and
  `ONE_ROW` (a crumb head that stays one row on a phone, the back link's
  words, `PAGE_HEAD_BACK_LABEL`, hidden) in `class`.

## Icons

plinth's own chrome (a dialog's close, a select's chevron, a busy spinner)
draws its icons from an `IconSet`, and the app keeps its own icons in the same
set. The set is opaque, built with a builder, so adding a glyph to plinth never
breaks an adopter. `Glyph` (`Close`, `ChevronDown`, `ChevronUp`, `ChevronRight`,
`Check`, `Plus`, `Minus`, `Spinner`, `Grip`, `Ellipsis`, `Search`, `Eye`,
`EyeOff`, `Copy`) is `#[non_exhaustive]`: a `match` on it needs a wildcard. It is
the one place plinth uses a context for something other than a runtime setting,
and it carries icons only: classes never go through it.

`IconSet::default()` holds nothing of its own: a glyph the app did not set is
the built-in inline svg, built when it is first drawn, so an app that
overrides a glyph does not build the built-in one. Without a provider plinth
uses the default set, so it renders standalone. An app supplies its set once at
its root:

```rust
let set = IconSet::default()
    .with(Glyph::Close, html! { <MyIcon name="x" /> })
    .with_named("trash", html! { <MyIcon name="bin" /> });

html! {
    <ContextProvider<IconSet> context={set}>
        <App/>
    </ContextProvider<IconSet>>
}
```

`<Icon glyph={Glyph::Close} class=.../>` renders a glyph and `<Icon
name="trash"/>` an app icon; `use_icon(glyph)`, `use_icon_named(name)` and
`use_icons()` (`get`, `get_named`) return icons for a component of your own,
and `icon(class, glyph)` gives the box's classes (`plinth::class::GLYPH`, plus
`SPIN` for `Spinner`). A name the set lacks renders the same empty box, and a
debug build warns in the console with the name. Cloning a set shares its icons.

Size guarantees: the glyph sits in a `.plinth-glyph` box of `--glyph-size`
(`1em`, so it follows the text) square, `overflow: hidden`, and what fills it
is stretched to it. A swapped set, whatever its icons' own width and height,
cannot change a control's size. Icons should draw in `currentColor`. The
spinner's box turns (slower under `prefers-reduced-motion`); the set's
spinner need not animate itself.

## Notice, ErrorBox and EmptyState

`Notice` is an inline message. `tone` (`Tone`, the accent by default;
see above), coloured from `--accent`, `--success`, `--warning` or
`--danger`; `Neutral` is muted. `icon` is the app's icon, before the text; `action` is Html
(a link or button) after it; `onclose: Some(callback)` shows a close button
(`close_label`, "Dismiss", names it). The notice does not remove itself: the
app stops rendering it, and the space it held collapses at once, which is
intended. A `Warning` or `Danger` notice has `role="alert"`, the others
`role="status"`. `notice(tone)` gives the classes.

`ErrorBox` is an error, in `--tone: var(--danger)`: `title`, the message (its children), `cause` (the
technical text, cut to three lines, whole in its `title` attribute) and
`onretry` (a small button, `retry_label`, "Try again"; `retrying` makes it
busy and ignore clicks).

`EmptyState` is what stands where there is nothing: `title`, `icon`, `hint`
and actions (its children). `compact` (`COMPACT`) is one row for a list or a
table. `empty_state(compact)` gives the classes.

Every one takes `class` and `testid`. Parts: `NOTICE_ICON`, `NOTICE_BODY`,
`NOTICE_ACTION`, `NOTICE_CLOSE`, `ERROR_BOX_TITLE`, `ERROR_BOX_MESSAGE`,
`ERROR_BOX_CAUSE`, `ERROR_BOX_ACTIONS`, `EMPTY_STATE_ICON`,
`EMPTY_STATE_TITLE`, `EMPTY_STATE_HINT`, `EMPTY_STATE_ACTIONS`.

Size guarantees: a notice is at least one small control plus its padding
high, its text centred in that, so one line is as high with a close button
and an action as without; its text wraps, it never widens. An error box
keeps its retry button's box while retrying. A compact empty state is
exactly `--control-height` high whatever it says (title and hint cut to one
line, its actions kept at the end), so it replaces a row of that height
without moving what follows; the regular one grows with its text.

## Toast, ToastHost and SaveBar

`ToastHost` is where toasts stand: fixed over the window's bottom
(`--layer-toast`), centred, newest last, in an always-present polite live
region. It renders in the overlay layer (see `OverlayHost`), wherever the
app places it, so an open dialog never puts it out of reach. Fixed, it takes no room: a toast coming or going moves nothing.
`raised` lifts it by `--toast-tray`, the height of a tray or bar the app
keeps at the bottom (0 unless the app sets it on `:root`). `toast_host(raised)`
gives the classes.

`Toast` is one message: `tone` (`Tone`, the accent by default), `icon`, the
text (its children), `action` (a label such as "Undo") with `onaction`,
`ondismiss` (closed or timed out), `duration` (milliseconds, `TOAST_MS` =
8000; 0 keeps it until dismissed), `shortcut` (Ctrl or Cmd+Z takes the action
while it shows, unless typing in a field) and `close_label`. The clock stops
while the pointer is over the toast or focus is in it and starts afresh
after; the action and the dismissal happen once between them. Give each
toast its own `key`. `toast(tone)` gives the classes. The app owns the list:
it stops rendering a toast when `ondismiss` or `onaction` is called.

`SaveBar` is a form's bar of unsaved changes: `dirty`, `busy` (the save
button's spinner, clicks ignored), `disabled`, `label` ("Unsaved changes"),
`status` (how the last save went), `onsave`, `ondiscard` (shows Discard),
`save_label`, `discard_label`, `submit` (Save submits the form it stands in),
`sticky`, and children (further actions). While `dirty` it holds an
`UnloadGuard`, the browser's own prompt on closing or reloading the tab
(`guard=false` leaves that to the app, which can call `use_unload_guard(active)`
itself; `unsaved()` says whether any guard is held). `save_bar(idle, sticky)`
gives the classes.

Every one takes `class` and `testid`. Parts: `TOAST_ICON`, `TOAST_BODY`,
`TOAST_ACTION`, `TOAST_CLOSE`, `SAVE_BAR_NOTE`, `SAVE_BAR_STATUS`,
`SAVE_BAR_ACTIONS`. Modifiers: `RAISED`, `IDLE`, `STICKY`, the tones. Token:
`--toast-max-width` (28rem); `--toast-tray` is the app's to set.

Size guarantees: a toast is at least one small control plus its padding
high, its text wrapping within `--toast-max-width` and the window. The
`SaveBar` choice is to keep its room: it is always in the flow at one control
plus padding high, hidden (`visibility`) while clean and not removed, so the
form's first edit moves nothing; `sticky` keeps it at the bottom of the scroll
container, still in the flow. Its save button is busy in its own box.

## Spinner, Skeleton and Loadable

`Spinner` is a spinning glyph (`Glyph::Spinner` from the icon set) in the
icon box: inline, `1em`, in the colour of the text around it. `label` makes
it a `role="status"` with that name; without one it is decoration. `Button`'s
busy mark is a `Spinner`.

`Skeleton` is a placeholder of the box its content will have. `shape`
(`SkeletonShape`): `Line` (`LINE`, one line of the text around it, `1lh`),
`Block` (the default, `--control-height`, or `--skeleton-height`), `Row`
(`ROW`, one `RowListItem` high) or `Image(Ratio)` (an `Image`'s ratio
classes, `POSTER` and the rest). `count` stacks several in a column
(`SKELETON_GROUP`); the last of several lines is `SHORT`. A group reserves
the 1px border a `RowList` does, so `count` skeleton rows are exactly as high
as `count` one-line rows. It is `aria-hidden` and shimmers, still under
`prefers-reduced-motion` (`--skeleton-duration`). `skeleton(shape, short)`
gives the classes.

`Loadable` is one box for content that loads. `state` (`LoadState`):
`Loading` shows the placeholder (`placeholder`, or `rows` skeleton rows),
`Ready` the children, `Empty` an `EmptyState` (`empty` replaces it,
`empty_title` names it) and `Error` an `ErrorBox` (`error`, `error_cause`,
`onretry`, `retrying`). The empty state waits `empty_delay` (250 ms) before
it shows, the placeholder standing in, so it never flashes before data.
While it loads it is `aria-busy` and a visually hidden `role="status"` reads
`label` ("Loading"). Every one takes `class` and `testid`.

Size guarantees: while loading, a `Loadable` is at least `rows` rows high the
first time and at least the height it last had settled (ready, empty or
error) afterwards, so loading to ready and ready to loading again move
nothing when the placeholder has the content's rows. Once settled it is as
high as its content, so a shorter list shrinks. An empty state or an error
taller than the box grows it; use `EmptyState` `compact` to stay inside.

## Image and Avatar

`Image` is a picture in a box whose ratio is fixed before it loads, so the
page does not shift as pictures arrive. Props:

- `ratio` (required; there is no default, so no image lacks a box):
  `Ratio::Poster` (2/3), `Screenshot` (16/9), `Banner` (21/4), `Square`, or
  `Explicit(width, height)`, which also sets the `<img>`'s `width` and
  `height` attributes.
- `src` (`Option`), `alt`, `lazy` (default on: `loading="lazy"`), `class`,
  `onload`, `onerror` and `testid`.

Its modifiers are the ratio's classes (`POSTER`, `SCREENSHOT`, `BANNER`,
`SQUARE`, `RATIO`) and its status's: `LOADING`, `ERROR`, `EMPTY` (no `src`) and
`LOADED`. The status follows the `<img>`'s `load` and `error` events and starts
over when `src` changes.

Size guarantees: the box is `aspect-ratio` at the width it is given (the
container's, unless `class` sets one), and the picture is laid over it with
`object-fit: cover`, so the loading, error and empty placeholders (a glyph
from the icon set, centred) and the loaded picture all fill the same box.
A picture of another shape is cropped, never stretched; the box never
resizes.

`Avatar` is a round portrait of a fixed size: `name` (its initials, the first
letters of its first and last word, show behind the picture and are its
accessible name), `src`, `size` (`AvatarSize::Small`, `Medium` by default,
`Large`, `Huge`: the `--avatar-sm`, `-md`, `-lg` and `-xl` tokens), `lazy`,
`class` and `testid`. The box is `--avatar-*` square whether the picture is
loading, failed, missing or loaded. `initials(name)` is the same function for
a place of your own.

## OverlayHost

Popovers, menus, selects, tooltips, dialogs and toasts render in the overlay
layer, a portal out of whatever scrolls or clips their owner. Its events
still bubble through the owner. `OverlayHost` is optional and gives the app
its own layer:

```rust
html! {
    <div class="my-app" data-theme="dark">
        <OverlayHost>
            <App/>
        </OverlayHost>
    </div>
}
```

- Overlays render in a `.plinth-overlay-host` element after its children,
  so they inherit what the element around it sets (colour variables, a
  theme scoped to a sub-element). It adds no box of its own. Place it where
  no ancestor has `transform`, `filter`, `contain` or `will-change`, or
  fixed overlays are placed against that ancestor.
- It keeps the app's stack of open overlays, so Escape and a press outside
  close the innermost first. Two hosts in two separate roots keep separate
  stacks. A host inside another gets its own layer, for a sub-tree with
  its own colours, and shares the outer one's stack. The colours derived
  from the app's (`--surface-raised`, `--on-accent`, `--scrim`, the status
  hues) resolve on `:root`, so a scope that sets its own colours sets
  those there too, as `--surface-raised: var(--surface)`.
- Props: `class` (on the host element) and `testid`. `overlay_host(class)`
  gives the classes.

Without one, nothing changes from before: overlays go to `#plinth-overlay`
(`plinth::OVERLAY_ID`), made on first use as the last child of `<body>`,
with one stack per thread. Because it is a body child, it does not inherit
colours an app sets below `:root`. On the server there is no layer, and
overlays render in place.

For a component of your own, `use_overlays()` returns the nearest host's
`Overlays` (or the fallback), and `.portal(html)` renders into it.
`use_popover` already takes its place on the stack from it. An overlay the
app draws itself (a full-screen panel, a drawer) joins the stack with
`use_overlay(open, on_escape)`: while it is open, Escape calls `on_escape`
only when nothing opened after it is still open, so a menu or dialog opened
from it closes first, and not when an element in it acted on Escape itself
(by preventing the key's default). `Overlays::any_open()` says whether
anything but a tip is open, for an app's own keyboard shortcuts to stand
aside meanwhile. The free
`overlay(html)` always uses the fallback layer, since a plain function
cannot see a context. Prefer `use_overlays().portal(html)` in components.

Every stacked overlay (popover, menu, list, hover card, dialog) is at
`--layer-overlay`, and the one opened later is later in the host, so on top.
Toasts and tooltips (`--layer-toast`, `--layer-tooltip`) stay above. Within
this, ordering is by opening, not kind. An open dialog puts the page and the overlays opened before it out of
reach (`inert`), but never the toast region: a toast's Undo over a dialog
still works, and it is still announced.

## Popover

`Popover` is a surface beside its anchor, rendered while `open` in the
overlay layer (see `OverlayHost`), so no scrolling or clipping box around
the owner cuts it, and its events still bubble through the owner. Props:

- `anchor` (a `NodeRef` on the element it opens beside, where focus returns),
  `open`, and `on_dismiss: Callback<Dismiss>`: the owner closes it.
- `side` (`Side::Below` by default, `Above`, `Start`, `End`) and `align`
  (`PopoverAlign::Start`, `Center`, `End`).
- `sheet` (on by default, `plinth::class::SHEET`): below `$sheet-breakpoint`
  (the phone's) it is a sheet from the bottom over a `--scrim`, with
  `title` as its heading and a close button (`close_label`, "Close").
- `title` is also its accessible name (`role="dialog"`); `class`, `testid`.
- `focus` (a `NodeRef`): the element that takes focus once it is placed.

Dismissal (`Dismiss`), only while no overlay opened after it is open:
`Outside`, a `pointerdown` outside the anchor and the popover, keeps focus
where the press put it; `Escape` and `Close` (the
sheet's button) return focus to the anchor, as does any close that leaves
focus nowhere (it was inside the popover).

`use_popover(open, anchor, popover, side, align, on_dismiss)` is the hook
under it, for a component that renders its own surface (a menu, a select):
it returns `Option<Placed>` and does the measuring, following and
dismissing. `place(anchor, (width, height), viewport, side, align)` is its
pure placement: the given side unless the popover does not fit and the
opposite side has more room, then moved back inside the viewport, its
height capped (`max_height`) by the room on the side it took.
`popover(sheet, placed)` gives the classes (`POPOVER`, `SHEET`, `PLACED`).

Size guarantees: it is hidden (`visibility`) until it has been measured and
placed (`PLACED`), so it never shows at 0,0 first. It is `position: fixed`
in a layer that lays nothing out, and nothing locks the page's scroll, so
opening it never moves the page, its anchor or the scrollbar. Its width is
its content's up to `--popover-max-width`, whichever side it lands on, so a
flip moves it without reflowing it; what is taller than the room scrolls in
`.plinth-popover-body`, its scrollbar's room kept. It follows the anchor on
scroll, resize and a change of either's size.
## RowList, PickRow, SelectionBar, ActionRow and CardGrid

`RowList` is a column of rows (`tag`, `ul` by default, with no bullets, margin
or padding). `divided` (`DIVIDED`) draws a line above every row but the first;
`boxed` (`BOXED`) gives it a bordered, rounded surface. `RowListItem` is a
row: `name` (one line, cut), `detail` (under it, muted, cut), and its
children, which end the row (an `ActionRow`, a `Checkbox`). `top` (`TOP`) is a
tall row: parts to the top, the name wraps. `row_list(divided, boxed)` and
`row_list_item(top)` give the classes. Parts: `ROW_LIST_MAIN`,
`ROW_LIST_NAME`, `ROW_LIST_DETAIL`.

`PickRow` is a button that is a whole row choosing one thing: `name`,
`detail`, `mark` (before the name), children (after), `current`
(`aria-current`, marked by background only), `disabled`, `onclick`.
Parts: `PICK_ROW_MARK`, `PICK_ROW_NAME`, `PICK_ROW_DETAIL`.

`SelectionBar` is the band under a list: `count`, `label` (the count as text,
in a `role="status"`) and the actions as children. It is always in the flow
at one height: with `count == 0` it is `IDLE` (`visibility: hidden`), so
picking never moves the list. `inset` (`INSET`) levels its sides with the
rows; `sticky` (`STICKY`) keeps it at the bottom of its scroll container.
Part: `SELECTION_BAR_COUNT`. `selection_bar(idle, inset, sticky)`.

`ActionRow` is actions that wrap; `nowrap` keeps one line and never shrinks.
`overlay` (`OVERLAY`) lies over the end of its row, taking no room, with
`opacity: 0` and `pointer-events: none` until the row (a `RowListItem`) is
hovered or has focus. Its controls stay in the tab order, so focus reveals
it. It reads `--overlay-opacity`, `--overlay-events` and `--overlay-tint`,
which a row sets. `action_row(overlay, nowrap)`.

`CardGrid` lays cards in as many columns as fit (`tag`, `ul` by default).
`size` (`GridSize`): `Normal` (columns of at least `--card-grid-min`), `Wide`
(`--card-grid-wide`) and `Narrow` (`--card-grid-narrow`, and
`--card-grid-narrow-phone` with a closer gap on a phone). `card_grid(size)`.

Every one takes `class` and `testid` (the parts' children aside).

Size guarantees: a row is at least `--control-height` high, as a compact
`EmptyState` is, so one replaces the other without moving what follows. Every
row reserves its top border, so `divided`, hover, focus and a chosen
`PickRow` change no box. An overlay `ActionRow` takes no room, so revealing
it moves nothing. A selection bar keeps its height whether anything is
picked or not. A card grid's cards are as tall as their row.

## Reorder and DragHandle

`Reorder` is a list whose rows move by a drag on their `DragHandle` or by
keyboard. `rows` is a `Vec<ReorderRow>` (`ReorderRow::new(key, name)`,
`.detail(html)`, `.end(html)`; the key is stable across moves), and
`on_move((from, to))` reports a move, `to` an index in the list without the
row: apply it with `move_item(&mut list, from, to)`. `divided`, `boxed`
(as `RowList`'s), `disabled`, `class`, `testid`. The rows are
`RowListItem`s (`.plinth-reorder-row`), the handle in their `start` slot
(a new `RowListItem` prop: what begins a row).

- Pointer: press a handle and move past 4px; the row lifts (`LIFTED`) and
  follows the pointer, kept within the list. Release drops it, Escape or a
  cancelled pointer puts it back.
- Keyboard, on a focused handle: Space or Enter lifts, the arrows, Home and
  End aim, Space or Enter drops, Escape or leaving the handle cancels.
  Alt and an arrow moves the row at once. Focus follows the moved row.
- The row it lands beside is marked `DROP_BEFORE` or `DROP_AFTER`.
- `text: Callback<ReorderText, String>` gives the handle's name
  (`Handle`), its hint (`Hint`, its `aria-describedby`) and the
  announcements (`Lifted`, `Moving`, `Dropped`, `Cancelled`, positions from
  1), read from a `role="status"` region: an app passes its phrasebook's
  words; the default is `reorder_text` ("Moved Alpha to 3 of 5").
- `DragHandle` is the grip alone (the `IconSet`'s `Grip`): `label`, `lifted`,
  `disabled`, `describedby`, the pointer, key and blur callbacks, `class`,
  `testid`. Helpers: `reorder_row(index, lift)`, `drag_handle(lifted)`,
  `landing(from, slot)`, `slot_at(middles, y)`, `key_target(key, to, count)`.

Size guarantees: nothing reflows during a drag. The rows are measured once
at the press; the lifted row moves by a transform (`--reorder-shift`) and
keeps its own room, so no gap of another height opens and the list's
height holds; the drop marks are inset box shadows, never borders or
margins; the order changes once, on the drop. A keyboard lift moves only
the marks. The handle is `--control-height-small` square with a reserved
border; hover, focus and lifted change colours only. New classes:
`REORDER`, `REORDER_ROW`, `LIFTED`, `DROP_BEFORE`, `DROP_AFTER`,
`DRAG_HANDLE`. New partial: `_reorder.scss`. New tokens: none.

## Dialog

`Dialog` is a modal box over a `--scrim`, rendered while `open` in the
overlay layer (a portal, like `Popover`, not a native `<dialog>`: a modal
`<dialog>` makes everything outside the top layer inert, the overlay layer
included, so a popover or menu opened inside it could not be used). Props:

- `open`, `on_dismiss: Callback<Dismiss>` (the owner closes it), `title`
  (its heading and accessible name), `icon` (before the title).
- `size`: `DialogSize::Default` (`--dialog-width`), `Wide`
  (`plinth::class::WIDE`, `--dialog-width-wide`) or `Tall` (`TALL`, fixed at
  `--dialog-height-tall`).
- `dismiss`: `DialogDismiss::Any` (Escape, the close button, a press on the
  scrim), `NoOutside` (not the scrim) or `Actions` (no close button, Escape
  does nothing). `DialogDismiss::allows(Dismiss)` tells which.
- `actions` (the foot's end, the primary last) and `actions_start` (apart,
  at its start: a Delete), each an `ActionGroup` in `.plinth-dialog-foot`.
- `sheet` (on by default, `SHEET`): below `$sheet-breakpoint` a sheet from
  the bottom, up to `--dialog-sheet-height`. `close_label`, `class`, `testid`.

`dialog(size, sheet)` gives the classes. Parts: `.plinth-dialog-layer` (the
scrim), `-head` (a `SectionHead`), `-body` (what scrolls), `-foot`.

Behaviour: opening it moves focus to an `[autofocus]` or `[data-autofocus]`
element, else the body's first control, else the box; Tab and Shift+Tab
wrap inside it; the page and the overlays open before it are `inert`. A
dialog opened from a dialog stacks over it, the one beneath dimmed by its
scrim and out of reach. Escape and a press outside close only the innermost
overlay, a popover included: overlays are kept on one stack. On close focus
returns to what had it.

Size guarantees: while a dialog is open the root takes
`.plinth-scroll-locked` (no scrolling), and `.plinth-gutter-kept` when the
page had a scrollbar, so the scrollbar's room stays and the page neither
moves nor scrolls; an app whose root already has `scrollbar-gutter: stable`
keeps it either way. The box hangs `--dialog-top` from the window's top,
never centred, so content that loads or grows moves only its bottom edge;
it is at least `--dialog-min-height` and at most the window's room, beyond
which the body scrolls with its scrollbar's room kept and the foot stays
pinned below it. Its width is fixed by its size, never by its content. A
`Tall` dialog keeps one height whatever it holds; a sheet grows upward
from the bottom edge.

## Confirm, ConfirmPopover, ConfirmButton and ConfirmInline

Asking before an action, in three places. `Confirm` is a `Dialog` (rendered
while `open`): `title` (its heading and name), `explanation`, `details` (a
list of what it affects), `accept` (the accepting button's label) with `icon`,
`cancel_label` ("Cancel"), `danger` (on by default: the accepting button is
`Danger` and Cancel has the focus, so Enter never destroys by habit; off, it
is `Primary` and has the focus), `busy`, `error` (a danger `Notice` under the
explanation), `on_accept` and `on_cancel`. Cancel, Escape, the close button
and a press on the scrim all call `on_cancel`; the owner sets `open` to
false. While `busy` the accepting button spins and nothing closes it.

`ConfirmHost` and `ask` are promise-style confirms: render one `ConfirmHost`
near the app's root, then `ask(ConfirmRequest::new(title, explanation))`
from any code and `.await` the `Answer`: `true` when accepted, `false` when
cancelled, when another question replaced it, or when no host is mounted.
`ConfirmRequest` has the same fields as `Confirm`, public.

`ConfirmPopover` is the same beside an element: `anchor`, `open`, `side` and
`align` as `Popover`, `title` (its name, and the sheet's heading on a phone),
the same text, `danger`, `busy` and `error`, `on_accept` and `on_cancel`
(Cancel, Escape and a press outside; not while busy). The focus goes in once
it is placed (`Popover`'s `focus` prop) and returns to the anchor.
`ConfirmButton` is a `Button` (`variant`, `size`, `icon`, `busy`, `disabled`)
that opens one beside itself and calls `on_accept` once accepted.

`ConfirmInline` replaces a button by No and Yes in the same box, for a row's
own delete: the button's `variant`, `size`, `icon`, `disabled`, children as
label, `no_label`, `yes_label`, `question` (the group's accessible name),
`danger` (Yes is `Danger`, else `Primary`), `busy` (Yes spins, No is
disabled), `on_accept`. It holds its own state, or the owner's: `asking`
(`Some`) with `on_ask`. No takes the focus; Escape and No go back and focus
returns to the button; a click does not bubble to a clickable row.

Every one takes `class`, and `testid` (`confirm-cancel` and `confirm-accept`
name the buttons). Classes: `CONFIRM` and `CONFIRM_TEXT` (the body),
`CONFIRM_POPOVER`, `CONFIRM_INLINE` with its parts `CONFIRM_IDLE` (the button)
and `CONFIRM_ASK` (No and Yes), and `AWAY` on the part not shown;
`confirm_inline(part, away)` gives them. No new tokens.

Size guarantees: `ConfirmInline` lays the button and the No and Yes in one
grid cell and hides (`visibility`) the one not shown, so its box is the same
asking or not: as wide as the wider of the two, as high as the button, and a
row it ends never moves. `ConfirmPopover` is `--popover-max-width` wide
whatever it says, and a flip moves it without reflowing; the page behind a
`Confirm` holds still, and the dialog hangs from the top, so an error line
only grows its bottom edge. A `ConfirmButton` keeps the button's box open or
closed.

## Tooltip, HoverCard and ReasonButton

All three place themselves with `use_popover` in the overlay layer, so they
inherit Popover's placement, following and Escape or press-outside dismissal.

`Tooltip` is a short tip beside its anchor. Props: `anchor` (a `NodeRef`),
`text` (nothing is shown, and nothing described, while it is empty), `side`
(`Side::Above` by default), `delay` (`TOOLTIP_DELAY`, 250 ms of the pointer's
rest), `id`, `class`, `testid`. Keyboard focus (`:focus-visible`) shows it at
once; a touch pointer never does; a press, Escape, leaving or losing focus
hides it. The anchor gets `aria-describedby` naming the tip while it has
text. The tip (`.plinth-tooltip`, `plinth::class::TOOLTIP`; `tooltip(placed)`
gives its classes) is `--layer-tooltip`, `pointer-events: none`, hidden until
`PLACED`, and as wide as its text up to `--tooltip-max-width` (`$tooltip-max-width`,
24rem), wrapping beyond. `DataTips`, mounted once, gives every element with a
`data-tip` attribute the same tip (none when it says what the element already
says, or while its menu is open, `aria-expanded="true"`); it places the tip by
a fixed stand-in (`TIP_ANCHOR`), and hides on scroll and Escape.

`HoverCard` is a richer card (`.plinth-hover-card` on a `.plinth-popover`,
`hover_card(placed)`): props `anchor`, `side`, `align`, `delay`
(`HOVER_CARD_DELAY`, 400 ms to open and to close), `title` (its accessible
name), `class`, `testid` and the card's children. It opens on hover or
keyboard focus and stays open while the pointer is over the anchor or the
card, so a link in it can be reached.

`ReasonButton` is a `Button` (the props `class`, `variant`, `size`, `icon`,
`submit`, `onclick`, `label`, `testid`, children) with `reason: Option<AttrValue>`
and `busy` with `working: Option<AttrValue>`: its tooltip says the reason, else
what it is doing (`reason_text`). A button with a reason is `unavailable`
(`aria-disabled`, not `disabled`), so it keeps focus and hover and a click does
nothing. `Button` has `unavailable` and `node_ref` props for this.

Size guarantees: the tip and card are `position: fixed` in the overlay layer,
hidden until measured, so showing them never moves the page, the anchor or the
scrollbar. A `ReasonButton` is one `<button>` whether it is available,
unavailable or busy: the same node, box and focus across the states (the busy
spinner and the aria state change, the width does not).

## Menu, RowActions, SplitButton and ContextMenu

`Menu` is a `Button` (`aria-haspopup="menu"`, `aria-expanded`) opening a
list of actions (`role="menu"`, `plinth::class::MENU`) beside it in the
overlay layer, through `use_popover`. Props:

- `text` (the opener's visible content; without it the opener is its icon
  alone), `icon` (the app's; a text opener without one ends in the
  IconSet's chevron, an icon-only one shows its ellipsis), `label` (the
  accessible name of the opener and the list), `variant` and `size` (the
  opener's, `Regular` so it matches the buttons beside it), `opener_class`.
- `width` (`MenuWidth`): `Regular` (`--menu-width`), `Wide` (`WIDE`,
  `--menu-width-wide`), `Wider` (`WIDER`, `--menu-width-wider`), `Panel`
  (`PANEL`, fixed at `--menu-width-panel`); `tight` (`TIGHT`) drops the gap
  between rows.
- `side`, `align` (`End` by default), `disabled`; `open` and `on_toggle`
  for a menu its owner also opens another way; `class` (the list's),
  `testid` (the list's; the opener's is it suffixed `-toggle`).

Its children are the items: `MenuItem` (an action; `icon`, `detail` under
the label, `hint` at the end, `danger`, `disabled`, `onclick`),
`MenuCheck` (`menuitemcheckbox`: `checked`, `on_change`; the menu stays
open), `MenuRadio` (`menuitemradio`: `checked`, `on_pick`), `MenuSeparator`,
`MenuHeading` and `MenuNote` (a line such as "nothing here"). Each takes
`class`. `menu(width, tight, placed)` and `menu_item(danger)` give the
classes; `menu_step` and `typeahead` are the keyboard's pure parts.

Keyboard: focus moves to the first item once the list is placed; the
arrows (wrapping), Home and End move between enabled items, and typing
moves to the next item starting with what was typed (a pause of 0.7 s
starts over; one letter repeated cycles). A pick of an item or radio,
Tab, Escape or a press outside closes it; all but the press return focus
to the opener. A menu is on the one overlay stack, so Escape and a press
outside close a menu opened from a popover or a dialog before what it
opened from.

`RowActions` is a row's actions: `actions: Vec<RowAction>`
(`RowAction::new(icon, label, onclick)`, `.danger()`, `.pressed(on)`,
`.disabled(..)`, `.testid(..)`), shown as small quiet icon buttons, the
danger ones last (`ROW_ACTION_DANGER`, muted until hovered or focused),
then `children`, then the overflow `Menu` behind its ellipsis when `more`
holds its items (`more_label`, "More actions"; `more_disabled` keeps the
opener, disabled, so every row ends the same way). `disabled`, `open`,
`on_toggle`, `class`, `testid`.

`SplitButton` is a main action (`text`, `onclick`, `icon`, `busy`) joined
to a `Menu` of its variants (its children; `menu_label`, "More options";
`width`), `Primary` unless `variant` says otherwise, `size`, `disabled`,
`class`, `testid` (the menu's is it suffixed `-menu`).

`ContextMenu` wraps its children (`.plinth-context-target`,
`display: contents`, so it lays nothing out) and opens a menu of `items`
at the pointer on a right-click, or a press held for half a second on a
touch screen. Closed by Escape, a pick or Tab, it gives focus back to what
had it when it opened. `width`, `tight`, `label`, `disabled` (the browser's
menu instead), `on_toggle`, `class`, `testid`.

Size guarantees: the opener is a `Button`, so open and closed are the same
box (only `aria-expanded` changes). The list is fixed in a layer that lays
nothing out, hidden until placed. It is at least its width token wide and
never narrower than it opened (`--menu-open-width`, measured once on
open), so it does not follow its longest item as items change; past the
room on its side it scrolls (`scroll-area`), its scrollbar's room kept. Its
rows are `--control-height` at least, their border reserved, so hover and
focus change only the fill and border colour; a check's or radio's mark
keeps its box when off. Labels are one line, cut with an ellipsis.
## Table and SortHeader

`Table` is a table in a scroll area (`scrollbar-gutter: stable`, so rows
starting to scroll move nothing). `head` is the heading cells (`TableCell`
with `head`, or `SortHeader`s); the children are `TableRow`s of `TableCell`s.
`columns` is the count the `empty` row spans: `empty` (an `EmptyState` with
`compact`, or an `ErrorBox`) stands in the rows' place. `fixed` (`FIXED`)
sizes columns by the first row alone, so data arriving or a sort moves no
column; give a cell `width`, and `cut` to cut its text with an ellipsis.
`sticky` (`STICKY`, on the wrap) holds the head while the rows scroll past
`--table-max-height`. `bulk` is the bulk actions above the table: given,
its room (`TABLE_BULK`, one height) is always kept, and `bulk_active` shows
it, so selecting rows moves nothing; `IDLE` hides it. `numeric` on a cell
(`NUMERIC`) sets figures of one width, to the end. `table(fixed)`,
`table_wrap(sticky)`, `table_bulk(idle)` and `table_cell(head, numeric, cut)`
give the classes. Parts: `TABLE_FRAME`, `TABLE_WRAP`, `TABLE_ROW`,
`TABLE_CELL`, `TABLE_HEAD_CELL`, `TABLE_EMPTY`.

`SortHeader` is a heading cell with a button: `label`, `direction`
(`Option<SortDirection>`; `aria-sort` is `ascending`, `descending` or
`none`), `first` (the way a first press runs: names from A, sizes largest
first), `numeric`, `disabled` and `onsort`, called with the direction asked
for. `next_sort(current, first)` is the cycle: unsorted, `first`, the other
way, unsorted. The mark (`SORT_HEADER_MARK`, a chevron from the `IconSet`)
has its box whether sorted or not (`IDLE` hides it), so sorting never changes
the heading's width. `sort_header(numeric)` and `sort_mark(idle)`.

Every one takes `class` and `testid`. Size guarantees: a row and a heading
are at least `--control-height` high, every line is a reserved border, and
hover tints the row without changing a box; the mark and the bulk bar keep
their room. For markup plinth does not render, the mixins `table-base`
(`$fixed`), `table-head-cell`, `table-cell` and `table-row-hover` give the
same rules. New token: `--table-max-height`.

## Pager and Disclosure

`Pager` is page buttons in a fixed number of slots: first, previous, the
numbers with gaps (`…`), next and last. `page` (from 0), `pages`, `on_page`
(called with the page asked for), `slots` (default 7, at least `MIN_SLOTS`,
5), `edges` (default true: the first and last page buttons; hidden below the
phone breakpoint, and worth turning off in a narrow container), `label` ("Pages"),
`first_label`, `previous_label`, `next_label`, `last_label`, `page_label`
("Page", the number appended) and `disabled` and `busy` (the buttons keep
their room and ignore presses). `pager_slots(pages, page, slots)` is the pure
layout: every page when they fit, otherwise exactly `slots` entries, the
first and last page always among them and each gap hiding at least two
pages, so the count does not change with the page. `page_count(total, size)`
rounds up, at least one. The current page is marked by `aria-current`, its
colour and background, never its weight. `pager(disabled, busy)` and
`pager_button(edge)` give the classes. Parts: `PAGER_BUTTON`, `PAGER_GAP`,
`PAGER_MARK`, `PAGER_FLIP`, `PAGER_ECHO`, `PAGER_EDGE`.

`Disclosure` is a summary row that toggles its body: `title`, `summary` (what
it holds, beside the title while shut), `actions` (controls at the row's end,
outside the toggle), `open`, `on_toggle` (the state a press asks for), `id`
(the body's, named by `aria-controls` while open), `disabled`, `level` (a
`HeadLevel`: the toggle sits in that heading, `DISCLOSURE_HEADING`, its title
at a list head's size, for a section whose title is a heading), `class`,
`testid`, and the children as the body, rendered only while open.
`disclosure(open)` gives the classes (`DISCLOSURE`, `OPEN`),
`disclosure_summary(heading)` the toggle's (`HEADING`). Parts:
`DISCLOSURE_HEAD`, `DISCLOSURE_SUMMARY`, `DISCLOSURE_MARK`, `DISCLOSURE_TITLE`,
`DISCLOSURE_DETAIL`, `DISCLOSURE_ACTIONS`, `DISCLOSURE_BODY`.

Size guarantees: the Pager's slots are as many whatever the page, and each
button and gap is as wide as the widest page number (`--pager-chars`, the
digits of `pages`) with `numeric` figures, at least `--control-height`, so
its width holds from the first page to the last (in a container too narrow
the buttons shrink together, the same at every page); the current page adds
no weight. The Disclosure's row is `--control-height` high shut or open, the
mark turns inside its own box, and the title and summary are cut to one line;
only the body adds height. New classes: `PAGER`, `PAGER_BUTTON`, `PAGER_GAP`,
`PAGER_MARK`, `PAGER_FLIP`, `PAGER_ECHO`, `PAGER_EDGE`, `DISCLOSURE`,
`DISCLOSURE_HEAD`, `DISCLOSURE_SUMMARY`, `DISCLOSURE_MARK`,
`DISCLOSURE_TITLE`, `DISCLOSURE_DETAIL`, `DISCLOSURE_ACTIONS`,
`DISCLOSURE_BODY` (`OPEN` is new too). New partials: `_pager.scss`,
`_disclosure.scss`. New tokens: none (`--pager-chars` is set per Pager).

## Select and Listbox

`Select<T>` is the WAI-ARIA select-only combobox: a face in the tab order
(a `<button>`, `role="combobox"`, `aria-haspopup="listbox"`) showing what is
chosen, opening a listbox (`role="listbox"`, `plinth::class::LISTBOX`) under
it in the overlay layer through `use_popover`, while focus stays on the face
(`aria-activedescendant` names the active option). Props:

- `options: Vec<SelectOption<T>>` (`SelectOption::new(value, label)`,
  `.icon(html)` before its label in the list, `.disabled(..)`), `value:
  Option<T>`, `on_change` (told a pick that is not the one chosen),
  `placeholder` (muted, while nothing listed is chosen).
- `width` (`SelectWidth`): `Widest` (the widest option or placeholder) or
  `Fill` (`FILL`, its container's width).
- `size` (`InputSize`, as a `TextInput`'s), `invalid`, `disabled`, `busy`
  (`aria-busy`; it still opens on what it has).
- `id` and `describedby` inside a `Field` (its label names and opens it),
  `label` (the accessible name without one), `class` (the face's), `testid`
  (the face's; the list's is it suffixed `-options`).

In an `InputGroup`, give it `group_classes(..)[i]` as its `class` like any
member. `select(size, width, invalid, disabled, busy)` gives the face's
classes (the input box's, `SELECT`, `FILL`), `chosen(options, value)` the
chosen option's index.

Keyboard: closed, the arrows, Enter, Space, Home and End open it (on the
chosen option, else the first or, for ArrowUp and End, the last enabled
one), and typing opens it on the option starting with what was typed.
Open, the arrows (stopping at the ends), PageUp and PageDown (10), Home and
End move over the enabled options, typing moves as a `Menu`'s type-ahead
does, Enter, Space and Alt+ArrowUp pick, and Escape, Tab or a press outside
close it. A pointer over an option makes it active; a press on the list
keeps focus on the face.

`Listbox` is the list's keyboard state, for anything listing options under
what keeps focus (P12's searching pickers): `active`, `hover`, `set`,
`step`, `go_to_end`, `settle` (after filtering), `typeahead(pickable,
labels, char, now)`, `key(key, alt, pickable) -> ListKey` (`Moved`,
`Pick(i)`, `Ignored`) and `take_scroll` (scroll the active option into
view). `option_id(base, i)`, `listbox(placed)` and `listbox_option(active)`
(`LISTBOX_OPTION`, `ACTIVE`) give the ids and classes; an option's
`aria-selected` and `aria-disabled` carry the rest. Its parts are
`LISTBOX_OPTION_MARK` (the selected one's check, its box kept when off,
`OFF`), `LISTBOX_OPTION_ICON` and `LISTBOX_OPTION_LABEL`.

Size guarantees: the face is one width whatever is chosen. Every option's
label and the placeholder are in one grid cell, all but the shown one
`HIDDEN`, so `Widest` is the widest of them. Past its container it is cut
with an ellipsis, never wrapped. `Fill` is the container's width. It is
the input box's height for its size, and open, focus, hover and invalid
change colours only. The face shows no option icon, which would change its
width. The list is the menu's surface (fixed in a layer that lays nothing
out, hidden until placed), at least as wide as the face
(`--listbox-anchor-width`) and never narrower than it opened
(`--listbox-open-width`). Its rows are `--control-height` at least, their
border reserved, and the active row changes only its fill. Labels are one
line. New tokens: none (the two custom properties are set by the
component). New partials: `_listbox.scss`, `_select.scss`. The menu's
surface, row and mark are now the mixins `list`, `item`, `item-active` and
`mark` in `_menu.scss`, with the same output.

## DropZone

A target for files dropped on it, pasted, or picked with its button. While a
file is dragged over the page the zone is outlined, over it tinted; a file of
a type it does not take is refused. All of that, and the busy and error
messages, is drawn by an overlay inside the zone, so the zone never changes
size.

```rust
html! {
    <DropZone accept="image/*" max_size={Some(5_000_000)} paste=true
        pick="Choose a file" {on_files} {on_refused}>
        <p>{ "Drop images here" }</p>
    </DropZone>
}
```

Props: `accept` (as a file input's: media types, `image/*`, extensions; empty
takes anything), `max_size` (bytes), `multiple` (otherwise the first file
only), `label`, `reject` and `too_large` (what the overlay says), `busy`
(`Some(text)` while a file is worked on; the zone takes nothing then),
`error` (text, until the app clears it), `disabled`, `paste` (files pasted
outside a text field go the same way), `pick` (a button with this label
opens the file picker through a hidden input), `on_files` (the files taken,
in order, from a drop, paste or pick), `on_refused` (a `Refusal`: `Type` or
`TooLarge`), `testid`, `class`. `accepts(accept, name, kind)` and
`refusal(accept, max_size, name, kind, size)` are the same filter as plain
functions. A refusal the zone found itself is said until the next press in
it or the next drag; a drag that carries no files passes through.

Modifiers, set from `DropState` by `drop_zone(state, disabled)`:
`.plinth-dragging` (a file is dragged on the page), `.plinth-over`,
`.plinth-refused`, `.plinth-busy`, `.plinth-error` and `.plinth-disabled`.
Parts: `.plinth-drop-zone-overlay`, `-label` and `-pick`.

Size guarantees: the zone has a border at all times (`reserved-border`,
transparent at rest), and every state changes only colours, so its box is the
same at rest, dragged over, refused, busy and in error, with a short message
or a long one (the label is cut at two lines inside the overlay, which
clips). The overlay is absolutely placed and lets the drag through. The
zone's least height is `--drop-min-height`.

Tokens: `--drop-min-height`, `--drop-tint` and `--drop-tint-over`.

## ColorSelect

`ColorSelect` picks a colour from a palette the app passes in as data; plinth
owns no palette. It is the select-only combobox pattern of `Select`: a face
(a `<button>`, `role="combobox"`) with the chosen colour's swatch and name,
opening a grid (`role="listbox"`) of swatches in the overlay layer through
`use_popover`, while focus stays on the face. Props:

- `options: Vec<ColorOption>` (`ColorOption::new(value, label)`; the value is
  any CSS colour, and an empty one is "no colour", drawn struck through),
  `value: Option<AttrValue>`, `on_change` (told a pick that is not the one
  chosen), `placeholder` (muted, while `value` is `None`).
- `custom: Option<AttrValue>`: the label of a row under the grid holding the
  browser's native colour input (`#rrggbb`); a colour picked there is told
  to `on_change` like any other, and a value that is not in the palette is
  shown by its own text.
- `columns` (6): swatches to a grid row (`--color-select-columns`).
- `width` (`ColorSelectWidth`): `Fixed` (`--color-select-width`), `Fill`
  (`FILL`, the container's) or `Swatch` (`SWATCH_ONLY`, the swatch alone in a
  box as wide as it is high).
- `size` (`InputSize`), `invalid`, `disabled`, `busy` (`aria-busy`; it still
  opens on what it has), `id` and `describedby` inside a `Field`, `label` (the
  accessible name without one), `class` (the face's), `testid` (the face's;
  the grid's is it suffixed `-options`, the custom input's `-custom`).

In an `InputGroup`, give it `group_classes(..)[i]` as its `class`.
`color_select(size, width, invalid, disabled, busy)`, `color_swatch(none)`,
`color_list(placed)` and `color_option(active)` give the classes
(`COLOR_SELECT`, `COLOR_SWATCH` with `NONE`, `COLOR_LIST` with `PLACED`,
`COLOR_OPTION` with `ACTIVE`; parts `COLOR_SELECT_LABEL`,
`COLOR_SELECT_PLACEHOLDER`, `COLOR_SELECT_CHEVRON`, `COLOR_GRID`,
`COLOR_OPTION_MARK`, `COLOR_CUSTOM` with `_LABEL` and `_INPUT`);
`color_index(options, value)`, `color_name(options, value)` and
`color_input_value(value)` give the chosen option, its name and the native
input's value.

Keyboard: closed, the arrows, Enter, Space, Home and End open it on the
chosen colour (else the first or last), and typing opens it on the colour
whose name starts with what was typed. Open, Left and Right move by one,
Up and Down by a row (stopping at the ends), Home, End, PageUp and PageDown
and type-ahead move as in a `Listbox`, Enter, Space and Alt+ArrowUp pick,
Escape or a press outside close it, and Tab moves into the custom input
(with `custom`; Shift+Tab or Tab from it closes and returns to the face).

Size guarantees: the face is one width whatever is chosen or loaded, the
label cut with an ellipsis, never wrapped; it is the input box's height for
its size, and open, focus, hover and invalid change colours only. A swatch
is `--swatch-size` square, a grid cell `--control-height` square with its
border reserved, and the active and selected cells change fill and border
colour only; the selected mark keeps its box when off. The popup is the
menu's surface in a layer that lays nothing out, hidden until placed.
New tokens: `--swatch-size`, `--color-select-width`. New partial:
`_color-select.scss`; new classes as above (`SWATCH_ONLY` is new; `NONE`,
`FILL`, `PLACED`, `ACTIVE` and `OFF` are shared).
## SearchSelect and FilterPicker

`SearchSelect<T>` is the WAI-ARIA editable combobox with list
autocomplete: a field (the input box, `SEARCH_SELECT`) whose text filters a
listbox under it in the overlay layer, focus staying in the field. Props:

- `options: Vec<SearchOption<T>>` (`SearchOption::new(value, label)`,
  `.detail(..)` muted after the label and searched too, `.icon(html)`,
  `.group(..)` the heading it is listed under, adjacent options sharing
  one, `.disabled(..)`), `value: Option<T>` (shown in the field while it is
  not being edited), `on_change`.
- `filter` (on by default: `search_matches(options, query)`, label or
  detail containing the query, ignoring case), `on_search` (each edit, for
  options loaded elsewhere; turn `filter` off), `answers` (the query the
  options answer: until it is the one typed they are stale, and a loading
  row stands in their place, so a late answer to an older query is never
  shown), `busy` (a spinner in the field, `aria-busy`, and the loading row
  while there are no options).
- `label` (the placeholder and accessible name), `loading_label` and
  `empty_label` (the app's words), `id`, `describedby`, `size`, `fill`
  (`FILL`), `invalid`, `disabled`, `autofocus`, `class`, `testid` (the
  list's is it suffixed `-options`).

`listed(options, query, answers, busy, filter)` says what an open list shows
(`Listed::Loading` or `Listed::Options(indices)`), `search_select(size,
fill, invalid, disabled, busy)` gives the field's classes. Keyboard:
ArrowDown and ArrowUp open it; open, the arrows, PageUp and PageDown move
over the enabled options (Home, End and Space edit the text), Enter picks,
Escape, Tab or a press outside close it and give the field back what is
chosen. Typing makes the first match active. Parts:
`SEARCH_SELECT_LIST`, `SEARCH_SELECT_GROUP` with `SEARCH_SELECT_HEADING`,
`SEARCH_SELECT_DETAIL`, `SEARCH_SELECT_NOTE` (the loading or no-match row).

`FilterPicker<T>` is a list's filter, for a `LibraryToolbar`: a button
(`BUTTON`, `FILTER_PICKER`, `ON` while anything is chosen) saying `all`, or
the first choice and `+N` more, opening a multi-select listbox
(`aria-multiselectable`): an option for all, then one per choice, checked
when chosen. A pick leaves it open. Props: `choices: Vec<FilterChoice<T>>`,
`chosen: Vec<T>` (in the order chosen), `on_change` (the choices after a
pick, `toggled(chosen, value)`; empty for all), `label` (the accessible
name and tooltip before what is chosen), `all` (the app's word, `"All"` by
default), `disabled`, `class`, `testid`. Keys as a `Select`'s. Past
`SEARCH_FROM` (8) choices the pop-up is a `ChoiceList` (`multi`, all as
its first option) with its search field, prefix matches first and its
`--picker-rows` height whatever matches; the field takes the focus.
`filter_summary(all, labels)` gives its words and count. Parts:
`FILTER_PICKER_TEXT`, `FILTER_PICKER_COUNT` (`OFF` with none),
`FILTER_PICKER_CHEVRON`, `FILTER_PICKER_LIST`.

Size guarantees: SearchSelect's field is `--menu-width-wide` (or its
container's, `Fill`) and the input box's height, whatever is typed or
chosen. Its list is `--menu-width-wide` or the field's width if wider,
never measured from its text, and `--picker-rows` rows high (or what fits
below the field), so typing, results arriving, loading and no matches
never resize it; past that it scrolls. FilterPicker is `--filter-width`
wide from no choice to all of them: its words are cut with an ellipsis and
its count's room (`3ch`) is kept, hidden, with none. New token:
`--picker-rows` (`$picker-rows: 6`). New partials: `_search-select.scss`,
`_filter-picker.scss`.

## Picker, PickerMenu, ChoiceList and PickerDialog

`ChoiceList<T>` is the list every picker opens: a listbox (`CHOICE_LIST`,
`CHOICE_LIST_OPTIONS`) of `PickerOption<T>`s under a search field
(`CHOICE_LIST_SEARCH`, an input box) once there are more than
`SEARCH_FROM` (8) options, or with `always_search`. Focus stays in the
field (or on the list, without one), naming the active option with
`aria-activedescendant`: the arrows, PageUp and PageDown move over the
enabled options and Enter picks; without a field, Home, End, Space and
type-ahead too. Props: `options`, `value` (checked), `multi`
(`aria-multiselectable`, `chosen` checked, the owner toggles a pick with
`toggled`), `on_pick`, `label` (the list's name), `always_search`,
`search_label` and `empty_label` (the app's words), `autofocus`, `class`,
`testid` (the field's is it suffixed `-search`, each option's `-option`).

`PickerOption::new(value, label)` with `.detail(..)` (muted after the
label, searched), `.icon(html)` (a slot kept on every row once any option
has one), `.badge(html)` (at the row's end), `.keywords(..)` (searched, not
shown) and `.disabled(..)`. Rows are the listbox's (`LISTBOX_OPTION` with
its `-mark`, `-icon`, `-label`), plus `CHOICE_LIST_DETAIL`,
`CHOICE_LIST_BADGE`, and `CHOICE_LIST_NOTE` for no matches.
`picker_matches(options, query)` finds every word in the label, detail or
keywords, ignoring case, labels starting with the text first, then labels
with a word starting with it, then the rest.

`Picker<T>` is a field (the input box, `PICKER`) showing the current
option's icon and label (`PICKER_ICON`, `PICKER_TEXT`), or the placeholder
(`PICKER_PLACEHOLDER`), and a chevron (`PICKER_CHEVRON`), opening a
`ChoiceList` below it in the overlay layer (`PICKER_POPUP`, the menu's
surface). A pick, Escape, Tab or a press outside closes it; ArrowDown or
ArrowUp opens it. Props: `options`, `value`, `on_change`, `label` (the
accessible name, followed by the current option's label), `placeholder`,
`variant` (`PickerVariant::Field` or `Small`, `SMALL`), `fill` (`FILL`),
`invalid`, `disabled`, `always_search`, `search_label`, `empty_label`,
`id`, `describedby`, `class`, `testid` (the popup's is it suffixed
`-popup`, the list's `-list`). `picker(variant, fill, invalid, disabled)`
gives its classes. `PickerMenu<T>` is the same list opened from a `Button`
for an action that takes an option (Add a language): `text`, `icon`,
`label`, `variant`, `size`, `disabled`, `on_pick`. `PickerDialog<T>` is the
searchable shell for a large set (languages, countries): a `Dialog`
(`open`, `on_dismiss`, `title`, `size`, `actions` such as Done) holding a
`ChoiceList` that always searches, its field focused, single (the owner
closes it on a pick) or `multi`.

Size guarantees: the Picker's field is `--menu-width-wide` wide (small:
`--menu-width`; its container's with `Fill`) and the input box's height,
whatever is chosen; its label is cut with an ellipsis. The popup is
`--menu-width-wide` or the field's width if wider, never measured from its
text. A ChoiceList's options are `--picker-rows` rows high whatever
matches, none included, so typing never resizes the list, its popup or its
dialog; past that it scrolls, its scrollbar's room kept. Marks keep their
box when off. New partial: `_picker.scss`. No new tokens.

## ToolBand, Panel, Seam and use_width

**ToolBand** is a `Toolbar` with `band`: its controls wrap onto further
lines instead of scrolling sideways (`.plinth-toolbar.plinth-band`; no
hidden scrollbar or fade). `chrome` sets it on `--surface` with a rule
under it (`.plinth-chrome`, an inset shadow, so no border is added);
`search` sets it in `--text-sm`, as a list's filter (`.plinth-search`).
Group its controls in `ToolbarPart`s (`.plinth-toolbar-part`, `role=group`):
a band wraps between parts, and inside a part only when that part alone is
wider than the band (it never overflows the band).
`ruled` rules a part off from the one before it (`.plinth-ruled`, a
`--border` line and `--space-3` in) from the phone breakpoint up, where a
band has the room. A band's height changes only when it wraps, so give its
controls fixed widths (a `Button`'s busy and disabled states keep its box);
`toolbar_part(ruled)` gives a part's classes.

**Panel** is a ground over a list or a view holding its filters or a short
form (`.plinth-panel-box`): a column `--space-2` apart, on `--panel-ground`
in a `--border` ring that is always there, as wide as what it holds and
never wider than its column. `line` lays it out as one row that wraps
(`.plinth-line`); `ground` is `Sunk`, `Tint` (a rounded `--panel-tint`
chip, `.plinth-tint`) or `Bare` (only the spacing, `.plinth-bare`); the
ring turns transparent rather than going, so the ground never changes its
box. `centered` centres it in its column (`.plinth-center`). With a `label`
it is a named `group`. `panel(ground, line, centered)` gives its classes.

**Seam** is two panes, `start` and `end`, and the divider dragged between
them (`.plinth-seam`, `.plinth-seam-pane`, `.plinth-seam-grip`). `at` is the
first pane's share of the room beside the divider (0 to 1); the owner keeps
it and sets it from `on_change`, which a drag calls at most once an
animation frame, with the pointer captured, and `on_settle` once the drag
ends. The divider is a `separator`: the arrow keys move it by 5%, Home and
End to the ends, and a double-click returns it to `reset` (0.5). `min` and
`max` bound the share, and each pane keeps `--seam-min` (8rem) besides,
both in what it reports and in the grid, so neither pane collapses and an
`at` out of range still lays out. The divider is `--seam-grip` wide
whatever its state: hover, focus and a drag (`.plinth-active` on the seam)
light its line only. `axis: SeamAxis::Rows` stacks the panes
(`.plinth-rows`); give the seam a height. Each pane scrolls what overflows
it, so its content never widens it.

**`use_width(&node)`** is the width of `node`'s element (its border box, in
CSS pixels) from a `ResizeObserver`: `0.0` until it is first measured, then
at most one new value an animation frame, and only when it changed, so a
component laid out against its own width renders once a frame of a resize.
The observer goes with the component.

## Against jitter: mixins and utilities

The shared rules every component builds on, so nothing changes size as its
state or content changes. Each is a mixin (`@use "plinth" as p;` then
`@include p.numeric;`) and, where an element needs it directly, a utility
class. Utilities load after the components, so one wins over a component's
rule of the same weight; none uses `!important`.

| Mixin | Utility (`plinth::class::`) | What it guarantees |
| --- | --- | --- |
| `focus-ring` | `.plinth-focus-ring` (`FOCUS_RING`) | A `:focus-visible` outline from the `--focus-ring-*` tokens: never a border, so focus adds nothing to the box. |
| `reserved-border($color: transparent, $width: var(--space-px))` | | The border is always there; a state changes only its colour. |
| `numeric` | `.plinth-numeric` (`NUMERIC`) | Tabular figures: a count, readout, pager or clock keeps its width as it changes. |
| `scroll-area` | `.plinth-scroll-area` (`SCROLL_AREA`) | Scrolls, with `scrollbar-gutter: stable`: content does not jump when it starts to scroll. Contains overscroll and may shrink in a flex column. |
| | `.plinth-toolbar` | Scrolls sideways with its scrollbar hidden, so its height does not change when its controls start to overflow; the edge with hidden controls fades (a CSS scroll-driven animation: browsers without `animation-timeline` show no fade). Wheel, touch and keyboard scroll as before. |
| `truncate` | `.plinth-truncate` (`TRUNCATE`) | One line, cut with an ellipsis. |
| `clamp($lines)` | `.plinth-clamp-2`, `-3`, `-4` (`CLAMP_2`...) | At most that many lines, the last cut with an ellipsis. |
| `visually-hidden` | `.plinth-visually-hidden` (`VISUALLY_HIDDEN`) | Read by a screen reader, takes no room (a hidden h1). |
| `phone-only`, `desktop-only` | `.plinth-phone-only`, `.plinth-desktop-only` | Shown only below `$visibility-breakpoint`, or only from it up. |
| `current($by: background)` | | Marks `[aria-current]` (not `"false"`) and `[aria-selected="true"]` by `background` (`--accent` under `--on-accent`), `color` or `underline`; never by weight, so the item keeps its width. |
| `scroll-fade` | | Scrolls sideways with the scrollbar hidden and the edge that has hidden content faded (`Toolbar`, `Tabs`). |

### Recipes for markup plinth does not render

A dependency's markup (a sign-in form, a user table) cannot wear plinth's
classes. The mixins below are the rules of the classes they name, which
include them, so such markup styled with them looks and behaves as
plinth's own: `.auth-card { @include p.card; }`.

| Mixin | The rule of |
| --- | --- |
| `stack($gap: var(--space-3))` | `.plinth-stack` (a gap of the scale for its modifiers) |
| `cluster($gap: var(--space-2), $wrap: true)` | `.plinth-cluster` (`$wrap: false` for `.plinth-nowrap`) |
| `card` | `.plinth-card` |
| `field`, `field-label` | `.plinth-field`, `.plinth-field-label` |
| `input`, `input-box` | `.plinth-input`; `input-box` is the box every text field shares (border, fill, sizes) |
| `button($variant: secondary)` | `.plinth-button` with its states; `button-variant($variant)` is a variant's colours alone (`primary`, `ghost`, `quiet`, `danger`, `success`) |
| `table-base`, `table-head-cell`, `table-cell`, `table-row-hover` | a table's (see Table) |

A button styled this way lays out its own content (plinth's puts it in
`.plinth-button-content`): give the markup a gap of its own if it holds an
icon and a label.

Layers: `--layer-sticky`, `--layer-overlay`, `--layer-toast` and
`--layer-tooltip` are the one `z-index` scale for what floats, lowest first.
Popovers, menus, lists and dialogs all sit at `--layer-overlay`.

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

Under its frames, a page shows the code that renders its component as
the controls set it, highlighted, with a Copy button. A page builds it
with `showcase/src/code.rs` (`El` for each element, its props and
children, and `snippet` around them), listing only the props that differ
from their defaults, and passes it to `Demo` as `code`. A page without a
`Demo` shows a `CodeCard` itself; the test fails for a page that shows
no code.

### Stability probe

Each page also declares the boxes that must hold (`holds`, next to its
`covers`): a part's class, which of its size holds (`Size::Width`,
`Height`, `Both`, or `Box` for its size and place), and across which
states (`Across::All`, `Only` or `Except` a list of control labels, a
single value as `"label=value"`, or the probe's own `focus` and
`click`). The test fails for a page without one, unless it is listed in
`UNPROBED`.

The panel in the window's corner (fixed, so it never moves the page)
runs the probe: it sets each control to each of its values in turn,
one control at a time, toggles each checkbox, then focuses each control
in the frames and clicks each button there (pressing Escape after each
click, so nothing it opened stays open), measuring every declared
part in both frames after each state. Its `<pre>` reads `probe: stable`
when no declared box moved or resized by more than half a pixel, and
otherwise lists each change by state, frame and part. The panel's
`data-probe` attribute carries the verdict (`idle`, `running`, `stable`,
`unstable`), and `?probe` before the page's `#id` runs it on load, so a
browser check reads the result as text. Hover cannot be scripted, so it
stays a manual check.

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
