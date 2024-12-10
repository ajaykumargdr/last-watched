use std::iter;

use windows::Win32::{
    Foundation::{HINSTANCE, MAX_PATH},
    System::LibraryLoader::GetModuleFileNameW,
};

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {{
        let msg = $crate::misc::to_pcstr(&format!("[{}:{}] {}", module_path!(), line!(), format!($($arg)*)));
        unsafe { windows::Win32::System::Diagnostics::Debug::OutputDebugStringA(windows::core::PCSTR(msg.as_ptr())) };
    }};
}

pub unsafe fn get_module_path(instance: HINSTANCE) -> String {
    let mut path = [0u16; MAX_PATH as usize];
    let len = GetModuleFileNameW(instance, &mut path);
    String::from_utf16_lossy(&path[..len as usize])
}

pub fn to_pcstr(s: &str) -> Vec<u8> {
    s.bytes().chain(iter::once(0)).collect()
}

pub fn to_pcwstr(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(iter::once(0)).collect()
}
