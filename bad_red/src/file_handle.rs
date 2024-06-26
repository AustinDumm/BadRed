use std::{fs::{File, OpenOptions}, io::{Read, Write}, path::Path};


pub struct FileHandle {
    file: File,
    pub path: Box<Path>
}

impl Read for FileHandle {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.file.read(buf)
    }
}

pub trait FileWrite {
    fn write_file(&mut self, buf: &[u8]) -> std::io::Result<()>;
}

impl FileWrite for FileHandle {
    fn write_file(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.path)?;

        self.file.write_all(buf)?;
        self.file.flush()
    }
}

impl FileHandle {
    pub fn new(path: &Path) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        Ok(Self {
            file,
            path: Box::from(path),
        })
    }
}
