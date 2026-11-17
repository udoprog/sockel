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
