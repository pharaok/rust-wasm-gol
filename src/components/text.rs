use leptos::prelude::*;
use regex::Regex;

use crate::components::Link;

#[component]
pub fn Text(text: String) -> impl IntoView {
    let conway_life_link =
        Regex::new(r"\b(?:https?://)?((?:www\.)?conwaylife\.com(\S*))\b").unwrap();

    view! {
        <p class="whitespace-pre-line">
            {move || {
                let mut prev = 0;
                let mut nodes: Vec<AnyView> = conway_life_link
                    .captures_iter(&text)
                    .map(|capture| {
                        let prev_text = text[prev..capture.get(0).unwrap().start()].to_string();
                        let inner_text = capture.get(2).unwrap().as_str().to_string();
                        prev = capture.get(0).unwrap().end();
                        view! {
                            {prev_text}
                            <Link
                                href=format!("https://{}", capture.get(1).unwrap().as_str())
                                attr:target="_blank"
                            >
                                <img src="/conwaylife.ico" alt="ConwayLife.com" class="inline" />
                                {inner_text}
                            </Link>
                        }
                            .into_any()
                    })
                    .collect();
                nodes.push(text[prev..].to_string().into_any());
                nodes
            }}

        </p>
    }
}
