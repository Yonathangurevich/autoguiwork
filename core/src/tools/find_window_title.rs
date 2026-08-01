use windows::Win32::Foundation::{HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible,
};
use windows::core::BOOL;
use crate::models::engine_error::EngineErrorKind;
use crate::tools::find_image::sleep_for_f64;

/// Collect the titles of all currently visible top-level windows.
///
/// Windows has no "give me all titles" call — instead EnumWindows calls a
/// callback once per window. We pass a pointer to our Vec through `lparam`
/// so the callback can push each title into it.
fn all_window_titles() -> Vec<String> {
    let mut titles: Vec<String> = Vec::new();

    unsafe {
        // Pass the Vec's address to the callback as the lparam.
        let ptr = &mut titles as *mut Vec<String> as isize;
        let _ = EnumWindows(Some(enum_callback), LPARAM(ptr));
    }

    titles
}

/// Called by Windows once per top-level window. Returning TRUE means "keep going".
extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        // Recover our Vec from the pointer we passed in.
        let titles = &mut *(lparam.0 as *mut Vec<String>);

        // Skip invisible/background windows — we only care about real ones.
        if !IsWindowVisible(hwnd).as_bool() {
            return BOOL(1);
        }

        let len = GetWindowTextLengthW(hwnd);
        if len > 0 {
            // GetWindowTextW writes UTF-16 into a buffer; +1 for the null terminator.
            let mut buf = vec![0u16; (len + 1) as usize];
            let written = GetWindowTextW(hwnd, &mut buf);
            if written > 0 {
                let title = String::from_utf16_lossy(&buf[..written as usize]);
                titles.push(title);
            }
        }
    }

    BOOL(1) // keep enumerating (1 = TRUE)
}

/// True if any visible window's title contains `substring`.
/// Substring match (not exact) so "Inbox (3) - Outlook" still matches "Outlook".
pub fn is_window_open(substring: &str) -> bool {
    all_window_titles()
        .iter()
        .any(|title| title.contains(substring))
}

/// Wait for a window whose title contains `substring` to appear.
///
/// This snapshots the titles that exist *before* waiting, then only accepts a
/// match that is genuinely NEW — so if a matching window was already open, we
/// wait for a fresh one rather than returning instantly on the stale one.
/// Gives up after `timeout_ms` with a WindowNotFound error.
pub fn wait_for_new_window(substring: &str, timeout_ms: f64) -> Result<(), EngineErrorKind> {
    const POLL_MS: f64 = 200.0;

    // Snapshot: which matching windows already existed when we started?
    let already_open: Vec<String> = all_window_titles()
        .into_iter()
        .filter(|t| t.contains(substring))
        .collect();

    let mut remaining = timeout_ms;

    loop {
        let matches: Vec<String> = all_window_titles()
            .into_iter()
            .filter(|t| t.contains(substring))
            .collect();

        // Accept a match only if it wasn't in the original snapshot.
        if matches.iter().any(|t| !already_open.contains(t)) {
            return Ok(());
        }

        if remaining <= 0.0 {
            return Err(EngineErrorKind::WindowNotFound {
                title: substring.to_string(),
                waited_ms: timeout_ms,
            });
        }

        sleep_for_f64(POLL_MS / 1000.0);
        remaining -= POLL_MS;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lists_some_windows() {
        // On any running desktop there should be at least one titled window.
        let titles = all_window_titles();
        println!("open windows: {:?}", titles);
        assert!(!titles.is_empty());
    }
}
