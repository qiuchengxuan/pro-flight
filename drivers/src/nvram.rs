use core::mem;

use hal::flash::Flash;

#[cfg(target_endian = "big")]
const ACTIVE: u32 = 0x4E565241;
#[cfg(target_endian = "little")]
const ACTIVE: u32 = 0x4152564E;

const EMPTY: u32 = 0xFFFFFFFF;

fn locate(sector: &[u32]) -> (usize, usize) {
    if sector[1] == EMPTY {
        return (0, 1);
    }
    let mut read = 1;
    let mut index = 1;
    while index < sector.len() && sector[index] != EMPTY {
        let length = sector[index] as usize / 4;
        let words = &sector[index + 1..];
        if words[length - 1] != EMPTY {
            read = index;
        }
        index += 1 + length;
    }
    (read, index)
}

pub struct NVRAM<F> {
    flash: F,
    sectors: [&'static mut [u32]; 2],
    active_sector: usize,
    read_offset: usize, // 0 means empty
    write_offset: usize,
}

impl<E, F: Flash<u32, Error = E>> NVRAM<F> {
    pub fn new(flash: F, sectors: [&'static mut [u32]; 2]) -> Self {
        Self { flash, sectors, active_sector: 0, read_offset: 0, write_offset: 1 }
    }

    pub fn init(&mut self) -> Result<(), E> {
        let sectors = &mut self.sectors;
        let flash = &mut self.flash;
        if sectors[0][0] != ACTIVE && sectors[0][0] != EMPTY {
            flash.erase(sectors[0].as_ptr() as *const _ as usize)?;
        }
        if sectors[1][0] != ACTIVE && sectors[1][0] != EMPTY {
            flash.erase(sectors[1].as_ptr() as *const _ as usize)?;
        }
        let active_sector = match (sectors[0][0], sectors[1][0]) {
            (ACTIVE, ACTIVE) => {
                let sector1 = &sectors[0];
                let mut active_sector = 0;
                if sector1[1] != EMPTY {
                    if sector1[sector1[1] as usize] != EMPTY {
                        active_sector = 1;
                    }
                }
                flash.erase(&sectors[active_sector ^ 1][0] as *const _ as usize)?;
                active_sector
            }
            (ACTIVE, _) => 0,
            (_, ACTIVE) => 1,
            (_, _) => 0,
        };
        if sectors[active_sector][0] != ACTIVE {
            flash.program(&sectors[active_sector][0] as *const _ as usize, &[ACTIVE])?;
        }
        let (read_offset, write_offset) = locate(sectors[active_sector]);
        self.active_sector = active_sector;
        self.read_offset = read_offset;
        self.write_offset = write_offset;
        debug!("NVRAM address 0x{:x}", &sectors[active_sector][read_offset] as *const _ as usize);
        Ok(())
    }

    pub fn load<'a, T: From<&'a [u32]> + Default>(&'a self) -> Result<Option<T>, E> {
        if self.read_offset == 0 {
            debug!("NVRAM empty");
            return Ok(None);
        }
        let sector = &self.sectors[self.active_sector];
        let size = sector[self.read_offset] as usize;
        if size != mem::size_of::<T>() {
            debug!("NVRAM ignored, expected size {} actual {}", mem::size_of::<T>(), size);
            return Ok(None);
        }
        let sector = &sector[self.read_offset + 1..];
        debug!("Loading from NVRAM address 0x{:x}", sector.as_ptr() as *const _ as usize);
        Ok(Some(T::from(&sector[..size / 4])))
    }

    pub fn store<'a, T: AsRef<[u32]>>(&mut self, t: T) -> Result<(), E> {
        let sector = &self.sectors[self.active_sector];
        let words = t.as_ref();
        let offset = self.write_offset;
        if offset + 1 + words.len() > sector.len() {
            self.active_sector = self.active_sector ^ 1;
            let new_sector = &self.sectors[self.active_sector];
            debug!("Programming to address 0x{:x}", &new_sector[0] as *const _ as usize);
            self.flash.program(&new_sector[0] as *const _ as usize, &[ACTIVE])?;
            self.flash.program(&new_sector[1] as *const _ as usize, &[words.len() as u32 * 4])?;
            self.flash.program(&new_sector[2] as *const _ as usize, words)?;
            self.flash.erase(&sector[0] as *const _ as usize)?;
            self.read_offset = 1;
            self.write_offset = 1 + words.len();
        } else {
            let buffer = &sector[offset..];
            debug!("Programming to address 0x{:x}", &buffer[0] as *const _ as usize);
            self.flash.program(&buffer[0] as *const _ as usize, &[words.len() as u32 * 4])?;
            self.flash.program(&buffer[1] as *const _ as usize, words)?;
            self.read_offset = self.write_offset;
            self.write_offset += 1 + words.len();
        }
        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), E> {
        let address = self.sectors[self.active_sector].as_ptr() as *const _ as usize;
        self.flash.erase(address)?;
        self.flash.program(address, &[ACTIVE])?;
        self.read_offset = 0;
        self.write_offset = 1;
        Ok(())
    }
}

mod test {
    use core::slice;

    use crate::hal::flash::Flash;

    #[derive(Default)]
    pub struct DummyFlash(());

    impl Flash<u32> for DummyFlash {
        type Error = ();
        fn erase(&mut self, address: usize) -> Result<(), ()> {
            unsafe { *(address as *mut u32) = super::EMPTY }
            Ok(())
        }

        fn program(&mut self, address: usize, words: &[u32]) -> Result<(), ()> {
            let dest = unsafe { slice::from_raw_parts_mut(address as *mut u32, words.len()) };
            dest.copy_from_slice(words);
            Ok(())
        }
    }

    #[derive(Copy, Clone, Debug, Default, PartialEq)]
    pub struct Data([u32; 2]);

    impl AsRef<[u32]> for Data {
        fn as_ref(&self) -> &[u32] {
            &self.0[..]
        }
    }

    impl From<&[u32]> for Data {
        fn from(words: &[u32]) -> Self {
            let mut data = Self::default();
            data.0.copy_from_slice(words);
            data
        }
    }

    #[test]
    fn test_nvram() {
        let sector0 = Box::leak(Box::new([super::EMPTY; 8]));
        let sector1 = Box::leak(Box::new([super::EMPTY; 8]));
        let flash = DummyFlash::default();
        let mut nvram = super::NVRAM::new(flash, [&mut sector0[..], &mut sector1[..]]).unwrap();
        assert_eq!(nvram.sectors[0][0], super::ACTIVE);
        let expected: Option<&[u32]> = None;
        assert_eq!(expected, nvram.load().expect("load"));
        let expect = Data([3, 4]);
        nvram.store(expect).expect("store");
        assert_eq!(Some(expect), nvram.load().expect("load"));
        let expect = Data([5, 6]);
        nvram.store(expect).expect("store");
        assert_eq!(Some(expect), nvram.load().expect("load"));
        let expect = Data([7, 8]);
        nvram.store(expect).expect("store");
        assert_eq!(Some(expect), nvram.load().expect("load"));
        assert_eq!(nvram.sectors[0][0], super::EMPTY);
        let expect = Data([9, 10]);
        nvram.store(expect).expect("store");
        assert_eq!(Some(expect), nvram.load().expect("load"));
        nvram.sectors[0][1] = super::EMPTY;
        let expect = Data([11, 12]);
        nvram.store(expect).expect("store");
        assert_eq!(Some(expect), nvram.load().expect("load"));
    }

    #[test]
    fn test_load_from_existing() {
        let sector0 = Box::leak(Box::new([super::ACTIVE, 2, 1, 2]));
        let sector1 = Box::leak(Box::new([super::EMPTY; 4]));
        let flash = DummyFlash::default();
        let nvram = super::NVRAM::new(flash, [&mut sector0[..], &mut sector1[..]]).unwrap();
        assert_eq!(nvram.active_sector, 0);
        let actual: Option<&[u32]> = nvram.load().expect("load");
        assert_eq!(Some(&[1, 2][..]), actual);
    }
}
