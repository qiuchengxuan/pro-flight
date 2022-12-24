pub struct FileDescriptor(pub usize);

pub trait Media {
    fn open(&self, path: &str, options: OpenOptions) -> Result<FileDescriptor, Error>;
    fn close(&self, fd: FileDescriptor);
    fn read(&self, fd: &FileDescriptor, buf: &mut [u8]) -> Result<usize, Error>;
    fn write(&self, fd: &FileDescriptor, bytes: &[u8]) -> Result<usize, Error>;
    fn metadata(&self, fd: &FileDescriptor) -> Result<Metadata, Error>;
}

pub struct NoMedia;

impl Media for NoMedia {
    fn open(&self, _: &str, _: OpenOptions) -> Result<FileDescriptor, Error> {
        Err(Error::NoMedia)
    }

    fn close(&self, _: FileDescriptor) {}

    fn read(&self, _: &FileDescriptor, _: &mut [u8]) -> Result<usize, Error> {
        Err(Error::NoMedia)
    }

    fn write(&self, _: &FileDescriptor, _: &[u8]) -> Result<usize, Error> {
        Err(Error::NoMedia)
    }

    fn metadata(&self, _: &FileDescriptor) -> Result<Metadata, Error> {
        Err(Error::Generic)
    }
}
