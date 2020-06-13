use embedded_sdmmc::{
    BlockDevice, Controller, Directory, Error, File, Mode, TimeSource, Volume, VolumeIdx,
};

use crate::sys::fs::{self, FileDescriptor, OpenOptions};

pub struct Sdcard<'a, D: BlockDevice, T: TimeSource> {
    controller: &'a mut Controller<D, T>,
    volume: Volume,
    root: Option<Directory>,
    files: [Option<File>; 4],
    num_open_file: usize,
}

impl Into<Mode> for OpenOptions {
    fn into(self) -> Mode {
        match (self.read, self.write, self.create, self.append, self.truncate) {
            (true, false, false, false, false) => Mode::ReadOnly,
            (true, true, false, true, _) => Mode::ReadWriteAppend,
            (true, true, false, false, true) => Mode::ReadWriteTruncate,
            (true, true, true, false, false) => Mode::ReadWriteCreate,
            (true, true, true, false, true) => Mode::ReadWriteCreateOrTruncate,
            (true, true, true, true, _) => Mode::ReadWriteCreateOrAppend,
            _ => Mode::ReadOnly,
        }
    }
}

impl<'a, D: BlockDevice, T: TimeSource> Sdcard<'a, D, T> {
    pub fn new(controller: &'a mut Controller<D, T>) -> Option<Self> {
        match controller.get_volume(VolumeIdx(0)) {
            Ok(volume) => Some(Self {
                controller,
                volume,
                root: None,
                files: Default::default(),
                num_open_file: 0,
            }),
            Err(e) => {
                warn!("Get volumn 0 err: {:?}", e);
                None
            }
        }
    }

    pub fn open(&mut self, path: &str, options: OpenOptions) -> Result<FileDescriptor, fs::Error> {
        if self.num_open_file == 0 {
            let root = match self.controller.open_root_dir(&self.volume) {
                Ok(root) => root,
                Err(e) => {
                    warn!("Open root err: {:?}", e);
                    return Err(fs::Error::Generic);
                }
            };
            self.root = Some(root);
        }
        let index = match self.files.iter().position(|option| option.is_none()) {
            Some(i) => i,
            None => return Err(fs::Error::InsufficentResource),
        };
        let root = self.root.as_ref().unwrap();
        let mode: Mode = options.into();
        match self.controller.open_file_in_dir(&mut self.volume, root, &path, mode) {
            Ok(file) => {
                self.files[index] = Some(file);
            }
            Err(e) => {
                return Err(match e {
                    Error::FileNotFound => fs::Error::NotFound,
                    _ => fs::Error::Generic,
                })
            }
        }
        self.num_open_file += 1;
        Ok(FileDescriptor(index))
    }

    pub fn close(&mut self, fd: FileDescriptor) {
        if let Some(file) = self.files[fd.0].take() {
            self.controller.close_file(&self.volume, file).ok();
        }
        self.num_open_file -= 1;
        if self.num_open_file == 0 {
            self.controller.close_dir(&self.volume, self.root.take().unwrap());
        }
    }

    pub fn read(&mut self, fd: &FileDescriptor, buf: &mut [u8]) -> Result<usize, fs::Error> {
        if let Some(file) = &mut self.files[fd.0] {
            match self.controller.read(&self.volume, file, buf) {
                Ok(size) => Ok(size),
                Err(e) => Err(match e {
                    Error::EndOfFile => fs::Error::EndOfFile,
                    _ => fs::Error::Generic,
                }),
            }
        } else {
            Err(fs::Error::NotFound)
        }
    }

    pub fn destroy(&mut self) {
        if let Some(root) = self.root.take() {
            self.controller.close_dir(&self.volume, root);
        }
    }
}
