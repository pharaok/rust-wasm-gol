use crate::{components::Link, parse::get_index};
use js_sys::RegExp;
use leptos::prelude::*;

thread_local! {
    static CONWAY_LIFE_LINK_RE: RegExp = RegExp::new(r"\b(?:https?://)?((?:www\.)?conwaylife\.com(\S*))\b", "");
}

#[component]
pub fn Text(text: String) -> impl IntoView {
    view! {
        <p class="whitespace-pre-line">
            {move || {
                let mut prev = 0;
                let mut nodes = Vec::new();
                while let Some(captures) = CONWAY_LIFE_LINK_RE.with(|re| re.exec(&text[prev..])) {
                    let index = prev + get_index(&captures);
                    let prev_text = text[prev..index].to_owned();
                    let inner_text = captures.get(2).as_string().unwrap();
                    prev = index + captures.get(0).as_string().unwrap().len();
                    nodes
                        .push(
                            view! {
                                {prev_text}
                                <Link
                                    href=format!("https://{}", captures.get(1).as_string().unwrap())
                                    attr:target="_blank"
                                >
                                    <img
                                        src="/conwaylife.ico"
                                        alt="ConwayLife.com"
                                        class="inline"
                                    />
                                    {inner_text}
                                </Link>
                            }
                                .into_any(),
                        );
                }
                nodes.push(text[prev..].to_string().into_any());
                nodes
            }}

        </p>
    }
}
