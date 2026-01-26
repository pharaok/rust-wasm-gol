use crate::{
    app::{GolContext, use_fit_universe},
    components::{
        Button, ButtonVariant, Dialog, FileInput, Icon, IconSize, Link, LinkVariant, Popover,
        PopoverPlacement, PopoverTrigger, Surface, TextArea, use_toast,
    },
    parse::rle,
    universe::InsertMode,
    utils::{base64_gz_from_str, download_text_file, str_from_base64_gz},
};
use leptos::{logging, prelude::*, task::spawn_local};
use leptos_router::hooks::use_url;
use leptos_use::{UseClipboardReturn, use_clipboard};
use wasm_bindgen_futures::JsFuture;
use web_sys::File;

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
            };
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
                    <FileInput on_change=on_file_change accept=".rle,.txt" />
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

    let url = use_url();
    let UseClipboardReturn { copy, .. } = use_clipboard();
    let copy = StoredValue::new(copy);

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
                            <h2 class="text-lg font-bold text-center">IMPORT PATTERN</h2>
                            <div class="border-t border-neutral-800 w-full" />
                            <ImportForm close=move || set_is_import_open.set(false) />
                        </Dialog>
                    </PopoverTrigger>

                    <MenuButton on_press=move || {
                        let rle = universe
                            .with(|u| {
                                let (x1, y1, x2, y2) = u.get_bounding_rect();
                                rle::from_iter(u.iter_alive(), x1, y1, x2, y2)
                            });
                        spawn_local(async move {
                            let base64 = base64_gz_from_str(&rle).await.unwrap();
                            copy.read_value()(&format!("{}/#{}", url.get().origin(), base64));
                        });
                    }>
                        <Icon icon=icondata::LuShare2 />
                        Share
                    </MenuButton>

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
