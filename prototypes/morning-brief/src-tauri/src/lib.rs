use tauri::{AppHandle, Emitter, Manager};

#[cfg(desktop)]
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

const FONT_REGULAR: &[u8] = include_bytes!("../fonts/Inter-Regular.ttf");
const FONT_MEDIUM: &[u8] = include_bytes!("../fonts/Inter-Medium.ttf");
const FONT_SEMIBOLD: &[u8] = include_bytes!("../fonts/Inter-SemiBold.ttf");

#[tauri::command]
fn get_bundled_font(name: String) -> Result<Vec<u8>, String> {
    match name.as_str() {
        "Inter-Regular" | "Inter-Regular.ttf" => Ok(FONT_REGULAR.to_vec()),
        "Inter-Medium" | "Inter-Medium.ttf" => Ok(FONT_MEDIUM.to_vec()),
        "Inter-SemiBold" | "Inter-SemiBold.ttf" => Ok(FONT_SEMIBOLD.to_vec()),
        _ => Err(format!("unknown bundled font: {name}")),
    }
}

#[cfg(desktop)]
fn register_global_shortcuts(app: &tauri::App) -> tauri::Result<()> {
    let omnibar_shortcut = shortcut_for_platform(Code::Space);
    let inbox_shortcut = shortcut_for_platform(Code::Period);

    app.handle().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, shortcut, event| {
                if event.state() != ShortcutState::Pressed {
                    return;
                }

                if shortcut == &omnibar_shortcut {
                    emit_shortcut_toggle(app, "omnibar:toggle");
                    println!("TODO: handle omnibar global shortcut");
                } else if shortcut == &inbox_shortcut {
                    emit_shortcut_toggle(app, "inbox:toggle");
                    println!("TODO: handle inbox global shortcut");
                }
            })
            .build(),
    )?;

    if let Err(e) = app.global_shortcut().register(omnibar_shortcut) {
        eprintln!("failed to register omnibar shortcut: {e}");
    }
    if let Err(e) = app.global_shortcut().register(inbox_shortcut) {
        eprintln!("failed to register inbox shortcut: {e}");
    }

    Ok(())
}

#[cfg(desktop)]
fn shortcut_for_platform(code: Code) -> Shortcut {
    #[cfg(target_os = "macos")]
    let modifiers = Modifiers::SUPER;

    #[cfg(not(target_os = "macos"))]
    let modifiers = Modifiers::CONTROL;

    Shortcut::new(Some(modifiers), code)
}

#[cfg(desktop)]
fn emit_shortcut_toggle(app: &AppHandle, event: &str) {
    if let Err(error) = app.emit(event, ()) {
        eprintln!("failed to emit {event}: {error}");
    }
}

#[cfg(target_os = "macos")]
fn apply_window_vibrancy(window: &tauri::WebviewWindow) {
    use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};

    if let Err(error) = apply_vibrancy(window, NSVisualEffectMaterial::HudWindow, None, None) {
        eprintln!("failed to apply macOS window vibrancy: {error}");
    }
}

#[cfg(not(target_os = "macos"))]
fn apply_window_vibrancy(_window: &tauri::WebviewWindow) {}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_bundled_font])
        .setup(|app| {
            #[cfg(desktop)]
            register_global_shortcuts(app)?;

            if let Some(window) = app.get_webview_window("main") {
                apply_window_vibrancy(&window);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running morning brief native app");
}
