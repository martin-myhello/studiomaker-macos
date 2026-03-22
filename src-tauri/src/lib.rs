#[cfg(target_os = "macos")]
mod browser;
#[cfg(target_os = "macos")]
mod menu;
#[cfg(target_os = "macos")]
mod updater;

#[cfg(target_os = "macos")]
use tauri::Manager;

#[cfg(target_os = "macos")]
fn setup_macos_window(window: &tauri::WebviewWindow) {
    use cocoa::appkit::{NSColor, NSWindow};
    use cocoa::base::{id, nil};

    if let Ok(ns_window) = window.ns_window() {
        let ns_window = ns_window as id;
        unsafe {
            let bg_color = NSColor::colorWithSRGBRed_green_blue_alpha_(
                nil,
                1.0,
                1.0,
                1.0,
                1.0,
            );
            ns_window.setBackgroundColor_(bg_color);
        }
    }
}

#[cfg(target_os = "macos")]
const STUDIO_URL: &str = "https://studiomaker.app";

/// Returns true if the URL is an allowed domain (studiomaker.app, supabase, googleapis)
#[cfg(target_os = "macos")]
fn is_allowed_url(url: &str) -> bool {
    let allowed_patterns = [
        "http://localhost",
        "https://studiomaker.app",
        "https://www.studiomaker.app",
        // Supabase for API calls
        ".supabase.co",
        // Google APIs for calendar integration
        ".googleapis.com",
        // Google auth
        "accounts.google.com",
    ];
    for pattern in &allowed_patterns {
        if url.starts_with(pattern) || url.contains(pattern) {
            return true;
        }
    }
    false
}

#[tauri::command]
fn open_external(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_platform() -> String {
    #[cfg(target_os = "macos")]
    {
        "macos".to_string()
    }
    #[cfg(target_os = "ios")]
    {
        // Distinguish iPhone from iPad using screen size heuristic
        // On iOS, UIDevice info isn't directly available from Rust,
        // so the web app's JS detection handles iPhone vs iPad
        "ios".to_string()
    }
    #[cfg(not(any(target_os = "macos", target_os = "ios")))]
    {
        "unknown".to_string()
    }
}

#[cfg(target_os = "macos")]
#[tauri::command]
fn open_new_window(app: tauri::AppHandle) -> Result<(), String> {
    let label = format!(
        "studio-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );

    let _window = tauri::WebviewWindowBuilder::new(
        &app,
        &label,
        tauri::WebviewUrl::External(STUDIO_URL.parse().unwrap()),
    )
    .title("StudioMaker")
    .inner_size(1280.0, 860.0)
    .min_inner_size(380.0, 380.0)
    .decorations(true)
    .hidden_title(true)
    .title_bar_style(tauri::TitleBarStyle::Overlay)
    .traffic_light_position(tauri::Position::Logical(tauri::LogicalPosition::new(7.0, 16.0)))
    .on_navigation(|url| {
        // Allow navigation to studiomaker.app and partner domains
        is_allowed_url(url.as_str())
    })
    .build()
    .map_err(|e| e.to_string())?;

    setup_macos_window(&_window);

    Ok(())
}

/// No-op on iOS — multi-window is not supported
#[cfg(target_os = "ios")]
#[tauri::command]
fn open_new_window(_app: tauri::AppHandle) -> Result<(), String> {
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_pencilkit::init());

    // macOS-only plugins (window state, updater)
    #[cfg(target_os = "macos")]
    {
        builder = builder
            .plugin(tauri_plugin_window_state::Builder::new().with_filename("window-state.json").build())
            .plugin(tauri_plugin_updater::Builder::new().build());
    }

    builder
        .invoke_handler(tauri::generate_handler![
            open_new_window,
            open_external,
            get_platform,
            #[cfg(target_os = "macos")]
            updater::check_for_updates,
            #[cfg(target_os = "macos")]
            updater::download_update,
            #[cfg(target_os = "macos")]
            updater::install_update,
            #[cfg(target_os = "macos")]
            browser::commands::open_browser_window,
            #[cfg(target_os = "macos")]
            browser::commands::browser_navigate,
            #[cfg(target_os = "macos")]
            browser::commands::browser_go_back,
            #[cfg(target_os = "macos")]
            browser::commands::browser_go_forward,
            #[cfg(target_os = "macos")]
            browser::commands::browser_get_page_info,
            #[cfg(target_os = "macos")]
            browser::commands::browser_new_tab,
            #[cfg(target_os = "macos")]
            browser::commands::browser_close_tab,
            #[cfg(target_os = "macos")]
            browser::commands::browser_switch_tab,
        ])
        .setup(|_app| {
            #[cfg(target_os = "macos")]
            {
                let window = _app.get_webview_window("main").unwrap();
                setup_macos_window(&window);
                menu::setup_menu(_app)?;
                updater::spawn_update_checker(_app.handle());
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
