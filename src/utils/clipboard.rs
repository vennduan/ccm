// Clipboard operations - Cross-platform clipboard support

use std::process::Command;

/// Copy text to clipboard
/// Returns true if successful, false otherwise
pub fn copy_to_clipboard(text: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        copy_windows(text)
    }

    #[cfg(target_os = "macos")]
    {
        copy_macos(text)
    }

    #[cfg(target_os = "linux")]
    {
        copy_linux(text)
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        false
    }
}

/// Windows: Use PowerShell Set-Clipboard
#[cfg(target_os = "windows")]
fn copy_windows(text: &str) -> bool {
    // Escape special characters for PowerShell
    let escaped = text.replace("'", "''").replace("$", "`$");

    let result = Command::new("powershell")
        .args(["-Command", &format!("Set-Clipboard -Value '{}'", escaped)])
        .output();

    match result {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// macOS: Use pbcopy
#[cfg(target_os = "macos")]
fn copy_macos(text: &str) -> bool {
    use std::io::Write;
    use std::process::Stdio;

    let child = Command::new("pbcopy").stdin(Stdio::piped()).spawn();

    match child {
        Ok(mut process) => {
            if let Some(mut stdin) = process.stdin.take() {
                if stdin.write_all(text.as_bytes()).is_err() {
                    return false;
                }
            }
            match process.wait() {
                Ok(status) => status.success(),
                Err(_) => false,
            }
        }
        Err(_) => false,
    }
}

/// Linux: Try xclip first, then xsel
#[cfg(target_os = "linux")]
fn copy_linux(text: &str) -> bool {
    use std::io::Write;
    use std::process::Stdio;

    // Try xclip first
    let xclip = Command::new("xclip")
        .args(["-selection", "clipboard"])
        .stdin(Stdio::piped())
        .spawn();

    if let Ok(mut process) = xclip {
        if let Some(mut stdin) = process.stdin.take() {
            if stdin.write_all(text.as_bytes()).is_ok() {
                if let Ok(status) = process.wait() {
                    if status.success() {
                        return true;
                    }
                }
            }
        }
    }

    // Fall back to xsel
    let xsel = Command::new("xsel")
        .args(["--clipboard", "--input"])
        .stdin(Stdio::piped())
        .spawn();

    if let Ok(mut process) = xsel {
        if let Some(mut stdin) = process.stdin.take() {
            if stdin.write_all(text.as_bytes()).is_ok() {
                if let Ok(status) = process.wait() {
                    return status.success();
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_to_clipboard() {
        // This test will fail in CI environments without clipboard access
        // So we just test that the function doesn't panic
        let _ = copy_to_clipboard("test");
    }
}
