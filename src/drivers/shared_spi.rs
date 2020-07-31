use core::cell::{Cell, RefCell};
use core::marker::PhantomData;

use embedded_hal::blocking::spi;
use embedded_hal::digital::v2::OutputPin;

pub trait ChipSelects<E> {
    fn with(&mut self, index: usize, f: impl FnMut(&mut dyn OutputPin<Error = E>));
    fn foreach(&mut self, f: impl FnMut(&mut dyn OutputPin<Error = E>));
}

macro_rules! peel {
    (($idx0:tt) -> $P0:ident
     $(
         ($idx:tt) -> $P:ident
     )*
    ) => (chip_selects! { $(($idx) -> $P)* })
}

macro_rules! chip_selects {
    () => ();
    ($(($idx:tt) -> $CS:ident)+) => {
        impl<E, $($CS: OutputPin<Error = E>,)+> ChipSelects<E> for ($($CS,)+) {
            fn with(&mut self, index: usize, mut f: impl FnMut(&mut dyn OutputPin<Error = E>)) {
                match index {
                    $(
                        $idx => f(&mut self.$idx),
                    )+
                    _ => (),
                }
            }

            fn foreach(&mut self, mut f: impl FnMut(&mut dyn OutputPin<Error = E>)) {
                $(f(&mut self.$idx);)+
            }
        }
        peel!{ $(($idx) -> $CS)+ }
    }
}

chip_selects! {
    (1) -> CS1
    (0) -> CS0
}

// NOTE: thread unsafe
pub struct SharedSpi<E, SPI, CSS> {
    cell: RefCell<SPI>,
    chip_selects: RefCell<CSS>,
    owner: Cell<isize>,
    _e: PhantomData<E>,
}

impl<E, SPI, CSS: ChipSelects<E>> SharedSpi<E, SPI, CSS> {
    pub fn new(spi: SPI, mut chip_selects: CSS) -> Self {
        chip_selects.foreach(|cs| {
            cs.set_high().ok();
        });
        chip_selects.with(0, |cs| {
            cs.set_low().ok();
        });
        Self {
            cell: RefCell::new(spi),
            chip_selects: RefCell::new(chip_selects),
            owner: Cell::new(0),
            _e: PhantomData,
        }
    }

    pub fn into_inner(self) -> (SPI, CSS) {
        (self.cell.into_inner(), self.chip_selects.into_inner())
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
            Ok(mut css) => {
                if owner >= 0 {
                    css.with(owner as usize, |cs| {
                        cs.set_high().ok();
                    })
                }
                css.with(index, |cs| {
                    cs.set_low().ok();
                });
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
            Ok(mut css) => {
                css.with(index, |cs| {
                    cs.set_high().ok();
                });
                self.owner.set(-1);
            }
            _ => (),
        }
    }
}

pub struct VirtualSpi<'a, E, SPI, CSS> {
    shared: &'a SharedSpi<E, SPI, CSS>,
    index: usize,
}

impl<'a, E, SPI, CSS> VirtualSpi<'a, E, SPI, CSS> {
    pub fn new(shared: &'a SharedSpi<E, SPI, CSS>, index: usize) -> Self {
        Self { shared, index }
    }
}

impl<'a, E, T, SPI, CSS: ChipSelects<E>> spi::Write<u8> for VirtualSpi<'a, E, SPI, CSS>
where
    SPI: spi::Write<u8, Error = T>,
{
    type Error = T;

    fn write(&mut self, bytes: &[u8]) -> Result<(), T> {
        self.shared.acquire(self.index);
        let result = self.shared.cell.borrow_mut().write(bytes);
        if bytes.len() > 1 {
            self.shared.release(self.index);
        }
        result
    }
}

impl<'a, E, T, SPI, CSS: ChipSelects<E>> spi::Transfer<u8> for VirtualSpi<'a, E, SPI, CSS>
where
    SPI: spi::Transfer<u8, Error = T>,
{
    type Error = T;

    fn transfer<'b>(&mut self, bytes: &'b mut [u8]) -> Result<&'b [u8], T> {
        self.shared.acquire(self.index);
        let result = self.shared.cell.borrow_mut().transfer(bytes);
        self.shared.release(self.index);
        result
    }
}
