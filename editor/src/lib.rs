#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use winit::platform::web::EventLoopExtWebSys;

use winit::event_loop::EventLoop;

mod app;
mod state;

use crate::app::App;

fn setup_tracing() {
    #[cfg(target_arch = "wasm32")]
    {
        tracing_wasm::set_as_global_default();
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        use tracing_subscriber::Layer;
        use tracing_subscriber::layer::SubscriberExt;

        color_eyre::config::HookBuilder::new()
            .panic_section("report this issue at https://github.com/RadiatedMonkey/mktools")
            // .issue_url("https://github.com/RadiatedMonkey/mktools/issues/new")
            // .add_issue_metadata("version", "v0.1.0")
            .display_location_section(true)
            .display_env_section(true)
            .install()
            .unwrap();

        let filter = tracing_subscriber::filter::filter_fn(|meta| !meta.target().contains("winit"));

        let layer = tracing_tree::HierarchicalLayer::new(2)
            .with_indent_lines(true)
            .with_targets(true)
            .with_bracketed_fields(true)
            .with_filter(filter);

        let subscriber = tracing_subscriber::Registry::default().with(layer);

        tracing::subscriber::set_global_default(subscriber).unwrap();
    }

    tracing::debug!("Logging initialized");
}

pub fn run() -> eyre::Result<()> {
    setup_tracing();

    let event_loop = EventLoop::with_user_event().build()?;
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut app = App::new();
        event_loop.run_app(&mut app)?;
    }

    #[cfg(target_arch = "wasm32")]
    {
        let app = App::new(&event_loop);
        event_loop.spawn_app(app);
    }

    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_web() -> Result<(), wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();
    run().unwrap_throw();

    Ok(())
}
