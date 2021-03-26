use core::sync::atomic::{AtomicBool, Ordering};
use core::{mem, slice};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Owner {
    CPU,
    DMA,
}

impl Into<bool> for Owner {
    fn into(self) -> bool {
        match self {
            Self::CPU => false,
            Self::DMA => true,
        }
    }
}

#[derive(Default)]
pub struct Meta {
    pub size: usize,
    pub owner: AtomicBool,
    pub transfer_done: Option<&'static mut dyn FnMut(&[u8])>,
}

impl Meta {
    pub unsafe fn from_raw<'a>(pointer: usize) -> &'a mut Self {
        let address = pointer - mem::size_of::<Self>();
        &mut *(address as *mut Self)
    }

    pub unsafe fn get_bytes<'a>(&self) -> &'a [u8] {
        let address = self as *const _ as usize + mem::size_of::<Self>();
        slice::from_raw_parts(address as *const u8, self.size)
    }

    pub fn release(&mut self) {
        self.owner.store(Owner::CPU.into(), Ordering::Relaxed);
    }
}

#[repr(C)]
pub struct Buffer<W: Default + Copy, const N: usize> {
    meta: Meta,
    buffer: [W; N],
}

impl<W: Default + Copy, const N: usize> Default for Buffer<W, N> {
    fn default() -> Self {
        Self { meta: Meta { size: N, ..Default::default() }, buffer: [W::default(); N] }
    }
}

impl<W: Copy + Default, const N: usize> Buffer<W, N> {
    pub fn new(array: [W; N]) -> Self {
        Self { meta: Meta { size: N, ..Default::default() }, buffer: array }
    }

    pub fn set_transfer_done(&mut self, closure: &'static mut (dyn FnMut(&[u8]) + Send + 'static)) {
        self.meta.transfer_done = Some(closure)
    }
}

pub struct BufferDescriptor<'a, W> {
    buffer: &'a mut [W],
    owner: &'a AtomicBool,
}

impl<'a, W: Copy + Default, const N: usize> From<&'a mut Buffer<W, N>> for BufferDescriptor<'a, W> {
    fn from(buffer: &'a mut Buffer<W, N>) -> Self {
        Self { buffer: &mut buffer.buffer[..], owner: &buffer.meta.owner }
    }
}

impl<'a, W> BufferDescriptor<'a, W> {
    pub fn borrow_mut(&mut self) -> Option<&mut [W]> {
        if self.owner.load(Ordering::Relaxed) == Owner::CPU.into() {
            return Some(self.buffer);
        }
        None
    }

    pub fn take(&self) -> &[W] {
        self.owner.store(Owner::DMA.into(), Ordering::Relaxed);
        self.buffer
    }
}

pub trait Peripheral {
    fn enable_dma(&mut self);
    fn address(&mut self) -> usize;
    fn word_size(&self) -> usize;
}

pub trait DMA: Send + 'static {
    fn setup_peripheral(&mut self, channel: u8, periph: &mut dyn Peripheral);
    fn tx<W>(&mut self, buffer: &BufferDescriptor<W>, repeat: Option<usize>);
    fn rx<W>(&mut self, buffer: &BufferDescriptor<W>, circle: bool);
}
