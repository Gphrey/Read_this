#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    fs,
    io::Cursor,
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use base64::{engine::general_purpose, Engine as _};
use clipboard_win::{formats, get_clipboard, set_clipboard, Clipboard};
use msedge_tts::{
    tts::{client::connect, SpeechConfig},
    voice::{get_voices_list, Voice},
};
use rodio::{MixerDeviceSink, Player};
use serde::{Deserialize, Serialize};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager, State, WindowEvent,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, ShortcutState};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_C,
    VK_CONTROL, VK_MENU, VK_R,
};

const DEFAULT_VOICE: &str = "en-US-EmmaMultilingualNeural";
const DEFAULT_HOTKEY: &str = "Ctrl+Alt+R";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppSettings {
    voice_id: String,
    rate: i32,
    volume: u8,
    hotkey: String,
    enable_local_fallback: bool,
    start_minimized: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            voice_id: DEFAULT_VOICE.to_string(),
            rate: 15,
            volume: 100,
            hotkey: DEFAULT_HOTKEY.to_string(),
            enable_local_fallback: false,
            start_minimized: true,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct ReadStatus {
    state: String,
    message: String,
}

impl ReadStatus {
    fn idle(message: impl Into<String>) -> Self {
        Self {
            state: "Idle".into(),
            message: message.into(),
        }
    }

    fn reading(message: impl Into<String>) -> Self {
        Self {
            state: "Reading".into(),
            message: message.into(),
        }
    }

    fn fetching(message: impl Into<String>) -> Self {
        Self {
            state: "FetchingVoice".into(),
            message: message.into(),
        }
    }

    fn error(message: impl Into<String>) -> Self {
        Self {
            state: "Error".into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct VoiceInfo {
    id: String,
    label: String,
    locale: String,
    gender: String,
}

struct PlaybackHandle {
    _stream: MixerDeviceSink,
    player: Arc<Player>,
}

struct AppState {
    settings: Mutex<AppSettings>,
    playback: Mutex<Option<PlaybackHandle>>,
    local_child: Mutex<Option<Child>>,
    status: Mutex<ReadStatus>,
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let settings = load_settings(app.handle()).unwrap_or_default();
            let state = Arc::new(AppState {
                settings: Mutex::new(settings.clone()),
                playback: Mutex::new(None),
                local_child: Mutex::new(None),
                status: Mutex::new(ReadStatus::idle("Ready.")),
            });
            app.manage(state);

            create_tray(app.handle())?;
            if let Err(error) = register_shortcuts(app.handle()) {
                let state = app.state::<Arc<AppState>>().inner().clone();
                set_status(
                    app.handle(),
                    &state,
                    ReadStatus::idle(format!(
                        "Global shortcut unavailable, using keyboard watcher: {error}"
                    )),
                );
            }
            start_hotkey_polling(app.handle().clone());

            if !settings.start_minimized {
                show_main_window(app.handle());
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            read_selection,
            stop_reading,
            test_voice,
            list_voices,
            get_settings,
            save_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running Read This");
}

fn create_tray(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open Panel", true, None::<&str>)?;
    let read = MenuItem::with_id(app, "read", "Read Highlighted Text", true, None::<&str>)?;
    let stop = MenuItem::with_id(app, "stop", "Stop", true, None::<&str>)?;
    let test = MenuItem::with_id(app, "test", "Test Voice", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&open, &read, &stop, &test, &separator, &quit])?;

    let icon = app.default_window_icon().cloned();
    let mut tray = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .tooltip("Read This");

    if let Some(icon) = icon {
        tray = tray.icon(icon);
    }

    tray.on_menu_event(|app, event| match event.id().as_ref() {
        "open" => show_main_window(app),
        "read" => trigger_read(app.clone()),
        "stop" => {
            let state = app.state::<Arc<AppState>>().inner().clone();
            let _ = stop_reading_impl(app, &state);
        }
        "test" => trigger_test(app.clone()),
        "quit" => {
            let state = app.state::<Arc<AppState>>().inner().clone();
            let _ = stop_reading_impl(app, &state);
            app.exit(0);
        }
        _ => {}
    })
    .build(app)?;

    Ok(())
}

fn register_shortcuts(app: &AppHandle) -> Result<(), String> {
    app.plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .map_err(|e| e.to_string())?;

    app.global_shortcut()
        .on_shortcut("ctrl+alt+r", |app, shortcut, event| {
            if event.state == ShortcutState::Pressed
                && shortcut.matches(Modifiers::CONTROL | Modifiers::ALT, Code::KeyR)
            {
                trigger_read(app.clone());
            }
        })
        .map_err(|e| e.to_string())
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn trigger_read(app: AppHandle) {
    let state = app.state::<Arc<AppState>>().inner().clone();
    tauri::async_runtime::spawn(async move {
        let app_for_task = app.clone();
        let _ = tauri::async_runtime::spawn_blocking(move || read_selection_impl(&app_for_task, &state)).await;
    });
}

fn trigger_test(app: AppHandle) {
    let state = app.state::<Arc<AppState>>().inner().clone();
    tauri::async_runtime::spawn(async move {
        let app_for_task = app.clone();
        let _ = tauri::async_runtime::spawn_blocking(move || test_voice_impl(&app_for_task, &state)).await;
    });
}

fn start_hotkey_polling(app: AppHandle) {
    thread::spawn(move || {
        let mut last_read = false;
        loop {
            let pressed = unsafe {
                is_key_down(VK_CONTROL) && is_key_down(VK_MENU) && is_key_down(VK_R)
            };
            if pressed && !last_read {
                trigger_read(app.clone());
            }
            last_read = pressed;
            thread::sleep(Duration::from_millis(90));
        }
    });
}

#[tauri::command]
async fn read_selection(app: AppHandle, state: State<'_, Arc<AppState>>) -> Result<ReadStatus, String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || read_selection_impl(&app, &state))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn stop_reading(app: AppHandle, state: State<'_, Arc<AppState>>) -> Result<ReadStatus, String> {
    stop_reading_impl(&app, state.inner())
}

#[tauri::command]
async fn test_voice(app: AppHandle, state: State<'_, Arc<AppState>>) -> Result<ReadStatus, String> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || test_voice_impl(&app, &state))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn list_voices() -> Result<Vec<VoiceInfo>, String> {
    tauri::async_runtime::spawn_blocking(load_voice_infos)
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn get_settings(state: State<'_, Arc<AppState>>) -> Result<AppSettings, String> {
    state
        .settings
        .lock()
        .map(|settings| settings.clone())
        .map_err(|_| "Could not read settings.".to_string())
}

#[tauri::command]
fn save_settings(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    settings: AppSettings,
) -> Result<(), String> {
    {
        let mut current = state
            .settings
            .lock()
            .map_err(|_| "Could not update settings.".to_string())?;
        *current = settings.clone();
    }
    save_settings_file(&app, &settings)
}

fn read_selection_impl(app: &AppHandle, state: &Arc<AppState>) -> Result<ReadStatus, String> {
    let text = capture_selected_text()?;
    if text.trim().is_empty() {
        let status = ReadStatus::error("No highlighted text was detected.");
        set_status(app, state, status.clone());
        return Ok(status);
    }
    speak_text(app, state, &text)
}

fn test_voice_impl(app: &AppHandle, state: &Arc<AppState>) -> Result<ReadStatus, String> {
    speak_text(app, state, "Read This is ready.")
}

fn speak_text(app: &AppHandle, state: &Arc<AppState>, text: &str) -> Result<ReadStatus, String> {
    stop_reading_impl(app, state)?;
    let settings = state
        .settings
        .lock()
        .map_err(|_| "Could not read settings.".to_string())?
        .clone();

    set_status(app, state, ReadStatus::fetching("Fetching Edge TTS audio..."));
    match synthesize_edge_tts(text, &settings) {
        Ok(audio) => play_audio(app, state, audio, settings.volume),
        Err(edge_error) if settings.enable_local_fallback => {
            let msg = format!("Edge TTS failed, using local fallback: {edge_error}");
            speak_local_fallback(app, state, text, &settings, msg)
        }
        Err(edge_error) => {
            let status = ReadStatus::error(format!("Edge TTS failed: {edge_error}"));
            set_status(app, state, status.clone());
            Ok(status)
        }
    }
}

fn synthesize_edge_tts(text: &str, settings: &AppSettings) -> Result<Vec<u8>, String> {
    match synthesize_edge_tts_python(text, settings) {
        Ok(audio) => return Ok(audio),
        Err(python_error) => {
            let rust_result = synthesize_edge_tts_rust(text, settings);
            if let Err(rust_error) = rust_result.as_ref() {
                return Err(format!(
                    "Python edge-tts failed: {python_error}; Rust Edge TTS failed: {rust_error}"
                ));
            }
            return rust_result;
        }
    }
}

fn synthesize_edge_tts_python(text: &str, settings: &AppSettings) -> Result<Vec<u8>, String> {
    let root = find_project_root().ok_or_else(|| "Could not find python_deps.".to_string())?;
    let python = find_python_executable().ok_or_else(|| "Could not find Python.".to_string())?;
    let output = std::env::temp_dir().join(format!(
        "read-this-edge-{}.mp3",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_millis()
    ));

    let rate = format!("{:+}%", settings.rate);
    let status = Command::new(&python)
        .arg("-m")
        .arg("edge_tts")
        .arg("--text")
        .arg(text)
        .arg("--voice")
        .arg(&settings.voice_id)
        .arg("--rate")
        .arg(rate)
        .arg("--write-media")
        .arg(&output)
        .env("PYTHONPATH", root.join("python_deps"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| e.to_string())?;

    if !status.status.success() {
        let stderr = String::from_utf8_lossy(&status.stderr);
        let _ = fs::remove_file(&output);
        return Err(stderr.trim().to_string());
    }

    let audio = fs::read(&output).map_err(|e| e.to_string())?;
    let _ = fs::remove_file(&output);
    if audio.is_empty() {
        Err("edge-tts returned an empty audio file.".to_string())
    } else {
        Ok(audio)
    }
}

fn synthesize_edge_tts_rust(text: &str, settings: &AppSettings) -> Result<Vec<u8>, String> {
    let voices = get_voices_list().map_err(|e| e.to_string())?;
    let voice = select_voice(&voices, &settings.voice_id)
        .ok_or_else(|| format!("Voice '{}' was not found.", settings.voice_id))?;

    let mut config = SpeechConfig::from(voice);
    config.rate = settings.rate;
    config.volume = 0;

    let mut client = connect().map_err(|e| e.to_string())?;
    let audio = client
        .synthesize(&escape_ssml(text), &config)
        .map_err(|e| e.to_string())?;
    Ok(audio.audio_bytes)
}

fn find_project_root() -> Option<std::path::PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(current) = std::env::current_dir() {
        candidates.push(current);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.to_path_buf());
        }
    }

    for candidate in candidates {
        for ancestor in candidate.ancestors() {
            if ancestor.join("python_deps").join("edge_tts").exists() {
                return Some(ancestor.to_path_buf());
            }
        }
    }

    None
}

fn find_python_executable() -> Option<std::path::PathBuf> {
    let bundled = std::path::PathBuf::from(
        r"C:\Users\Godfrey\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe",
    );
    if bundled.exists() {
        return Some(bundled);
    }

    for name in ["python.exe", "python", "py.exe", "py"] {
        if Command::new(name)
            .arg("--version")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
        {
            return Some(std::path::PathBuf::from(name));
        }
    }

    None
}

fn play_audio(
    app: &AppHandle,
    state: &Arc<AppState>,
    audio: Vec<u8>,
    volume: u8,
) -> Result<ReadStatus, String> {
    let stream = rodio::DeviceSinkBuilder::open_default_sink().map_err(|e| e.to_string())?;
    let cursor = Cursor::new(audio);
    let player = rodio::play(stream.mixer(), cursor).map_err(|e| e.to_string())?;
    player.set_volume((volume as f32 / 100.0).clamp(0.0, 1.0));

    let player = Arc::new(player);
    let monitor_player = player.clone();
    {
        let mut playback = state
            .playback
            .lock()
            .map_err(|_| "Could not update playback state.".to_string())?;
        *playback = Some(PlaybackHandle {
            _stream: stream,
            player,
        });
    }

    let app_handle = app.clone();
    let state_handle = state.clone();
    thread::spawn(move || {
        monitor_player.sleep_until_end();
        if let Ok(mut playback) = state_handle.playback.lock() {
            if playback
                .as_ref()
                .map(|current| Arc::ptr_eq(&current.player, &monitor_player))
                .unwrap_or(false)
            {
                *playback = None;
                set_status(&app_handle, &state_handle, ReadStatus::idle("Finished reading."));
            }
        }
    });

    let status = ReadStatus::reading("Reading highlighted text.");
    set_status(app, state, status.clone());
    Ok(status)
}

fn speak_local_fallback(
    app: &AppHandle,
    state: &Arc<AppState>,
    text: &str,
    settings: &AppSettings,
    message: String,
) -> Result<ReadStatus, String> {
    let encoded_text = general_purpose::STANDARD.encode(text.as_bytes());
    let rate = ((settings.rate as f32 / 10.0).round() as i32).clamp(-10, 10);
    let volume = settings.volume.min(100);
    let script = format!(
        "$text=[Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('{encoded_text}'));\
         $voice=New-Object -ComObject SAPI.SpVoice;\
         $voice.Rate={rate};\
         $voice.Volume={volume};\
         [void]$voice.Speak($text);"
    );
    let encoded_script = encode_powershell_command(&script);
    let child = Command::new("powershell.exe")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-EncodedCommand", &encoded_script])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;

    {
        let mut local_child = state
            .local_child
            .lock()
            .map_err(|_| "Could not update local fallback state.".to_string())?;
        *local_child = Some(child);
    }

    let app_handle = app.clone();
    let state_handle = state.clone();
    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(250));
        let finished = {
            let mut child_lock = match state_handle.local_child.lock() {
                Ok(lock) => lock,
                Err(_) => return,
            };
            match child_lock.as_mut().and_then(|child| child.try_wait().ok().flatten()) {
                Some(_) => {
                    *child_lock = None;
                    true
                }
                None => false,
            }
        };
        if finished {
            set_status(&app_handle, &state_handle, ReadStatus::idle("Finished reading."));
            return;
        }
    });

    let status = ReadStatus::reading(message);
    set_status(app, state, status.clone());
    Ok(status)
}

fn stop_reading_impl(app: &AppHandle, state: &Arc<AppState>) -> Result<ReadStatus, String> {
    if let Ok(mut playback) = state.playback.lock() {
        if let Some(handle) = playback.take() {
            handle.player.stop();
        }
    }

    if let Ok(mut child) = state.local_child.lock() {
        if let Some(mut process) = child.take() {
            let _ = process.kill();
            let _ = process.wait();
        }
    }

    let status = ReadStatus::idle("Stopped.");
    set_status(app, state, status.clone());
    Ok(status)
}

fn set_status(app: &AppHandle, state: &Arc<AppState>, status: ReadStatus) {
    if let Ok(mut current) = state.status.lock() {
        *current = status.clone();
    }
    let _ = app.emit("status-changed", status);
}

fn load_voice_infos() -> Result<Vec<VoiceInfo>, String> {
    let mut voices: Vec<VoiceInfo> = get_voices_list()
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter_map(|voice| {
            let id = voice.short_name.clone().unwrap_or_else(|| voice.name.clone());
            let locale = voice.locale.unwrap_or_default();
            if !locale.starts_with("en-") {
                return None;
            }
            let friendly = voice.friendly_name.unwrap_or_else(|| id.clone());
            Some(VoiceInfo {
                id,
                label: friendly,
                locale,
                gender: voice.gender.unwrap_or_default(),
            })
        })
        .collect();

    voices.sort_by(|a, b| a.label.cmp(&b.label));
    Ok(voices)
}

fn select_voice<'a>(voices: &'a [Voice], voice_id: &str) -> Option<&'a Voice> {
    voices
        .iter()
        .find(|voice| {
            voice.short_name.as_deref() == Some(voice_id)
                || voice.name == voice_id
                || voice.name.contains(voice_id)
        })
        .or_else(|| {
            voices.iter().find(|voice| {
                voice.short_name.as_deref() == Some(DEFAULT_VOICE)
                    || voice.name.contains("EmmaMultilingualNeural")
            })
        })
        .or_else(|| {
            voices.iter().find(|voice| {
                voice
                    .locale
                    .as_deref()
                    .map(|locale| locale.starts_with("en-"))
                    .unwrap_or(false)
            })
        })
}

fn capture_selected_text() -> Result<String, String> {
    wait_for_modifiers_to_release();
    let previous_text: Option<String> = get_clipboard(formats::Unicode).ok();

    {
        let _clip = Clipboard::new_attempts(10).map_err(|e| e.to_string())?;
        clipboard_win::empty().map_err(|e| e.to_string())?;
    }

    send_ctrl_c()?;

    let mut copied = String::new();
    for _ in 0..15 {
        thread::sleep(Duration::from_millis(80));
        if let Ok(text) = get_clipboard::<String, _>(formats::Unicode) {
            if !text.trim().is_empty() {
                copied = text;
                break;
            }
        }
    }

    match previous_text {
        Some(text) => {
            let _ = set_clipboard(formats::Unicode, text.as_str());
        }
        None => {
            if let Ok(_clip) = Clipboard::new_attempts(10) {
                let _ = clipboard_win::empty();
            }
        }
    }

    Ok(copied)
}

fn wait_for_modifiers_to_release() {
    for _ in 0..20 {
        let pressed = unsafe { is_key_down(VK_CONTROL) || is_key_down(VK_MENU) };
        if !pressed {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
}

fn send_ctrl_c() -> Result<(), String> {
    unsafe {
        send_key(VK_CONTROL, false)?;
        send_key(VK_C, false)?;
        send_key(VK_C, true)?;
        send_key(VK_CONTROL, true)?;
    }
    Ok(())
}

unsafe fn is_key_down(key: u16) -> bool {
    unsafe { (GetAsyncKeyState(key as i32) & 0x8000u16 as i16) != 0 }
}

unsafe fn send_key(key: u16, key_up: bool) -> Result<(), String> {
    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key,
                wScan: 0,
                dwFlags: if key_up { KEYEVENTF_KEYUP } else { 0 },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    let sent = unsafe { SendInput(1, &input, std::mem::size_of::<INPUT>() as i32) };
    if sent == 0 {
        Err("Windows did not accept simulated Ctrl+C.".to_string())
    } else {
        Ok(())
    }
}

fn escape_ssml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn encode_powershell_command(script: &str) -> String {
    let bytes: Vec<u8> = script
        .encode_utf16()
        .flat_map(|unit| unit.to_le_bytes())
        .collect();
    general_purpose::STANDARD.encode(bytes)
}

fn load_settings(app: &AppHandle) -> Result<AppSettings, String> {
    let path = settings_path(app)?;
    if !path.exists() {
        return Ok(AppSettings::default());
    }
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    Ok(serde_json::from_str(&text).unwrap_or_default())
}

fn save_settings_file(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let path = settings_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(path, text).map_err(|e| e.to_string())
}

fn settings_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?;
    Ok(dir.join("settings.json"))
}
