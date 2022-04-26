use alloc::boxed::Box;
use core::ops;

use super::meta::{Meta, Owner, TransferResult};

#[repr(C)]
pub struct BufferDescriptor<W: Copy + Default + 'static, const N: usize> {
    meta: Meta<W>,
    buffer: [W; N],
}

unsafe impl<W: Copy + Default + Send + 'static, const N: usize> Send for BufferDescriptor<W, N> {}
unsafe impl<W: Copy + Default + Sync + 'static, const N: usize> Sync for BufferDescriptor<W, N> {}

impl<W: Copy + Default + 'static, const N: usize> Default for BufferDescriptor<W, N> {
    fn default() -> Self {
        Self { meta: Meta { size: N, ..Default::default() }, buffer: [W::default(); N] }
    }
}

pub struct Buffer<'a, W: 'static> {
    meta: &'a Meta<W>,
    buffer: &'a mut [W],
}

impl<'a, W> Drop for Buffer<'a, W> {
    fn drop(&mut self) {
        unsafe { self.meta.release() }
    }
}

impl<'a, W> ops::Deref for Buffer<'a, W> {
    type Target = [W];

    fn deref(&self) -> &[W] {
        self.buffer
    }
}

impl<'a, W> ops::DerefMut for Buffer<'a, W> {
    fn deref_mut(&mut self) -> &mut [W] {
        self.buffer
    }
}

impl<'a, W> AsRef<[W]> for Buffer<'a, W> {
    fn as_ref(&self) -> &[W] {
        self.buffer
    }
}

impl<'a, W> AsMut<[W]> for Buffer<'a, W> {
    fn as_mut(&mut self) -> &mut [W] {
        self.buffer
    }
}

impl<W: Copy + Default + 'static, const N: usize> Drop for BufferDescriptor<W, N> {
    fn drop(&mut self) {
        if let Some(callback) = self.meta.callback.take() {
            core::mem::drop(unsafe { Box::from_raw(callback) });
        }
    }
}

impl<W: Copy + Default + 'static, const N: usize> BufferDescriptor<W, N> {
    pub fn new(array: [W; N]) -> Self {
        Self { meta: Meta { size: N, ..Default::default() }, buffer: array }
    }

    pub unsafe fn get_buffer(&self) -> &[W; N] {
        &self.buffer
    }

    pub fn with_callback<C>(callback: C) -> Self
    where
        C: FnMut(TransferResult<W>) + Send + 'static,
    {
        let callback = Box::leak(Box::new(callback));
        Self {
            meta: Meta { size: N, callback: Some(callback), ..Default::default() },
            buffer: [W::default(); N],
        }
    }

    pub fn new_with_callback<C>(array: [W; N], callback: C) -> Self
    where
        C: FnMut(TransferResult<W>) + Send + 'static,
    {
        let callback = Box::leak(Box::new(callback));
        Self {
            meta: Meta { size: N, callback: Some(callback), ..Default::default() },
            buffer: array,
        }
    }

    pub fn set_size(&mut self, size: usize) {
        self.meta.size = size
    }

    pub fn try_get_buffer<'a>(&'a mut self) -> Result<Buffer<'a, W>, Owner> {
        self.meta
            .take_ownership(Owner::CPU)
            .map(move |_| Buffer { meta: &self.meta, buffer: &mut self.buffer[..] })
    }

    pub fn try_take(&self) -> Result<&[W], Owner> {
        self.meta.take_ownership(Owner::DMA).map(|_| &self.buffer[..])
    }
}
