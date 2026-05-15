use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    webview::PageLoadEvent,
    Manager, WebviewUrl, WebviewWindowBuilder,
};

const WHATSAPP_URL: &str = "https://web.whatsapp.com";

/// Returns a Chrome-compatible user-agent string so WhatsApp Web
/// does not show its "unsupported browser" banner.
fn user_agent() -> String {
    // Chrome 124 on Windows 11 – WhatsApp Web requires a Chromium-based user-agent.
    // Update the Chrome version number periodically if WhatsApp Web shows a
    // "browser not supported" warning (check https://www.whatismybrowser.com
    // for a current Chrome UA to copy).
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
     AppleWebKit/537.36 (KHTML, like Gecko) \
     Chrome/124.0.0.0 Safari/537.36"
        .to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // ---------------------------------------------------------------
            // Main window – loads WhatsApp Web directly in the system WebView.
            // Using the OS WebView (WebView2 on Windows) keeps memory usage
            // far lower than bundling a full Chromium runtime like Electron.
            // ---------------------------------------------------------------
            let _win = WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::External(WHATSAPP_URL.parse().unwrap()),
            )
            .title("WhatsApp")
            .inner_size(1200.0, 800.0)
            .min_inner_size(800.0, 600.0)
            .resizable(true)
            .user_agent(&user_agent())
            // Start hidden; shown once the page finishes loading (no white flash).
            .visible(false)
            .on_page_load(|win, payload| {
                if payload.event() == PageLoadEvent::Finished {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            })
            .build()?;

            // ---------------------------------------------------------------
            // System-tray icon with a Quit menu item.
            // This lets the user close the window without fully quitting so
            // WhatsApp notifications keep working in the background.
            // ---------------------------------------------------------------
            let quit = MenuItem::with_id(app, "quit", "Quit WhatsApp", true, None::<&str>)?;
            let show = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("WhatsApp")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "show" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    // Single-click the tray icon → toggle visibility.
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(win) = app.get_webview_window("main") {
                            if win.is_visible().unwrap_or(false) {
                                let _ = win.hide();
                            } else {
                                let _ = win.show();
                                let _ = win.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        // Intercept the close button: hide to tray instead of quitting.
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
