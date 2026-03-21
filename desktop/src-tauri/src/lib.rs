#[cfg(not(debug_assertions))]
use plugins::logging;
use tauri::Manager;
use tauri_plugin_decorum::WebviewWindowExt;

pub mod plugins;

const APP_ICON: tauri::image::Image<'_> = tauri::include_image!("./icons/32x32.png");

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder =
        tauri::Builder::default().plugin(tauri_plugin_single_instance::init(|_, _, _| {}));

    // CrabNebula DevTools prevents other logging plugins from working
    // https://docs.crabnebula.dev/devtools/troubleshoot/log-plugins/
    #[cfg(debug_assertions)]
    {
        let devtools = tauri_plugin_devtools::init();
        builder = builder.plugin(devtools);
    }

    #[cfg(not(debug_assertions))]
    {
        builder = builder.plugin(logging::tauri_plugin_logging());
    }
    builder
        .plugin(tauri_plugin_decorum::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![])
        .setup(|app| {
            let main_window = app.get_webview_window("main").unwrap();
            main_window
                .set_icon(APP_ICON)
                .expect("failed to set main window icon");
            main_window.create_overlay_titlebar().unwrap();

            // Some macOS-specific helpers
            #[cfg(target_os = "macos")]
            {
                // Set a custom inset to the traffic lights
                main_window.set_traffic_lights_inset(12.0, 16.0).unwrap();

                // Make window transparent without privateApi
                main_window.make_transparent().unwrap();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
