use std::{cell::RefCell, fs::File, path::Path};

use common::sidecar::Sidecar;
use windows::Win32::{
    Foundation::{BOOL, ERROR_FILE_NOT_FOUND, ERROR_NOT_FOUND},
    System::Com::{IClassFactory, IClassFactory_Impl},
    UI::Shell::PropertiesSystem::{
        IInitializeWithFile, IInitializeWithFile_Impl, IPropertyStore, IPropertyStore_Impl,
        PROPERTYKEY,
    },
};
use windows_core::{
    implement, IInspectable, IUnknown, Interface, Result, GUID, PCWSTR, PROPVARIANT,
};
use winreg::{
    enums::{HKEY_CLASSES_ROOT, KEY_ALL_ACCESS},
    RegKey,
};

use crate::{
    log,
    registry::{format_guid, register_clsid, unregister_clsid},
};

// {1c49d817-ff89-46d2-b25f-5be1cedb338a}
pub const PROPERTY_STORE_CLSID: GUID = GUID::from_u128(0x1c49d817_ff89_46d2_b25f_5be1cedb338a);
// {1c49d817-ff89-46d2-b25f-5be1cedb338a}
pub const WATCHED_PROPERTY: GUID = GUID::from_u128(0x1c49d817_ff89_46d2_b25f_5be1cedb338a);

#[implement(IPropertyStore, IInitializeWithFile)]
pub struct WatchedPropertyStore {
    sidecar: RefCell<Option<Sidecar>>,
}

#[implement(IClassFactory)]
pub struct WatchedPropertyStoreFactory;

impl IPropertyStore_Impl for WatchedPropertyStore_Impl {
    fn GetCount(&self) -> Result<u32> {
        log!("GetCount");
        Ok(1)
    }

    fn GetAt(&self, iprop: u32, _pkey: *mut PROPERTYKEY) -> Result<()> {
        log!("GetAt: {iprop}");
        Ok(())
    }

    fn GetValue(&self, key: *const PROPERTYKEY) -> Result<PROPVARIANT> {
        log!("GetValue");
        if unsafe { (*key).fmtid } != WATCHED_PROPERTY {
            return Err(ERROR_NOT_FOUND.into());
        }

        Ok(true.into())
    }

    fn SetValue(&self, _key: *const PROPERTYKEY, _propvar: *const PROPVARIANT) -> Result<()> {
        log!("SetValue");
        Ok(())
    }

    fn Commit(&self) -> Result<()> {
        log!("Commit");
        Ok(())
    }
}

impl IInitializeWithFile_Impl for WatchedPropertyStore_Impl {
    fn Initialize(&self, pszfilepath: &PCWSTR, _grfmode: u32) -> Result<()> {
        let path_string = unsafe { pszfilepath.to_string() }?;
        log!("Initialize: {path_string:?}");

        let path = Path::new(&path_string);
        let sidecar_path = path.parent().unwrap().join(".watched");

        let Ok(sidecar) = Sidecar::new(File::open(sidecar_path)?) else {
            return Err(ERROR_FILE_NOT_FOUND.into());
        };

        self.sidecar.replace(Some(sidecar));
        Ok(())
    }
}

impl IClassFactory_Impl for WatchedPropertyStoreFactory_Impl {
    fn CreateInstance(
        &self,
        _punkouter: Option<&IUnknown>,
        riid: *const windows_core::GUID,
        ppvobject: *mut *mut core::ffi::c_void,
    ) -> Result<()> {
        let obj = IInspectable::from(WatchedPropertyStore {
            sidecar: RefCell::new(None),
        });
        unsafe { obj.query(riid, ppvobject).ok() }
    }

    fn LockServer(&self, _flock: BOOL) -> windows_core::Result<()> {
        Ok(())
    }
}

pub fn register(module_path: &str) -> anyhow::Result<()> {
    let inproc =
        register_clsid(module_path, PROPERTY_STORE_CLSID)?.open_subkey("InProcServer32")?;
    inproc.set_value("DisableProcessIsolation", &1u32)?;

    let associations = RegKey::predef(HKEY_CLASSES_ROOT).open_subkey("SystemFileAssociations")?;
    for format in [".mkv"] {
        let (key, _) = associations
            .create_subkey(format!(r"{format}\ShellEx\ContextMenuHandlers\LastWatched"))?;
        key.set_value("", &format_guid(&PROPERTY_STORE_CLSID))?;
    }

    Ok(())
}

pub fn unregister() -> anyhow::Result<()> {
    unregister_clsid(PROPERTY_STORE_CLSID)?;

    let associations = RegKey::predef(HKEY_CLASSES_ROOT).open_subkey("SystemFileAssociations")?;
    for format in [".mkv"] {
        let key = associations.open_subkey_with_flags(
            format!(r"{format}\ShellEx\ContextMenuHandlers"),
            KEY_ALL_ACCESS,
        )?;
        key.delete_subkey_all("LastWatched")?;
    }

    Ok(())
}
