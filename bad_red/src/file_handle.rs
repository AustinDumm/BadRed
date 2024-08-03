use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
};

pub struct FileHandle {
    file: File,
    pub path: Box<str>,
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
            .open(Path::new(self.path.as_ref()))?;

        self.file.write_all(buf)?;
        self.file.flush()
    }
}

impl FileHandle {
    pub fn new(path_str: String) -> std::io::Result<Self> {
        let expanded_path = shellexpand::full(&path_str).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to expand file path due to: {}", e.var_name),
            )
        })?.into_owned();
        let path = Path::new(&expanded_path);

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        Ok(Self {
            file,
            path: expanded_path.into_boxed_str(),
        })
    }
}
