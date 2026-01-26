use crate::{
    app::{GolContext, use_fit_universe},
    components::{
        Button, ButtonVariant, Dialog, FileInput, Icon, IconSize, Link, LinkVariant, Popover,
        PopoverPlacement, PopoverTrigger, Surface, TextArea, use_toast,
    },
    parse::rle,
    universe::InsertMode,
};
use js_sys::wasm_bindgen::{JsCast, JsValue};
use leptos::{prelude::*, task::spawn_local};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, File, HtmlAnchorElement, Url};

pub fn download_text_file(filename: &str, content: &str) {
    let document = window().document().unwrap();
    let body = document.body().unwrap();

    let parts = js_sys::Array::of1(&JsValue::from_str(content));
    let blob = Blob::new_with_str_sequence(parts.as_ref()).expect("failed to create blob");

    let url = Url::create_object_url_with_blob(&blob).expect("failed to create object URL");

    let a = document
        .create_element("a")
        .expect("failed to create anchor")
        .dyn_into::<HtmlAnchorElement>()
        .expect("failed to cast to HtmlAnchorElement");

    a.set_href(&url);
    a.set_download(filename);
    body.append_child(&a).unwrap();
    a.click();
    body.remove_child(&a).unwrap();
    Url::revoke_object_url(&url).unwrap();
}

#[component]
pub fn ImportForm(#[prop(into)] close: Callback<()>) -> impl IntoView {
    let GolContext { universe, .. } = use_context::<GolContext>().unwrap();
    let logging = use_toast();
    let (rle, set_rle) = signal(String::new());
    let on_file_change = move |file: File| {
        spawn_local(async move {
            let promise = file.text();

            match JsFuture::from(promise).await {
                Ok(text) => {
                    let content = text.as_string().unwrap_or_default();
                    set_rle.set(content);
                }
                Err(err) => logging.error(&err.as_string().unwrap_or_default()),
            }
        });
    };
    let (error_text, set_error_text) = signal("".to_owned());

    view! {
        <form on:submit=move |ev| {
            ev.prevent_default();
            if let Ok(points) = rle::iter_alive(&rle.get()).map(|it| it.collect::<Vec<_>>()) {
                universe
                    .update(|u| {
                        let half = 1i64 << (u.level() - 1);
                        u.set_points(&points, -half, -half, half - 1, half - 1, &InsertMode::Copy);
                    });
                use_fit_universe();
                close.run(());
            } else {
                set_error_text.set("Invalid format".to_owned());
            }
        }>
            <div class="flex flex-col gap-2">
                <h2 class="text-lg font-bold text-center">IMPORT PATTERN</h2>
                <div class="border-t border-neutral-800 w-full" />
                <p>Supports RLE file format.</p>
                <div class="flex flex-col gap-2">
                    <TextArea
                        class="w-full text-sm resize-none"
                        attr:spellcheck="false"
                        attr:name="rle"
                        attr:cols=70
                        attr:rows=8
                        prop:value=move || rle.get()
                    />
                    <FileInput on_change=on_file_change />
                    {move || {
                        if !error_text.get().is_empty() {
                            view! {
                                <div class="text-red-400 flex items-center gap-2">
                                    // looks better with this pixel
                                    <div class="mb-px">
                                        <Icon icon=icondata::LuCircleAlert size=IconSize::Small />
                                    </div>
                                    <span class="text-sm">{move || error_text.get()}</span>
                                </div>
                            }
                                .into_any()
                        } else {
                            ().into_any()
                        }
                    }}
                </div>
                <div class="w-full flex justify-end">
                    <Button variant=ButtonVariant::Primary attr:r#type="submit" class="rounded-md">
                        IMPORT
                    </Button>
                </div>
            </div>
        </form>
    }
}

#[component]
pub fn MenuButton(
    children: Children,
    #[prop(into, optional)] on_press: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        // HACK: passing empty Callback instead of None
        <Button
            class="w-full flex items-center gap-2 px-2"
            on_press=on_press.unwrap_or(Callback::from(|| {}))
        >
            {children()}
        </Button>
    }
}

#[component]
pub fn AppMenu() -> impl IntoView {
    let (is_open, set_is_open) = signal(false);
    let (is_import_open, set_is_import_open) = signal(false);
    let GolContext { universe, name, .. } = use_context::<GolContext>().unwrap();

    view! {
        <PopoverTrigger is_open=is_open set_is_open=set_is_open>
            <Surface class="overflow-hidden">
                <Button
                    variant=ButtonVariant::Icon
                    on_press=move || {
                        set_is_open.update(|open| *open = !*open);
                    }
                >
                    <Icon icon=icondata::LuMenu />
                </Button>
            </Surface>

            <Popover placement=PopoverPlacement::TopStart>
                <Surface class="flex flex-col justify-start overflow-hidden mb-2">
                    <div>
                        <MenuButton on_press=move || {
                            universe
                                .with(|u| {
                                    let (x1, y1, x2, y2) = u.get_bounding_rect();
                                    let rle = rle::from_iter(u.iter_alive(), x1, y1, x2, y2);
                                    let mut filename = name.get();
                                    if !filename.ends_with(".rle") {
                                        filename = format!("{}.rle", filename);
                                    }
                                    download_text_file(&filename, &rle);
                                });
                        }>
                            <Icon icon=icondata::LuFileDown />
                            Export
                        </MenuButton>
                    </div>

                    <PopoverTrigger is_open=is_import_open set_is_open=set_is_import_open>
                        <MenuButton on_press=move || {
                            set_is_import_open.set(true);
                        }>
                            <Icon icon=icondata::LuFileUp />
                            Import
                        </MenuButton>
                        <Dialog>
                            <ImportForm close=move || set_is_import_open.set(false) />
                        </Dialog>
                    </PopoverTrigger>

                    <Link
                        variant=LinkVariant::Icon
                        class="justify-start gap-2"
                        href="https://github.com/pharaok/rust-wasm-gol".to_owned()
                        attr:target="_blank"
                    >
                        <Icon icon=icondata::SiGithub />
                        Source
                    </Link>
                </Surface>
            </Popover>
        </PopoverTrigger>
    }
}
