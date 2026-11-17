// Not compiled: sources the usage checks scan.
fn view(tone: &str) -> String {
    // class::DEAD is only named in this comment
    format!("{} usage-tone-{tone} usage-live {}", class::LIVE, class::ONCE)
}
