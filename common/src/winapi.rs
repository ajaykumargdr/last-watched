use std::{iter, path::Path};

use anyhow::Result;

use windows::{
    core::PCWSTR,
    Win32::Storage::FileSystem::{
        GetFileAttributesW, SetFileAttributesW, FILE_ATTRIBUTE_HIDDEN, FILE_FLAGS_AND_ATTRIBUTES,
    },
};

pub fn ensure_hidden(path: &Path) -> Result<()> {
    let string = to_pcwstr(&path.to_string_lossy());
    let pcwstr = PCWSTR(string.as_ptr());

    let attributes = unsafe { GetFileAttributesW(pcwstr) };
    let is_hidden = attributes & FILE_ATTRIBUTE_HIDDEN.0 != 0;

    if !is_hidden {
        unsafe {
            SetFileAttributesW(
                pcwstr,
                FILE_FLAGS_AND_ATTRIBUTES(attributes) | FILE_ATTRIBUTE_HIDDEN,
            )?
        };
    }

    Ok(())
}

fn to_pcwstr(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(iter::once(0)).collect()
}
