use anyhow::Result;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
};
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

use crate::pipeline::context::ContextDecision;

const MAX_NAME_LEN: usize = 260;

const DEFAULT_TERMINAL_PROCESS_NAMES: &[&str] = &[
    "WindowsTerminal.exe",
    "powershell.exe",
    "pwsh.exe",
    "cmd.exe",
    "wezterm-gui.exe",
    "alacritty.exe",
    "conhost.exe",
    "putty.exe",
    "kitty.exe",
];

pub fn detect() -> Result<ContextDecision> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd == HWND::default() {
        return Ok(ContextDecision::unknown());
    }

    let mut process_id: u32 = 0;
    let _ = unsafe { windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId(hwnd, Some(&mut process_id)) };

    if process_id == 0 {
        return Ok(ContextDecision::unknown());
    }

    let process_name = get_process_name(process_id).unwrap_or_else(|_| String::from("unknown"));

    let is_terminal = DEFAULT_TERMINAL_PROCESS_NAMES
        .iter()
        .any(|name| process_name.eq_ignore_ascii_case(name));

    let confidence = if is_terminal { 0.85 } else { 0.0 };

    Ok(ContextDecision {
        is_terminal,
        confidence,
        process_name: Some(process_name),
        window_title: None,
        matched_signature: if is_terminal { Some("process-name".into()) } else { None },
    })
}

fn get_process_name(pid: u32) -> Result<String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid)?;
        if handle.is_invalid() {
            return Ok("unknown".to_string());
        }

        let mut name = [0u16; MAX_NAME_LEN];
        let _ = windows::Win32::System::ProcessStatus::GetModuleBaseNameW(handle, None, &mut name);
        let name_str = String::from_utf16_lossy(&name);
        let trimmed = name_str.trim_end_matches('\0');
        Ok(trimmed.to_string())
    }
}