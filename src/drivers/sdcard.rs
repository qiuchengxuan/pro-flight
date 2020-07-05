use core::cell::RefCell;
use core::ops::DerefMut;

use embedded_sdmmc::{
    BlockDevice, Controller, Directory, Error, File, Mode, TimeSource, Volume, VolumeIdx,
};

use crate::sys::fs::{self, FileDescriptor, Media, OpenOptions};

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

pub struct Sdcard<D: BlockDevice, T: TimeSource> {
    controller: RefCell<Controller<D, T>>,
    filesystem: Option<RefCell<(Volume, Directory, [Option<File>; 4])>>,
}

fn generic_error<E: core::fmt::Debug>(e: E) -> fs::Error {
    warn!("{:?}", e);
    fs::Error::Generic
}

impl<D: BlockDevice, T: TimeSource> Sdcard<D, T> {
    pub fn new(controller: Controller<D, T>) -> Self {
        Self { controller: RefCell::new(controller), filesystem: None }
    }

    pub fn probe<F>(&mut self, init: F) -> Result<(), fs::Error>
    where
        F: Fn(&mut Controller<D, T>) -> bool,
    {
        let mut controller = match self.controller.try_borrow_mut() {
            Ok(controller) => controller,
            Err(e) => return Err(generic_error(e)),
        };
        if !init(&mut controller) {
            return Err(fs::Error::NoMedia);
        }

        let volume = match controller.get_volume(VolumeIdx(0)) {
            Ok(volume) => volume,
            Err(e) => return Err(generic_error(e)),
        };
        let root = match controller.open_root_dir(&volume) {
            Ok(root) => root,
            Err(e) => return Err(generic_error(e)),
        };
        self.filesystem = Some(RefCell::new((volume, root, Default::default())));
        return Ok(());
    }

    pub fn invalidate(&mut self) {
        self.filesystem = None;
    }
}

impl<D: BlockDevice, T: TimeSource> Media for Sdcard<D, T> {
    fn open(&self, path: &str, options: OpenOptions) -> Result<FileDescriptor, fs::Error> {
        let mut controller = match self.controller.try_borrow_mut() {
            Ok(controller) => controller,
            Err(e) => return Err(generic_error(e)),
        };
        let mut filesystem = match &self.filesystem {
            Some(ref_cell) => ref_cell.borrow_mut(),
            None => return Err(fs::Error::NoMedia),
        };
        let (volume, root, files) = filesystem.deref_mut();

        let index = match files.iter().position(|option| option.is_none()) {
            Some(i) => i,
            None => return Err(fs::Error::InsufficentResource),
        };
        let mode: Mode = options.into();
        match controller.open_file_in_dir(volume, &root, path, mode) {
            Ok(file) => {
                files[index] = Some(file);
            }
            Err(e) => {
                return Err(match e {
                    Error::FileNotFound => fs::Error::NotFound,
                    _ => generic_error(e),
                });
            }
        }
        Ok(FileDescriptor(index))
    }

    fn close(&self, fd: FileDescriptor) {
        let mut controller = match self.controller.try_borrow_mut() {
            Ok(controller) => controller,
            Err(_) => return,
        };
        let mut filesystem = match &self.filesystem {
            Some(ref_cell) => ref_cell.borrow_mut(),
            None => return,
        };
        let (volume, _, files) = filesystem.deref_mut();
        if let Some(file) = files[fd.0].take() {
            controller.close_file(&volume, file).ok();
        }
    }

    fn read(&self, fd: &FileDescriptor, buf: &mut [u8]) -> Result<usize, fs::Error> {
        let mut controller = match self.controller.try_borrow_mut() {
            Ok(controller) => controller,
            Err(e) => return Err(generic_error(e)),
        };
        let mut filesystem = match &self.filesystem {
            Some(ref_cell) => ref_cell.borrow_mut(),
            None => return Err(fs::Error::NoMedia),
        };
        let (volume, _, files) = filesystem.deref_mut();

        if let Some(ref mut file) = files[fd.0] {
            return match controller.read(volume, file, buf) {
                Ok(size) => Ok(size),
                Err(e) => Err(match e {
                    Error::EndOfFile => fs::Error::EndOfFile,
                    _ => generic_error(e),
                }),
            };
        }
        Err(fs::Error::Generic)
    }

    fn write(&self, fd: &FileDescriptor, bytes: &[u8]) -> Result<usize, fs::Error> {
        let mut controller = match self.controller.try_borrow_mut() {
            Ok(controller) => controller,
            Err(e) => return Err(generic_error(e)),
        };
        let mut filesystem = match &self.filesystem {
            Some(ref_cell) => ref_cell.borrow_mut(),
            None => return Err(fs::Error::NoMedia),
        };
        let (volume, _, files) = filesystem.deref_mut();

        if let Some(ref mut file) = files[fd.0] {
            return match controller.write(volume, file, bytes) {
                Ok(size) => Ok(size),
                Err(e) => Err(generic_error(e)),
            };
        }
        Err(fs::Error::NotFound)
    }
}
