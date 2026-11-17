//! The roving tabindex a group of options shares (`Choice`, `Tabs`): one tab
//! stop, and the keys that move it. Options are told apart by whether each is
//! disabled.

/// The option holding the tab stop: the selected one if enabled, else the
/// first that is enabled.
pub fn stop(disabled: &[bool], selected: Option<usize>) -> Option<usize> {
    selected
        .filter(|i| !disabled[*i])
        .or_else(|| disabled.iter().position(|d| !d))
}

/// The enabled option a key moves to from `from`: the arrows step (and
/// wrap), Home and End jump; `None` for any other key.
pub fn step(disabled: &[bool], from: usize, key: &str) -> Option<usize> {
    let len = disabled.len();
    let enabled = |i: &usize| !disabled[*i];
    let walk = |forward: bool| {
        (1..len)
            .map(|n| {
                if forward {
                    (from + n) % len
                } else {
                    (from + len - n) % len
                }
            })
            .find(enabled)
    };
    match key {
        "ArrowRight" | "ArrowDown" => walk(true),
        "ArrowLeft" | "ArrowUp" => walk(false),
        "Home" => (0..len).find(enabled),
        "End" => (0..len).rev().find(enabled),
        _ => None,
    }
}
