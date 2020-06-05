use core::cell::{Cell, RefCell};
use core::convert::Infallible;

use embedded_hal::blocking::spi;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::FullDuplex;

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

pub struct VirtualChipSelect<'a, E, SPI> {
    shared: &'a SharedSpi<'a, E, SPI>,
    index: usize,
}

impl<'a, E, SPI> VirtualChipSelect<'a, E, SPI> {
    pub fn new(shared: &'a SharedSpi<'a, E, SPI>, index: usize) -> Self {
        Self { shared, index }
    }
}

impl<'a, E, SPI> OutputPin for VirtualChipSelect<'a, E, SPI> {
    type Error = Infallible;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(self.shared.acquire(self.index))
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(self.shared.release(self.index))
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

impl<'a, E, T, W, SPI> FullDuplex<W> for VirtualSpi<'a, T, SPI>
where
    SPI: FullDuplex<W, Error = E>,
{
    type Error = E;

    fn send(&mut self, word: W) -> nb::Result<(), E> {
        self.shared.acquire(self.index);
        let mut spi = self.shared.cell.borrow_mut();
        spi.send(word)
    }

    fn read(&mut self) -> nb::Result<W, E> {
        self.shared.acquire(self.index);
        let mut spi = self.shared.cell.borrow_mut();
        spi.read()
    }
}

impl<'a, W, E, SPI: FullDuplex<W>> spi::transfer::Default<W> for VirtualSpi<'a, E, SPI> {}

impl<'a, W, E, SPI: FullDuplex<W>> spi::write::Default<W> for VirtualSpi<'a, E, SPI> {}
