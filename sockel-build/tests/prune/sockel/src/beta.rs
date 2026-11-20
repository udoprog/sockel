use crate::Alpha;

pub struct Beta;

fn view(tone: &str) -> Html {
    let tone = format!("sockel-tone-{tone}");
    html! { <div class={classes!(class::BETA, tone)}><Alpha /></div> }
}
