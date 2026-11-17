use plinth::STYLESHEET;

#[test]
fn embeds_the_rules() {
    assert!(STYLESHEET.contains(":root {"));
    assert!(STYLESHEET.contains("--text-md: 0.875rem;"));
    assert!(STYLESHEET.contains("--radius-lg: 16px;"));
    assert!(STYLESHEET.contains(".stack {"));
    assert!(STYLESHEET.contains(".card {"));
    assert!(STYLESHEET.contains("@media (width < 40rem)"));
}
