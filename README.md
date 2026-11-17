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

## Theming

The app supplies seven colour tokens: `--bg`, `--surface`, `--text`,
`--muted`, `--border`, `--accent` and `--shadow-1`. plinth derives six more
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
| `$check-size`, `$switch-width`, `$switch-height`, `$choice-width` | `1.25rem`, `2.5rem`, `1.5rem`, `6rem` |
| `$filter-width` | `10.5rem` |
| `$field-label-width`, `$field-message-line`, `$field-error-lines` | `12rem`, `1rem`, `1` |
| `$avatar-sm`, `$avatar-md`, `$avatar-lg`, `$avatar-xl` | `1.75rem`, `2.5rem`, `3.5rem`, `7rem` |
| `$page-workspace`, `$page-reading`, `$page-wide`, `$page-gutter` | `64rem`, `40rem`, `90rem`, `var(--space-8)` |
| `$card-padding` | `var(--space-4)` |
| `$card-padding-phone` | `null` (a card keeps `$card-padding` on a phone) |
| `$toolbar-gap` | `var(--space-1)` |
| `$focus-ring-color`, `$focus-ring-width`, `$focus-ring-offset` | `var(--accent)`, `2px`, `2px` |
| `$layer-sticky`, `$layer-overlay`, `$layer-dialog`, `$layer-toast`, `$layer-tooltip` | `10`, `100`, `200`, `300`, `400` |
| `$danger`, `$success`, `$warning`, `$on-accent`, `$surface-raised`, `$scrim` | see [Theming](#theming) |
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

## Heads

`PageHead` and `SectionHead` take one set of props (`HeadProps`):

- `title` is Html; `icon` goes before its text in the heading, `subtitle` is a
  muted line under the row, `crumbs` (a list of links) stack above it, and
  `back` is the inline "back / title" crumb. Each is its own prop.
- `help` follows the heading as its sibling, in `.plinth-head-heading` with
  the `suffix`, so the heading's text and test id exclude the help button.
- `SectionHead` takes the modifiers `plinth::class::LABEL` (small-caps label),
  `LIST` and `LIST` + `NESTED` (smaller titles) in `class`; `HEAD_END` pushes a
  child to the row's end.

## Icons

plinth's own chrome (a dialog's close, a select's chevron, a busy spinner)
draws its icons from an `IconSet`: one `Html` per `Glyph` (`Close`,
`ChevronDown`, `ChevronUp`, `ChevronRight`, `Check`, `Plus`, `Minus`,
`Spinner`, `Grip`, `Ellipsis`, `Search`). It is the one place plinth uses a
context for something other than a runtime setting, and it carries icons
only: classes never go through it.

Without a provider plinth uses `IconSet::default()`, a built-in set of inline
svg, so it renders standalone. An app supplies its own once at its root,
mapping its icon component; a set may override only some glyphs:

```rust
let set = IconSet {
    close: html! { <MyIcon name="x" /> },
    chevron_down: html! { <MyIcon name="caret-down" /> },
    ..IconSet::default()
};

html! {
    <ContextProvider<IconSet> context={set}>
        <App/>
    </ContextProvider<IconSet>>
}
```

`<Icon glyph={Glyph::Close} class=.../>` renders a glyph; `use_icon(glyph)` and
`use_icons()` return the icon(s) for a component of your own, and `icon(class,
glyph)` gives the box's classes (`plinth::class::GLYPH`, plus `SPIN` for
`Spinner`).

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

## Popover

`Popover` is a surface beside its anchor, rendered while `open` in the
overlay layer: `overlay(html)` portals into `#plinth-overlay`
(`plinth::OVERLAY_ID`), the last child of `<body>`, so no scrolling or
clipping box around the owner cuts it, and its events still bubble through
the owner. Props:

- `anchor` (a `NodeRef` on the element it opens beside, where focus returns),
  `open`, and `on_dismiss: Callback<Dismiss>`: the owner closes it.
- `side` (`Side::Below` by default, `Above`, `Start`, `End`) and `align`
  (`PopoverAlign::Start`, `Center`, `End`).
- `sheet` (on by default, `plinth::class::SHEET`): below `$sheet-breakpoint`
  (the phone's) it is a sheet from the bottom over a `--scrim`, with
  `title` as its heading and a close button (`close_label`, "Close").
- `title` is also its accessible name (`role="dialog"`); `class`, `testid`.

Dismissal (`Dismiss`): `Outside`, a `pointerdown` outside the anchor and the
popover, keeps focus where the press put it; `Escape` and `Close` (the
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

Layers: `--layer-sticky`, `--layer-overlay`, `--layer-dialog`,
`--layer-toast` and `--layer-tooltip` are the one `z-index` scale for what
floats, lowest first.

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
in the frames and clicks each button there, measuring every declared
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
