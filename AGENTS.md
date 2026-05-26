<claude-mem-context>
# Memory Context

# [Read_This] recent context, 2026-05-26 3:51pm GMT+1

Legend: 🎯session 🔴bugfix 🟣feature 🔄refactor ✅change 🔵discovery ⚖️decision 🚨security_alert 🔐security_note
Format: ID TIME TYPE TITLE
Fetch details: get_observations([IDs]) | Search: mem-search skill

Stats: 50 obs (15,261t read) | 215,348t work | 93% savings

### May 6, 2026
S199 Plan and architect Tauri + Rust desktop application for Read This with edge-TTS voice synthesis (May 6, 7:29 PM)
### May 24, 2026
S201 Implement Tauri/Rust desktop app replacing PowerShell Read This prototype with Edge TTS voice synthesis (May 24, 1:36 PM)
434 1:47p 🔵 msedge-tts API surface supports both sync client and streaming patterns suitable for Tauri
S202 Implement Tauri/Rust desktop app replacing PowerShell Read This prototype with Edge TTS voice synthesis (May 24, 1:47 PM)
435 " 🔵 msedge-tts public API structure maps directly to Tauri command requirements
436 1:48p 🔵 msedge-tts SSML/WebSocket protocol details and default behavior
S203 Implement Tauri/Rust desktop app replacing PowerShell Read This prototype with Edge TTS voice synthesis (May 24, 1:48 PM)
439 1:49p 🔵 tauri-plugin-global-shortcut API: Builder pattern with per-shortcut handlers
440 " 🔵 Rodio architecture: cpal backend, Symphonia decoders, comprehensive playback examples
441 1:50p 🔵 Rodio playback pattern: DeviceSinkBuilder → mixer → play() with volume control
442 " 🔵 Rodio play() function auto-detects format via decoder::Decoder
S206 Continue Tauri Windows build - resolve missing icon resource error and validate build completion (May 24, 1:50 PM)
443 " 🔵 Cargo arboard query timed out; clipboard access may require alternative approach
445 1:51p 🔵 clipboard-win API: dual interface (manual lock + simplified) with Unicode text support
446 " 🔵 clipboard-win clearing behavior is configurable via Clearing trait
447 1:52p 🔵 Rodio playback pattern: Player vs Mixer for sequential vs parallel audio
448 " 🔵 Rodio Player control: stop(), empty(), sleep_until_end(), detach() for playback management
449 " 🔵 Rodio Player queue management: len(), empty(), skip_one(), clear(), pause(), play(), get_pos()
454 1:54p 🔵 windows-sys INPUT struct union: MOUSEINPUT, KEYBDINPUT, HARDWAREINPUT with type constants
455 " 🔵 KEYBDINPUT structure with virtual key codes and event flags for keyboard simulation
456 " 🔵 SendInput function linked from user32.dll for keyboard input simulation
457 1:55p 🔵 GetAsyncKeyState function for keyboard polling fallback (VK monitoring)
458 " 🔵 Tauri 2.11.2 provides built-in TrayIconBuilder with menu event handlers
459 " 🔵 Tauri Builder.on_menu_event() example: event.id() matching and app.exit(0) pattern
461 " 🔵 TrayIconBuilder API: with_id(), menu(), icon(), tooltip(), title(), show_menu_on_left_click()
462 1:56p 🔵 Tauri WindowEvent::CloseRequested with CloseRequestApi to prevent close and minimize to tray
465 2:00p 🔵 Tauri Rust build blocked by crates.io network inaccessibility
466 2:04p 🔵 Tauri build requires missing Windows application icon
467 " ✅ Generated Windows application icon for Tauri app
470 " 🔴 Generated Windows icon resource for Tauri build
468 2:05p 🔵 Compilation errors in Tauri app source code
469 " 🔴 Fixed Rust compilation errors in main.rs
471 " 🟣 Tauri read-this application successfully compiles
S207 Build Tauri Windows app after resolving icon resource requirement (May 24, 2:06 PM)
S208 Build and finalize configuration for Read This Tauri v2 desktop app with global JS API integration (May 24, 2:06 PM)
472 2:10p 🟣 Tauri desktop application compiled with text-to-speech and global hotkey support
473 2:11p ✅ Tauri configuration and documentation updated for new app architecture
474 " 🟣 Tauri v2 Desktop App Build Successful with Global JS API
475 " ✅ README Updated to Reflect Tauri v2 Migration
476 " 🔵 Configuration update validated with successful compile check
S209 Build a text-to-speech reader tool that reads highlighted text aloud when triggered via hotkey, with voice/speed/volume controls (May 24, 2:12 PM)
S211 Fix Tauri app crash from duplicate global shortcut registration (HotKey already registered panic) (May 24, 2:12 PM)
477 2:16p 🔴 Resolve HotKey registration panic with deferred shortcut registration
478 2:17p 🔴 Tauri hotkey collision crash → graceful fallback with keyboard polling
S210 Fix Tauri Read This desktop app hotkey collision crash and verify the solution compiles (May 24, 2:17 PM)
488 2:40p ✅ Vendored msedge-tts crate locally
489 2:41p 🔵 Edge TTS authentication requires Sec-MS-GEC security token
490 " 🔵 WebSocket connection requires specific headers and certificate verification
491 " 🔴 Fixed Edge TTS 403 Forbidden by adding MUID cookie and updating user agent
492 2:42p ✅ Edge TTS patches verified and compiled successfully
493 2:43p 🟣 Created Edge TTS probe example for end-to-end testing
494 2:44p 🔵 Edge TTS still returns 403 Forbidden despite patches
495 " 🔵 Installed Python edge-tts client for diagnostic testing
496 2:45p 🔵 Python edge-tts client succeeds; Rust crate fails with 403
497 " 🔵 Python edge-tts reveals missing headers in Rust crate implementation
498 2:46p ✅ Implemented Python/Rust fallback strategy for Edge TTS synthesis
499 " ✅ Fallback strategy compiled and verified
500 2:48p 🔵 Build failed due to locked executable file
### May 26, 2026
578 3:44p 🟣 Read This: Windows Text-to-Speech Tray Application
579 3:45p ✅ GitHub Actions Workflow: Improved PowerShell Bat File Generation

Access 215k tokens of past work via get_observations([IDs]) or mem-search skill.
</claude-mem-context>