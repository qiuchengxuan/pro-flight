use crate::hal::io::Read;

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

#[derive(Debug)]
pub struct FileDescriptor(pub usize);

#[derive(Copy, Clone)]
pub struct Media {
    pub open: fn(path: &str, options: OpenOptions) -> Result<FileDescriptor, Error>,
    pub close: fn(fd: FileDescriptor),
    pub read: fn(fd: &FileDescriptor, buf: &mut [u8]) -> Result<usize, Error>,
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

macro_rules! no_media {
    () => {
        Media { open: no_open, close: no_close, read: no_read }
    };
}

impl Default for Media {
    fn default() -> Self {
        no_media!()
    }
}

static mut MEDIAS: [Media; 2] = [no_media!(), no_media!()];

#[derive(Debug)]
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
