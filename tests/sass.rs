use std::path::{Path, PathBuf};
use std::process::Command;

const SASS: &str = ".cache/trunk/sass-1.104.1/sass";

fn sass() -> Option<PathBuf> {
    let path = Path::new(&std::env::var_os("HOME")?).join(SASS);
    if path.exists() {
        Some(path)
    } else {
        eprintln!("skipping: no sass at {}", path.display());
        None
    }
}

fn compile(sass: &Path, file: &str) -> String {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join(file);
    let output = Command::new(sass)
        .arg("--no-source-map")
        .arg(&file)
        .output()
        .expect("run sass");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("utf-8 css")
}

fn fails(sass: &Path, file: &str) -> String {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join(file);
    let output = Command::new(sass)
        .arg("--no-source-map")
        .arg(&file)
        .output()
        .expect("run sass");
    assert!(!output.status.success(), "{file:?} compiled");
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn defaults_compile() {
    let Some(sass) = sass() else { return };
    let css = compile(&sass, "style/_index.scss");
    assert!(css.contains("--text-md: 0.875rem;"));
    assert!(css.contains("--radius-lg: 16px;"));
    assert!(css.contains("@media (width < 40rem)"));
}

#[test]
fn configured_through_index() {
    let Some(sass) = sass() else { return };
    let css = compile(&sass, "tests/fixtures/configured.scss");
    assert!(css.contains("--text-md: 1.0625rem;"));
    assert!(css.contains("--radius-lg: 12px;"));
    assert!(css.contains("padding: var(--space-4) var(--space-5);"));
    assert!(
        css.contains("@media (width < 42rem) {\n  .plinth-card {\n    padding: var(--space-3);")
    );
    assert!(css.contains("gap: var(--space-2);\n  min-width: 0;\n  overflow-x: auto;"));
    assert!(!css.contains("@media (width < 40rem)"));
}

#[test]
fn configured_through_tokens() {
    let Some(sass) = sass() else { return };
    let css = compile(&sass, "tests/fixtures/configured_tokens.scss");
    assert!(css.contains("--text-md: 1.0625rem;"));
    assert!(css.contains("--radius-lg: 12px;"));
}

#[test]
fn dialog_settings() {
    let Some(sass) = sass() else { return };
    let defaults = compile(&sass, "style/_index.scss");
    assert!(defaults.contains("--dialog-room: calc(100dvh - var(--dialog-top) - var(--space-4));"));
    let css = compile(&sass, "tests/fixtures/dialog_centred.scss");
    assert!(css.contains(".plinth-dialog-layer {\n  align-items: center;"));
    assert!(css.contains("--dialog-room: min(85vh, 100dvh - 2 * var(--space-4));"));
    assert!(
        css.contains(
            "padding: 1rem var(--dialog-pad) var(--dialog-pad);\n  scrollbar-gutter: auto;"
        )
    );
    assert!(css.contains("--section-title-size: 1.125rem;"));
    assert!(
        css.contains("border-bottom-color: transparent;\n    border-width: var(--space-px) 0 0;")
    );
    assert!(!defaults.contains("border-width: var(--space-px) 0 0;"));
}

#[test]
fn popover_and_confirm_settings() {
    let Some(sass) = sass() else { return };
    let defaults = compile(&sass, "style/_index.scss");
    assert!(
        defaults.contains("border-radius: var(--radius-md);\n  background: var(--surface-raised);")
    );
    assert!(defaults.contains("flex-direction: column;\n  gap: var(--space-3);\n  min-width: 0;"));
    let css = compile(&sass, "tests/fixtures/popover_settings.scss");
    assert!(css.contains("border-radius: 12px;\n  background: var(--surface-raised);"));
    assert!(css.contains("gap: 1rem;"));
    assert!(css.contains("scrollbar-gutter: stable;\n  scrollbar-gutter: auto;"));
    assert!(!defaults.contains("scrollbar-gutter: auto;\n  flex: 1 1 auto;"));
    assert!(
        css.contains("border-bottom-color: transparent;\n    border-width: var(--space-px) 0 0;")
    );
    assert!(css.contains(
        ".plinth-confirm-popover.plinth-compact {\n    --popover-max-width: 100vw;\n    width: max-content;"
    ));
}

#[test]
fn error_box_settings() {
    let Some(sass) = sass() else { return };
    let centred = "padding-block: calc((var(--control-height) - 1lh) / 2);";
    let defaults = compile(&sass, "style/_index.scss");
    assert!(defaults.contains(centred));
    let css = compile(&sass, "tests/fixtures/error_box_settings.scss");
    assert!(css.contains("padding-block: calc((var(--control-height) - 1lh) / 2) 0;"));
    assert!(!css.contains(centred));
}

#[test]
fn drag_handle_settings() {
    let Some(sass) = sass() else { return };
    let defaults = compile(&sass, "style/_index.scss");
    assert!(
        defaults.contains(
            "width: var(--control-height-small);\n  height: var(--control-height-small);"
        )
    );
    assert!(defaults.contains("border-radius: var(--radius-xs);"));
    let css = compile(&sass, "tests/fixtures/drag_handle_settings.scss");
    assert!(css.contains("width: 32px;\n  height: 32px;"));
    assert!(css.contains("border-radius: 6px;"));
    assert!(css.contains("background: rgba(1, 2, 3, 0.08);"));
    assert!(css.contains("width: 44px;\n    height: 44px;"));
    assert!(!css.contains("width: var(--control-height-small)"));
}

#[test]
fn a_dialog_body_gap_spaces_its_paragraphs() {
    let Some(sass) = sass() else { return };
    let defaults = compile(&sass, "style/_index.scss");
    assert!(!defaults.contains(".plinth-dialog-body > p"));
    let css = compile(&sass, "tests/fixtures/dialog_body_gap.scss");
    assert!(css.contains("gap: 1rem;"));
    assert!(css.contains(".plinth-dialog-body > p {\n  margin: 0;\n}"));
}

#[test]
fn a_native_select_gives_way_in_a_row() {
    let Some(sass) = sass() else { return };
    let defaults = compile(&sass, "style/_index.scss");
    assert!(defaults.contains(
        ".plinth-native-select {\n  width: max-content;\n  max-width: 100%;\n  flex-shrink: 1;"
    ));
}

#[test]
fn a_picker_gives_way_in_a_row() {
    let Some(sass) = sass() else { return };
    let defaults = compile(&sass, "style/_index.scss");
    assert!(defaults.contains(
        ".plinth-picker {\n  width: var(--menu-width-wide);\n  max-width: 100%;\n  flex-shrink: 1;"
    ));
}

#[test]
fn input_settings() {
    let Some(sass) = sass() else { return };
    let defaults = compile(&sass, "style/_index.scss");
    assert!(defaults.contains("--input-edge: var(--accent);"));
    assert!(!defaults.contains("--input-height: 44px;"));
    assert!(defaults.contains(".plinth-native-select {\n  width: max-content;"));
    let css = compile(&sass, "tests/fixtures/input_settings.scss");
    assert!(css.contains("--input-edge: var(--border-strong);"));
    assert!(css.contains(":focus-within {\n  --input-edge: var(--accent);"));
    assert!(css.contains("--input-height: 44px;"));
    assert!(css.contains("padding-inline: 10px;"));
    assert!(css.contains("font-size: var(--text-sm);"));
    assert!(css.contains("outline-offset: -1px;"));
}

#[test]
fn omitted_parts_are_left_out() {
    let Some(sass) = sass() else { return };
    let defaults = compile(&sass, "style/_index.scss");
    let css = compile(&sass, "tests/fixtures/omit.scss");
    for part in [
        ".plinth-toolbar",
        "--band-chrome-ground",
        ".plinth-cluster.plinth-wide",
        ".plinth-button.plinth-fold {",
        ".plinth-head-trail",
        ".plinth-split-button",
        "--table-max-height",
        "--danger:",
        ".plinth-fieldset",
        ".plinth-field-error",
        ".plinth-input.plinth-small",
        ".plinth-input-box.plinth-touch",
        ".plinth-input.plinth-busy",
        ".plinth-input[aria-invalid=true]",
        ".plinth-input-box.plinth-invalid",
        ".plinth-input-box.plinth-disabled",
        ".plinth-menu-item[aria-disabled=true]",
        ".plinth-menu-item-mark",
        ".plinth-pager.plinth-busy",
    ] {
        assert!(defaults.contains(part), "{part} in the defaults");
        assert!(!css.contains(part), "{part} left out");
    }
    for kept in [
        ".plinth-stack.plinth-loose,\n.plinth-cluster.plinth-loose {",
        ".plinth-button.plinth-fold-wide",
        ".plinth-head-subtitle {\n  flex: 0 0 100%;",
        ".plinth-menu-item {",
        "--success:",
        ".plinth-field-hint {\n  margin: 0;",
        ".plinth-field-inputs {",
        ".plinth-input:disabled",
        ".plinth-input-box:focus-within",
        ".plinth-menu-item-icon {",
        ".plinth-pager-button {",
    ] {
        assert!(css.contains(kept), "{kept} kept");
    }
}

#[test]
fn an_unknown_part_is_an_error() {
    let Some(sass) = sass() else { return };
    assert!(fails(&sass, "tests/fixtures/omit_unknown.scss").contains("no part layout.toolbars"));
}
