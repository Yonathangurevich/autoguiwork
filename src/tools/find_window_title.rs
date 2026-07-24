use windows::Win32::UI::WindowsAndMessaging::FindWindowW;
use windows::core::HSTRING;

pub fn is_window_open(title: &str) -> bool {
    let hwnd = unsafe { FindWindowW(None, &HSTRING::from(title)) };

    match hwnd {
        Ok(_r) => return true,
        Err(_) => return false,
    };
}

#[cfg(test)]
mod tests {
    use crate::tools::find_window_title::is_window_open;

    #[test]
    fn test_windows_checking_if_window_is_up() {
        assert!(is_window_open("sd.docx - Word"));
    }
}
