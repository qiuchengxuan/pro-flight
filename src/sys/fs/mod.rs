mod error;
mod exfat;
mod media;

use core::fmt;

use error::Error;
use media::{FileDescriptor, Media, NoMedia};

use crate::io::{Read, Write};

#[derive(Copy, Clone, Debug)]
pub struct OpenOptions {
    pub read: bool,
    pub write: bool,
    pub create: bool,
    pub append: bool,
    pub truncate: bool,
}

impl Default for OpenOptions {
    fn default() -> Self {
        Self { read: true, write: false, create: false, append: false, truncate: false }
    }
}

impl OpenOptions {
    pub fn read(mut self, b: bool) -> Self {
        self.read = b;
        self
    }

    pub fn write(mut self, b: bool) -> Self {
        self.write = b;
        self
    }

    pub fn create(mut self, b: bool) -> Self {
        self.create = b;
        self
    }

    pub fn append(mut self, b: bool) -> Self {
        self.append = b;
        self
    }

    pub fn truncate(mut self, b: bool) -> Self {
        self.truncate = b;
        self
    }
}

static mut SCHEMAS: [&dyn Media; 2] = [&NoMedia {}, &NoMedia {}];

#[derive(Copy, Clone, Debug)]
pub enum Schema {
    Flash = 0,
    Sdcard = 1,
}

fn get_media(schema: Schema) -> &'static dyn Media {
    unsafe { SCHEMAS[schema as usize] }
}

pub struct Metadata {
    pub length: u64,
}

impl Metadata {
    pub fn len(&self) -> u64 {
        self.length
    }
}

pub struct File {
    schema: Schema,
    fd: Option<FileDescriptor>,
}

impl File {
    pub fn open(path: &str) -> Result<File, Error> {
        OpenOptions::default().open(path)
    }

    pub fn metadata(&self) -> Result<Metadata, Error> {
        if let Some(ref fd) = self.fd {
            return get_media(self.schema).metadata(fd);
        }
        Err(Error::Generic)
    }

    pub fn close(&mut self) {
        if let Some(fd) = self.fd.take() {
            get_media(self.schema).close(fd)
        }
    }
}

impl Read for File {
    type Error = Error;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        if let Some(fd) = self.fd.as_ref() { get_media(self.schema).read(fd, buf) } else { Ok(0) }
    }
}

impl Write for File {
    type Error = Error;

    fn write(&mut self, bytes: &[u8]) -> Result<usize, Error> {
        if let Some(fd) = self.fd.as_ref() {
            get_media(self.schema).write(fd, bytes)
        } else {
            Ok(0)
        }
    }

    fn flush(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

impl fmt::Write for File {
    fn write_char(&mut self, c: char) -> fmt::Result {
        if let Some(fd) = self.fd.as_ref() {
            let mut bytes = [0u8; 2];
            match get_media(self.schema).write(fd, c.encode_utf8(&mut bytes).as_bytes()) {
                Ok(_) => Ok(()),
                Err(_) => Err(fmt::Error),
            }
        } else {
            Err(fmt::Error)
        }
    }

    fn write_str(&mut self, s: &str) -> fmt::Result {
        if let Some(fd) = self.fd.as_ref() {
            let result = get_media(self.schema).write(fd, s.as_bytes());
            result.map(|_| ()).map_err(|_| fmt::Error)
        } else {
            Err(fmt::Error)
        }
    }
}

impl OpenOptions {
    pub fn open(self, path: &str) -> Result<File, Error> {
        let (schema, path) = if path.starts_with("flash://") {
            (Schema::Flash, &path[8..])
        } else if path.starts_with("sdcard://") {
            (Schema::Sdcard, &path[9..])
        } else {
            return Err(Error::BadSchema);
        };
        let result = get_media(schema).open(path, self);
        result.map(|fd| File { schema, fd: Some(fd) })
    }
}

pub fn set_media(schema: Schema, media: &'static dyn Media) {
    unsafe { SCHEMAS[schema as usize] = media }
}
