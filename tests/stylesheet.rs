use sockel::STYLESHEET;

#[test]
fn embeds_the_rules() {
    assert!(STYLESHEET.contains(":root {"));
    assert!(STYLESHEET.contains("--text-md: 0.875rem;"));
    assert!(STYLESHEET.contains("--radius-lg: 16px;"));
    assert!(STYLESHEET.contains(".sockel-stack {"));
    assert!(STYLESHEET.contains(".sockel-card {"));
    assert!(STYLESHEET.contains("@media (width < 40rem)"));
}

#[test]
fn a_glyph_box_has_a_fixed_size() {
    let rule = STYLESHEET
        .split(".sockel-glyph {")
        .nth(1)
        .and_then(|r| r.split('}').next())
        .expect("a .sockel-glyph rule");
    assert!(rule.contains("width: var(--glyph-size);"), "{rule}");
    assert!(rule.contains("height: var(--glyph-size);"), "{rule}");
    assert!(rule.contains("overflow: hidden;"), "{rule}");
    assert!(STYLESHEET.contains("--glyph-size: 1em;"));
}

#[test]
fn a_stepper_keeps_its_width_in_a_stretching_parent() {
    let rule = STYLESHEET
        .split(".sockel-stepper {")
        .nth(1)
        .and_then(|r| r.split('}').next())
        .expect("a .sockel-stepper rule");
    assert!(rule.contains("width: fit-content;"), "{rule}");
}

#[test]
fn a_mark_keeps_its_width_in_a_stretching_parent() {
    for selector in [
        ".sockel-pill {",
        ".sockel-badge {",
        ".sockel-count {",
        ".sockel-status-label {",
    ] {
        let rule = STYLESHEET
            .split(selector)
            .nth(1)
            .and_then(|r| r.split('}').next())
            .unwrap_or_else(|| panic!("a {selector} rule"));
        assert!(rule.contains("width: fit-content;"), "{selector} {rule}");
    }
}
