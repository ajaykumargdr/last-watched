pub mod context_menu;
pub mod icon_overlay;
pub mod property_store;
// pub mod drive_client;

pub fn notify(msg: &str) {

    let msg = format!("::{msg}::");
    unsafe {
        windows::Win32::UI::WindowsAndMessaging::MessageBoxA(
            windows::Win32::Foundation::HWND::default(),
            windows_core::PCSTR( (msg).as_ptr()),
            windows_core::s!("last-watched"),
            Default::default(),
        );
    };
}
