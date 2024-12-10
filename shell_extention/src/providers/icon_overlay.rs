use server::drive_client::DriveClient;
use server::drive_server::{Drive, DriveServer};
use std::ptr::null_mut;
use std::{ffi::c_void, fs, iter, os::windows::ffi::OsStrExt, path::Path};
use widestring::{utf16str, Utf16Str};
use windows::Win32::Foundation::{E_NOINTERFACE, E_UNEXPECTED};
use windows::{
    core::{implement, Error, Result, PCWSTR, PWSTR},
    Win32::{
        Foundation::{BOOL, ERROR_INSUFFICIENT_BUFFER, S_FALSE},
        System::Com::{IClassFactory, IClassFactory_Impl},
        UI::Shell::{
            IShellIconOverlayIdentifier, IShellIconOverlayIdentifier_Impl, SHChangeNotify,
            ISIOI_ICONFILE, ISIOI_ICONINDEX, SHCNE_ASSOCCHANGED, SHCNF_IDLIST,
        },
    },
};
use windows_core::{IInspectable, IUnknown, Interface, GUID};
use winreg::{
    enums::{HKEY_LOCAL_MACHINE, KEY_ALL_ACCESS},
    RegKey,
};
// use server::{Icon, Path};
use tonic::{transport::Server, Request, Response};

mod server {
    tonic::include_proto!("server");
}

use crate::{
    log,
    misc::get_module_path,
    providers::notify,
    registry::{format_guid, register_clsid, unregister_clsid},
    INSTANCE,
};
use common::{winapi::ensure_hidden, VIDEO_EXTENSIONS};

// {172d5af2-6916-48d3-a611-368273076434}
pub const OVERLAY_CLSID: GUID = GUID::from_u128(0x172d5af2_6916_48d3_a611_368273076434);

#[implement(IShellIconOverlayIdentifier)]
pub struct WatchedOverlay {
    rt: tokio::runtime::Runtime,
    client: Option<DriveClient<tonic::transport::Channel>>,
}

#[implement(IClassFactory)]
pub struct WatchedOverlayFactory;

enum IsMemberOfResult {
    Member,
    NotMember,
}

impl IShellIconOverlayIdentifier_Impl for WatchedOverlay_Impl {
    fn IsMemberOf(&self, pwszpath: &PCWSTR, _dwattrib: u32) -> Result<()> {
        let path = unsafe { pwszpath.to_string()? };
        // notify(&format!("imof:{}", path));
        let mut client = match self.client.clone() {
            Some(client) => client,
            None => return IsMemberOfResult::NotMember.into(),
        };

        let response = self.rt.block_on(async {
            let request = Request::new(server::Path { name: path.clone() });

            client.overlay_icon(request).await
        });

        if let Ok(response) = response {
            if response.get_ref().kind == "in_sync" {
                return IsMemberOfResult::Member.into();
            }
        }

        IsMemberOfResult::NotMember.into()
    }

    fn GetOverlayInfo(
        &self,
        pwsziconfile: PWSTR,
        cchmax: i32,
        pindex: *mut i32,
        pdwflags: *mut u32,
    ) -> Result<()> {
        log!("GetOverlayInfo");
        notify("GetOverlayInfo");

        let icon = Path::new(&unsafe { get_module_path(INSTANCE) })
            .parent()
            .unwrap()
            .join("icon.ico");

        // notify(icon.to_str().unwrap());

        // let icon = Path::new(r#"D:\icon.ico"#);

        let icon = icon
            .as_os_str()
            .encode_wide()
            .chain(iter::once(0))
            .collect::<Vec<_>>();

        unsafe {
            if cchmax < icon.len() as i32 + 1 {
                notify("innsufficent buffer!");
                return Err(Error::new(
                    ERROR_INSUFFICIENT_BUFFER.into(),
                    "Icon path too long",
                ));
            }

            *pindex = 0i32;
            *pdwflags = ISIOI_ICONFILE;

            notify("copying file");
            pwsziconfile.as_ptr().copy_from(icon.as_ptr(), icon.len());
        }

        /*
        const ICON_PATH: &Utf16Str = utf16str!(r#"D:\icon.ico"#);
        unsafe {
            pwsziconfile
                .as_ptr()
                .copy_from(ICON_PATH.as_ptr(), ICON_PATH.len() + 1);
            *pindex = 0i32;
            *pdwflags = ISIOI_ICONFILE;
        }
        */

        notify("info created!");
        Ok(())
    }

    fn GetPriority(&self) -> Result<i32> {
        Ok(0)
    }
}

impl IClassFactory_Impl for WatchedOverlayFactory_Impl {
    fn CreateInstance(
        &self,
        _punkouter: Option<&IUnknown>,
        riid: *const GUID,
        ppvobject: *mut *mut c_void,
    ) -> Result<()> {
        notify("create instance");
        let rt = tokio::runtime::Runtime::new().expect("not able to create runtime!");

        let client = rt.block_on(async { DriveClient::connect("http://[::1]:50051").await });

        let obj = match client {
            Ok(client) => IInspectable::from(WatchedOverlay {
                rt,
                client: Some(client),
            }),
            Err(e) => {
                IInspectable::from(WatchedOverlay { rt, client: None })

                // unsafe { *ppvobject = std::ptr::null_mut() };
                // return Err(Error::from_hresult(E_NOINTERFACE));
            }
        };

        unsafe { obj.query(riid, ppvobject).ok() }
    }

    fn LockServer(&self, _flock: BOOL) -> Result<()> {
        Ok(())
    }
}

impl From<IsMemberOfResult> for Result<()> {
    fn from(result: IsMemberOfResult) -> Result<()> {
        match result {
            IsMemberOfResult::Member => Ok(()),
            IsMemberOfResult::NotMember => Err(Error::from_hresult(S_FALSE)),
        }
    }
}

pub fn register(module_path: &str) -> anyhow::Result<()> {
    notify("registering");
    register_clsid(module_path, OVERLAY_CLSID)?;

    // notify("1. cls id registered!");

    let (overlays, _) = RegKey::predef(HKEY_LOCAL_MACHINE).create_subkey(
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\ShellIconOverlayIdentifiers\ LastWatched",
    )?;

    // notify("2. subkey created!");

    overlays.set_value("", &format_guid(&OVERLAY_CLSID))?;

    // notify("3. value is set!");

    unsafe {
        SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None);
    }

    notify("registration done!");
    Ok(())
}

pub fn unregister() -> anyhow::Result<()> {
    unregister_clsid(OVERLAY_CLSID)?;

    let overlays = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey_with_flags(
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\ShellIconOverlayIdentifiers",
        KEY_ALL_ACCESS,
    )?;
    overlays.delete_subkey_all("LastWatched")?;

    Ok(())
}

#[test]
fn test_resgister() {
    let module_path = unsafe { get_module_path(INSTANCE) };

    dbg!(&module_path);

    register(&module_path).expect("regristation failed!");
}
