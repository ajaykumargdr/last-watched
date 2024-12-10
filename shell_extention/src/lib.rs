#![allow(unused)]

use std::{ffi::c_void, panic, process};

use windows::Win32::{
    Foundation::{CLASS_E_CLASSNOTAVAILABLE, HANDLE, HINSTANCE, MAX_PATH, S_OK},
    System::{
        Com::IClassFactory,
        ProcessStatus::GetProcessImageFileNameA,
        SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
    },
    UI::Shell::{SHChangeNotify, SHCNE_ASSOCCHANGED, SHCNF_IDLIST},
};
use windows_core::{Interface, GUID, HRESULT};

mod misc;
mod providers;
mod registry;
use misc::get_module_path;
use providers::{
    context_menu::{self, WatchedContextMenuFactory, CONTEXT_MENU_CLSID},
    icon_overlay::{self, WatchedOverlayFactory, OVERLAY_CLSID},
    property_store::{self, WatchedPropertyStoreFactory, PROPERTY_STORE_CLSID},
};

static mut INSTANCE: HINSTANCE = HINSTANCE(0 as _);

#[no_mangle]
unsafe extern "system" fn DllMain(
    dll_module: HINSTANCE,
    call_reason: u32,
    _reserved: *mut (),
) -> bool {
    let mut buf = [0u8; MAX_PATH as usize];
    let len = GetProcessImageFileNameA(HANDLE(usize::MAX as *mut c_void), &mut buf);
    let name = String::from_utf8_lossy(&buf[..len as usize]);

    match call_reason {
        DLL_PROCESS_ATTACH => {
            log!("DLL Loaded by {name}");
            // providers::notify("DLL Loaded by {name}");
            
            INSTANCE = dll_module;
            panic::set_hook(Box::new(|info| {
                log!("== Panic ==");
                log!("{info}");
                process::abort();
            }));
        }
        DLL_PROCESS_DETACH => {
            log!("DLL Unloaded");
        }
        _ => return true,
    };

    true
}

#[no_mangle]
unsafe extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    log!("DllGetClassObject: {:?}", *rclsid);
    // providers::notify(&format!("DllGetClassObject: {:?}", *rclsid));
    
    match *rclsid {
        OVERLAY_CLSID => IClassFactory::from(WatchedOverlayFactory).query(riid, ppv),
        CONTEXT_MENU_CLSID => IClassFactory::from(WatchedContextMenuFactory).query(riid, ppv),
        PROPERTY_STORE_CLSID => IClassFactory::from(WatchedPropertyStoreFactory).query(riid, ppv),
        _ => CLASS_E_CLASSNOTAVAILABLE,
    }
}

#[no_mangle]
unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    S_OK
}

#[no_mangle]
unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    let module_path = get_module_path(INSTANCE);
    icon_overlay::register(&module_path).unwrap();
    // context_menu::register(&module_path).unwrap();
    // property_store::register(&module_path).unwrap();
    SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None);
    S_OK
}

#[no_mangle]
unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    icon_overlay::unregister().unwrap();
    context_menu::unregister().unwrap();
    property_store::unregister().unwrap();
    SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None);
    S_OK
}
