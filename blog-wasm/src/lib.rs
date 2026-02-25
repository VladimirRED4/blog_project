mod api;
mod app;
mod models;

use app::App;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // Устанавливаем обработчик паники
    console_error_panic_hook::set_once();

    // Инициализируем логирование
    #[cfg(debug_assertions)]
    console_log::init_with_level(log::Level::Debug).unwrap_or_else(|e| {
        web_sys::console::log_1(&format!("Failed to init logger: {}", e).into());
    });

    // Запускаем Yew приложение
    yew::Renderer::<App>::new().render();

    Ok(())
}
