use plinth::STYLESHEET;

#[test]
fn embeds_the_rules() {
    assert!(STYLESHEET.contains(":root {"));
    assert!(STYLESHEET.contains("--text-md: 0.875rem;"));
    assert!(STYLESHEET.contains("--radius-lg: 16px;"));
    assert!(STYLESHEET.contains(".plinth-stack {"));
    assert!(STYLESHEET.contains(".plinth-card {"));
    assert!(STYLESHEET.contains("@media (width < 40rem)"));
}

#[test]
fn a_glyph_box_has_a_fixed_size() {
    let rule = STYLESHEET
        .split(".plinth-glyph {")
        .nth(1)
        .and_then(|r| r.split('}').next())
        .expect("a .plinth-glyph rule");
    assert!(rule.contains("width: var(--glyph-size);"), "{rule}");
    assert!(rule.contains("height: var(--glyph-size);"), "{rule}");
    assert!(rule.contains("overflow: hidden;"), "{rule}");
    assert!(STYLESHEET.contains("--glyph-size: 1em;"));
}
