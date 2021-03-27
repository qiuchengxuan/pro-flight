use core::future::Future;
use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};

use drone_core::fib::{new_fn, Yielded};
use drone_cortexm::{reg::prelude::*, reg::Reg as _, thr::ThrNvic};
use drone_stm32_map::periph::dma::ch::*;

use crate::hal::dma::{BufferDescriptor, DMAFuture, Meta, Peripheral, DMA};

pub enum Direction {
    PeripheralToMemory = 0b00,
    MemoryToPeripheral = 0b01,
}

pub enum Burst {
    Single = 0b00,
    Incr4 = 0b01,
    Incr8 = 0b10,
    Incr16 = 0b11,
}

pub struct Reg<M: DmaChMap> {
    configuration: M::CDmaCcr,
    memory0_address: M::CDmaCm0Ar,
    number_of_data: M::SDmaCndtr,
    peripheral_address: M::SDmaCpar,
    transfer_complete: M::CDmaIfcrCtcif,
    half_transfer: M::CDmaIfcrChtif,
    transfer_error: M::CDmaIfcrCteif,
    direct_mode_error: M::CDmaIfcrCdmeif,
}

impl<M: DmaChMap> Reg<M> {
    fn clear_interrupts(&self) {
        self.transfer_complete.set_bit();
        self.half_transfer.set_bit();
        self.transfer_error.set_bit();
        self.direct_mode_error.set_bit();
    }
}

impl<M: DmaChMap> From<DmaChPeriph<M>> for Reg<M> {
    fn from(reg: DmaChPeriph<M>) -> Self {
        Self {
            configuration: reg.dma_ccr.into_copy(),
            memory0_address: reg.dma_cm0ar.into_copy(),
            number_of_data: reg.dma_cndtr,
            peripheral_address: reg.dma_cpar,
            transfer_complete: reg.dma_ifcr_ctcif.into_copy(),
            half_transfer: reg.dma_ifcr_chtif.into_copy(),
            transfer_error: reg.dma_ifcr_cteif.into_copy(),
            direct_mode_error: reg.dma_ifcr_cdmeif.into_copy(),
        }
    }
}

pub struct DMABusy<M: DmaChMap>(M::CDmaCcr);

impl<M: DmaChMap> Future for DMABusy<M> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _ctx: &mut Context) -> Poll<Self::Output> {
        return if self.0.en().read_bit() { Poll::Pending } else { Poll::Ready(()) };
    }
}

impl<M: DmaChMap> DMAFuture for DMABusy<M> {}

pub struct Channel<M: DmaChMap> {
    reg: Reg<M>,
}

impl<M: DmaChMap> Channel<M> {
    pub fn new<INT: ThrNvic>(raw: DmaChPeriph<M>, int: INT) -> Self {
        let reg: Reg<M> = raw.into();
        let (address_reg, transfer_complete) = (reg.memory0_address, reg.transfer_complete);
        int.add_fib(new_fn(move || {
            transfer_complete.set_bit();
            unsafe {
                let meta = Meta::from_raw(address_reg.load_bits() as usize);
                let bytes = meta.get_bytes();
                meta.transfer_done.as_mut().map(|f| f(bytes));
                meta.release();
            }
            Yielded::<(), ()>(())
        }));
        int.enable_int();
        reg.configuration.tcie().set_bit();
        Self { reg }
    }
}

impl<M: DmaChMap> DMA for Channel<M> {
    type Future = DMABusy<M>;

    fn setup_peripheral(&mut self, channel: u8, periph: &mut dyn Peripheral) {
        periph.enable_dma();
        self.reg.peripheral_address.store_bits(periph.address() as u32);
        self.reg.configuration.modify_reg(|r, v| {
            r.chsel().write(v, channel as u32);
            r.psize().write(v, periph.word_size() as u32 - 1);
        });
    }

    fn tx<'a, W, BD, const N: usize>(&'a self, bd: BD, repeat: Option<usize>) -> DMABusy<M>
    where
        W: Copy + Default,
        BD: AsRef<BufferDescriptor<W, N>> + 'a,
    {
        let bytes = bd.as_ref().take();
        self.reg.memory0_address.store_bits(bytes.as_ptr() as *const _ as u32);
        let msize = mem::size_of::<W>() as u32 - 1;
        self.reg.clear_interrupts();
        self.reg.number_of_data.store_bits(repeat.unwrap_or(bytes.len()) as u32);
        self.reg.configuration.modify_reg(|r, v| {
            if repeat.is_some() {
                r.minc().clear(v);
            } else {
                r.minc().set(v);
            }
            r.msize().write(v, msize);
            r.dir().write(v, Direction::MemoryToPeripheral as u32);
            r.en().set(v);
        });
        DMABusy(self.reg.configuration)
    }

    fn rx<'a, W, BD, const N: usize>(&'a self, bd: BD, circle: bool) -> DMABusy<M>
    where
        W: Copy + Default,
        BD: AsRef<BufferDescriptor<W, N>> + 'a,
    {
        let buffer = bd.as_ref().take();
        self.reg.memory0_address.store_bits(buffer.as_ptr() as *const _ as u32);
        let msize = mem::size_of::<W>() as u32 - 1;
        self.reg.clear_interrupts();
        self.reg.number_of_data.store_bits(buffer.len() as u32);
        self.reg.configuration.modify_reg(|r, v| {
            r.minc().set(v);
            r.msize().write(v, msize);
            if circle {
                r.circ().set(v);
            }
            r.dir().write(v, Direction::PeripheralToMemory as u32);
            r.en().set(v);
        });
        DMABusy(self.reg.configuration)
    }

    fn stop(&self) {
        if self.reg.configuration.en().read_bit() {
            self.reg.configuration.en().clear_bit();
            while self.reg.configuration.en().read_bit() {}
        }
        let address = self.reg.memory0_address.load_bits();
        if address > 0 {
            unsafe { Meta::from_raw(address as usize).release() }
            self.reg.memory0_address.store_bits(0);
        }
    }
}

impl<M: DmaChMap> Drop for Channel<M> {
    fn drop(&mut self) {
        self.stop()
    }
}

unsafe impl<M: DmaChMap> Send for Channel<M> {}
