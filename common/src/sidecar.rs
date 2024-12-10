use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufWriter, Read, Seek, SeekFrom, Write},
    path::Path,
};

use anyhow::{Context, Result};

use crate::winapi::ensure_hidden;

pub struct Sidecar {
    file: File,
    lines: Vec<String>,
}

impl Sidecar {
    pub fn new(mut file: File) -> Result<Self> {
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        let lines = data.lines().map(|x| x.unwrap()).collect();
        Ok(Self { lines, file })
    }

    pub fn rewrite(&mut self) -> Result<()> {
        self.file.set_len(0)?;
        self.file.seek(SeekFrom::Start(0))?;

        let mut writer = BufWriter::new(&self.file);
        for line in &self.lines {
            writer.write_all(line.as_bytes())?;
            writer.write_all(b"\n")?;
        }

        writer.flush()?;
        Ok(())
    }

    pub fn add(&mut self, file: &str) -> Result<()> {
        if self.lines.contains(&file.to_string()) {
            return Ok(());
        }

        self.lines.push(file.to_string());

        self.file.seek(SeekFrom::End(0))?;
        self.file.write_all(file.as_bytes())?;
        self.file.write_all(b"\n")?;
        Ok(())
    }

    pub fn remove(&mut self, file: &str) -> Result<()> {
        self.lines.retain(|x| x != file);
        self.rewrite()?;
        Ok(())
    }
}

pub fn open_sidecar(path: &Path) -> Option<Result<File>> {
    let sidecar = path.parent()?.join(".watched");
    let _ = ensure_hidden(&sidecar);

    sidecar.exists().then(|| {
        match OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(sidecar)
        {
            Ok(file) => Ok(file),
            Err(err) => Err(err.into()),
        }
    })
}

pub fn open_or_create_sidecar(path: &Path) -> Result<File> {
    let sidecar = path
        .parent()
        .context("Can't open sidecar for root directory")?
        .join(".watched");
    let _ = ensure_hidden(&sidecar);

    Ok(OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(sidecar)?)
}
