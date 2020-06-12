use core::cell::{Cell, RefCell};

use embedded_hal::blocking::spi;
use embedded_hal::digital::v2::OutputPin;

// NOTE: thread unsafe
pub struct SharedSpi<'a, E, SPI> {
    cell: RefCell<SPI>,
    chip_selects: RefCell<&'a mut [&'a mut dyn OutputPin<Error = E>]>,
    owner: Cell<isize>,
}

impl<'a, E, SPI> SharedSpi<'a, E, SPI> {
    pub fn new(spi: SPI, chip_selects: &'a mut [&'a mut dyn OutputPin<Error = E>]) -> Self {
        for cs in chip_selects.iter_mut() {
            cs.set_high().ok();
        }
        chip_selects[0].set_low().ok();
        Self {
            cell: RefCell::new(spi),
            chip_selects: RefCell::new(chip_selects),
            owner: Cell::new(0),
        }
    }

    pub fn owner(&self, index: usize) -> bool {
        self.owner.get() == index as isize
    }

    pub fn acquire(&self, index: usize) {
        let owner = self.owner.get();
        if owner == index as isize {
            return;
        }
        match self.chip_selects.try_borrow_mut() {
            Ok(mut cs) => {
                if owner >= 0 {
                    cs[owner as usize].set_high().ok();
                }
                cs[index].set_low().ok();
                self.owner.set(index as isize);
            }
            _ => (),
        }
    }

    pub fn release(&self, index: usize) {
        if self.owner.get() != index as isize {
            return;
        }
        match self.chip_selects.try_borrow_mut() {
            Ok(mut cs) => {
                cs[index].set_high().ok();
                self.owner.set(-1);
            }
            _ => (),
        }
    }
}

pub struct VirtualSpi<'a, E, SPI> {
    shared: &'a SharedSpi<'a, E, SPI>,
    index: usize,
}

impl<'a, E, SPI> VirtualSpi<'a, E, SPI> {
    pub fn new(shared: &'a SharedSpi<'a, E, SPI>, index: usize) -> Self {
        Self { shared, index }
    }
}

impl<'a, E, T, SPI> spi::Write<u8> for VirtualSpi<'a, T, SPI>
where
    SPI: spi::Write<u8, Error = E>,
{
    type Error = E;

    fn write(&mut self, bytes: &[u8]) -> Result<(), E> {
        self.shared.acquire(self.index);
        let result = self.shared.cell.borrow_mut().write(bytes);
        if bytes.len() > 1 {
            self.shared.release(self.index);
        }
        result
    }
}

impl<'a, E, T, SPI> spi::Transfer<u8> for VirtualSpi<'a, T, SPI>
where
    SPI: spi::Transfer<u8, Error = E>,
{
    type Error = E;

    fn transfer<'b>(&mut self, bytes: &'b mut [u8]) -> Result<&'b [u8], E> {
        self.shared.acquire(self.index);
        let result = self.shared.cell.borrow_mut().transfer(bytes);
        self.shared.release(self.index);
        result
    }
}
