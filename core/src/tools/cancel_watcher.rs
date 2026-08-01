//! Watches for the panic key (Esc) and flips a shared cancel flag.
//!
//! The automation runs on the main thread, fully busy clicking, so it can't
//! also watch the keyboard. This spawns a small background thread that polls
//! for Esc and — when pressed — sets the shared Arc<AtomicBool> the runner
//! checks between actions.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_ESCAPE};

/// Is the Escape key down *right now*? Uses GetAsyncKeyState, which reports the
/// physical key state globally — it works even when our app isn't the focused
/// window, which is exactly what we need (the automation is clicking in OTHER
/// apps). The high bit (0x8000) of the returned value means "currently down".
fn is_escape_down() -> bool {
    unsafe {
        let state = GetAsyncKeyState(VK_ESCAPE.0 as i32);
        (state as u16 & 0x8000) != 0
    }
}

/// Spawn a background thread that sets `cancel` to true as soon as Esc is
/// pressed, then exits. Poll every ~50ms — responsive to a keypress without
/// burning CPU.
///
/// TODO (your part): implement the body.
///   1. thread::spawn(move || { ... })  — `move` so the closure OWNS its Arc clone
///   2. inside, loop:
///        - if is_escape_down() { cancel.store(true, Ordering::Relaxed); break; }
///        - if cancel.load(Ordering::Relaxed) { break; }  // run ended some other way
///        - thread::sleep(Duration::from_millis(50));
///
/// Note: you pass in a CLONE of the flag (ctx.cancel_handle()), so this thread
/// and the runner share the same underlying bool.
pub fn spawn_escape_watcher(cancel: Arc<AtomicBool>) {
    // your implementation here
    thread::spawn(move || {
        loop {
            if is_escape_down() {
                cancel.store(true, Ordering::Relaxed);
                break;
            }
            if cancel.load(Ordering::Relaxed) {
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }
    });
}
