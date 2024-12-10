use anyhow::Result;
use windows_core::GUID;
use winreg::{
    enums::{HKEY_CLASSES_ROOT, KEY_ALL_ACCESS},
    RegKey,
};

pub fn register_clsid(module_path: &str, guid: GUID) -> Result<RegKey> {
    let clsid = RegKey::predef(HKEY_CLASSES_ROOT).open_subkey("CLSID")?;
    let (key, _) = clsid.create_subkey(&format_guid(&guid))?;
    key.set_value("", &"last-watched")?;

    let (inproc, _) = key.create_subkey("InProcServer32")?;
    inproc.set_value("", &module_path)?;
    inproc.set_value("ThreadingModel", &"Apartment")?;

    Ok(key)
}

pub fn unregister_clsid(guid: GUID) -> Result<()> {
    let clsid =
        RegKey::predef(HKEY_CLASSES_ROOT).open_subkey_with_flags("CLSID", KEY_ALL_ACCESS)?;
    clsid.delete_subkey_all(&format_guid(&guid))?;

    Ok(())
}

pub fn format_guid(guid: &GUID) -> String {
    format!("{{{guid:?}}}")
}
