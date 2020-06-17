use core::fmt;

use crate::hal::io::{Read, Write};

#[derive(Copy, Clone)]
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

#[derive(Copy, Clone, Debug)]
pub enum Error {
    BadSchema,
    NoMedia,
    NotFound,
    InsufficentResource,
    EndOfFile,
    Generic,
}

pub struct FileDescriptor(pub usize);

#[derive(Copy, Clone)]
pub struct Media {
    pub open: fn(path: &str, options: OpenOptions) -> Result<FileDescriptor, Error>,
    pub close: fn(fd: FileDescriptor),
    pub read: fn(fd: &FileDescriptor, buf: &mut [u8]) -> Result<usize, Error>,
    pub write: fn(fd: &FileDescriptor, bytes: &[u8]) -> Result<usize, Error>,
}

#[derive(Copy, Clone, Debug)]
pub enum Schema {
    Flash = 0,
    Sdcard = 1,
}

fn no_open(_: &str, _: OpenOptions) -> Result<FileDescriptor, Error> {
    Err(Error::NotFound)
}
fn no_close(_: FileDescriptor) {}
fn no_read(_: &FileDescriptor, _: &mut [u8]) -> Result<usize, Error> {
    Ok(0)
}
fn no_write(_: &FileDescriptor, _: &[u8]) -> Result<usize, Error> {
    Ok(0)
}

macro_rules! no_media {
    () => {
        Media { open: no_open, close: no_close, read: no_read, write: no_write }
    };
}

impl Default for Media {
    fn default() -> Self {
        no_media!()
    }
}

static mut MEDIAS: [Media; 2] = [no_media!(), no_media!()];

pub struct File {
    schema: Schema,
    fd: Option<FileDescriptor>,
}

impl File {
    pub fn open(path: &str) -> Result<File, Error> {
        OpenOptions::default().open(path)
    }

    pub fn close(&mut self) {
        let medias = unsafe { &MEDIAS };
        if let Some(fd) = self.fd.take() {
            (medias[self.schema as usize].close)(fd)
        }
    }
}

impl Read for File {
    type Error = Error;
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let medias = unsafe { &MEDIAS };
        if let Some(fd) = &self.fd {
            (medias[self.schema as usize].read)(&fd, buf)
        } else {
            Ok(0)
        }
    }
}

impl Write for File {
    type Error = Error;
    fn write(&mut self, bytes: &[u8]) -> Result<usize, Error> {
        let medias = unsafe { &MEDIAS };
        if let Some(fd) = &self.fd {
            (medias[self.schema as usize].write)(&fd, bytes)
        } else {
            Ok(0)
        }
    }
}

impl fmt::Write for File {
    fn write_char(&mut self, c: char) -> fmt::Result {
        let medias = unsafe { &MEDIAS };
        if let Some(fd) = &self.fd {
            match (medias[self.schema as usize].write)(&fd, &[c as u8]) {
                Ok(_) => Ok(()),
                Err(_) => Err(fmt::Error),
            }
        } else {
            Err(fmt::Error)
        }
    }

    fn write_str(&mut self, s: &str) -> fmt::Result {
        if let Some(fd) = &self.fd {
            let medias = unsafe { &MEDIAS };
            let write = medias[self.schema as usize].write;
            (write)(&fd, s.as_bytes()).map_err(|_| fmt::Error)?;
            Ok(())
        } else {
            Err(fmt::Error)
        }
    }
}

impl OpenOptions {
    pub fn open(self, path: &str) -> Result<File, Error> {
        let medias = unsafe { &MEDIAS };
        if path.starts_with("flash://") {
            let option = (medias[Schema::Flash as usize].open)(&path[8..], self);
            option.map(|fd| File { schema: Schema::Flash, fd: Some(fd) })
        } else if path.starts_with("sdcard://") {
            let option = (medias[Schema::Sdcard as usize].open)(&path[9..], self);
            option.map(|fd| File { schema: Schema::Sdcard, fd: Some(fd) })
        } else {
            Err(Error::BadSchema)
        }
    }
}

pub fn set_media(schema: Schema, media: Media) {
    unsafe { MEDIAS[schema as usize] = media }
}
