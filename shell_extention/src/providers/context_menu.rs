use std::mem;

use windows::Win32::{
    Foundation::{BOOL, ERROR_NOT_FOUND, S_FALSE},
    System::{Com::{IClassFactory, IClassFactory_Impl, IDataObject}, Registry::HKEY},
    UI::{
        Shell::{
            Common::ITEMIDLIST, IContextMenu, IContextMenu_Impl, IShellExtInit, IShellExtInit_Impl, CMF_DEFAULTONLY, CMINVOKECOMMANDINFO, GCS_HELPTEXTA, GCS_HELPTEXTW, GCS_VALIDATEA, GCS_VALIDATEW, GCS_VERBA, GCS_VERBW
        },
        WindowsAndMessaging::{
            InsertMenuItemW, HMENU, MENUITEMINFOW, MFT_STRING, MIIM_FTYPE, MIIM_STATE, MIIM_STRING,
        },
    },
};
use windows_core::{
    implement, Error, IInspectable, IUnknown, Interface, Result, GUID, PSTR, PWSTR,
};
use winreg::{
    enums::{HKEY_CLASSES_ROOT, KEY_ALL_ACCESS},
    RegKey,
};

use crate::{
    log,
    misc::to_pcwstr,
    registry::{format_guid, register_clsid, unregister_clsid},
};

// {fc46d627-5ab0-452b-9262-38121078759e}
pub const CONTEXT_MENU_CLSID: GUID = GUID::from_u128(0xfc46d627_5ab0_452b_9262_38121078759e);

const ID_MARK_WATCHED: u32 = 0x0;
const ID_MARK_UNWATCHED: u32 = 0x1;

const MENU_ITEMS: [MenuItem; 2] = [
    MenuItem {
        id: ID_MARK_WATCHED,
        name: "mark-watched@",
        help_text: "Mark video as watched",
    },
    MenuItem {
        id: ID_MARK_UNWATCHED,
        name: "mark-unwatched@",
        help_text: "Mark video as not watched",
    },
];

struct MenuItem {
    id: u32,
    name: &'static str,
    help_text: &'static str,
}

#[implement(IContextMenu, IShellExtInit)]
pub struct WatchedContextMenu;

#[implement(IClassFactory)]
pub struct WatchedContextMenuFactory;

impl IContextMenu_Impl for WatchedContextMenu_Impl {
    fn QueryContextMenu(
        &self,
        hmenu: HMENU,
        _indexmenu: u32,
        _idcmdfirst: u32,
        _idcmdlast: u32,
        uflags: u32,
    ) -> Result<()> {
        if uflags & CMF_DEFAULTONLY != 0 {
            return Ok(());
        }

        log!("QueryContextMenu");
        
        for item in MENU_ITEMS.iter() {
            let menu_item = MENUITEMINFOW {
                cbSize: mem::size_of::<MENUITEMINFOW>() as u32,
                fMask: MIIM_FTYPE | MIIM_STRING | MIIM_STATE,
                fType: MFT_STRING,
                dwTypeData: PWSTR(to_pcwstr(item.help_text).as_mut_ptr()),
                cch: 2 * item.help_text.len() as u32,
                ..Default::default()
            };

            unsafe { InsertMenuItemW(hmenu, item.id, true, &menu_item)? };
        }

        Ok(())
    }

    fn InvokeCommand(&self, pici: *const CMINVOKECOMMANDINFO) -> Result<()> {
        let command_name = unsafe { (*pici).lpVerb.to_string().unwrap() };
        let command = MENU_ITEMS
            .iter()
            .find(|item| item.name == command_name)
            .ok_or(Error::from(ERROR_NOT_FOUND))?;

        let parameter = unsafe { (*pici).lpParameters.to_string().unwrap() };
        let directory = unsafe { (*pici).lpDirectory.to_string().unwrap() };
        log!("InvokeCommand: {} {parameter} {directory}", command.name,);

        Ok(())
    }

    fn GetCommandString(
        &self,
        idcmd: usize,
        utype: u32,
        _preserved: *const u32,
        pszname: PSTR,
        cchmax: u32,
    ) -> Result<()> {
        log!("GetCommandString");

        let item = MENU_ITEMS.get(idcmd).ok_or(Error::from(S_FALSE))?;

        let return_unicode = |string: &str| unsafe {
            let len = string.len().max(cchmax as usize - 1);
            pszname.0.copy_from(string.as_ptr(), len);
            pszname.0.add(len).write(0);
        };

        let return_ascii = |string: &str| unsafe {
            let string = string
                .chars()
                .map(|c| if c.is_ascii() { c as u8 } else { b'#' })
                .collect::<Vec<_>>();
            let len = string.len().max(cchmax as usize - 1);
            pszname.0.copy_from(string.as_ptr() as *const u8, len);
            pszname.0.add(len).write(0);
        };

        match utype {
            // Get command help text
            GCS_HELPTEXTA => return_ascii(item.help_text),
            GCS_HELPTEXTW => return_unicode(item.help_text),

            // Language independent command name
            GCS_VERBA => return_ascii(item.name),
            GCS_VERBW => return_unicode(item.name),

            // Check if menu item exists
            GCS_VALIDATEA | GCS_VALIDATEW => return Ok(()),

            // Unknown command
            _ => return Err(Error::from(ERROR_NOT_FOUND)),
        }

        Ok(())
    }
}

impl IShellExtInit_Impl for WatchedContextMenu_Impl {
    fn Initialize(
        &self,
        _pidl_folder: *const ITEMIDLIST, // something like NULL/ParentFolder
        _idata_obj: Option<&IDataObject>, // the item we click
        _hkey_prog_id: HKEY, // this the HKEY of the item so you can get more details of it from registory
    ) -> Result<()> {
        Ok(())
    }
}


impl IClassFactory_Impl for WatchedContextMenuFactory_Impl {
    fn CreateInstance(
        &self,
        _punkouter: Option<&IUnknown>,
        riid: *const windows_core::GUID,
        ppvobject: *mut *mut core::ffi::c_void,
    ) -> Result<()> {
        let obj = IInspectable::from(WatchedContextMenu);
        unsafe { obj.query(riid, ppvobject).ok() }
    }

    fn LockServer(&self, _flock: BOOL) -> windows_core::Result<()> {
        Ok(())
    }
}

pub fn register(module_path: &str) -> anyhow::Result<()> {
    register_clsid(module_path, CONTEXT_MENU_CLSID)?;

    let associations = RegKey::predef(HKEY_CLASSES_ROOT).open_subkey("SystemFileAssociations")?;
    for format in [".mkv"] {
        let (key, _) = associations
            .create_subkey(format!(r"{format}\ShellEx\ContextMenuHandlers\LastWatched"))?;
        key.set_value("", &format_guid(&CONTEXT_MENU_CLSID))?;
    }

    Ok(())
}

pub fn unregister() -> anyhow::Result<()> {
    unregister_clsid(CONTEXT_MENU_CLSID)?;

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
