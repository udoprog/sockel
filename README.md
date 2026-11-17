# plinth

Opinionated layout primitives and components for [Yew] apps, built so
components keep their size and don't jitter as their content changes.

Each component comes with its Sass under `style/`, and its variants are
modifier classes set by props. An app uses them like this:

```toml
[dependencies]
plinth = { path = "../plinth" }
```

```scss
@use "../plinth/style";          // every primitive's rules
@use "../plinth/style/tokens";   // tokens and mixins
```

The app supplies the colour tokens: `--bg`, `--surface`, `--text`, `--muted`,
`--border`, `--accent` and `--shadow-1`.

## Checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo clippy --target wasm32-unknown-unknown -- -D warnings
cargo test
```

[Yew]: https://yew.rs
