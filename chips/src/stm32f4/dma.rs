use core::future::Future;
use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};

use drone_core::fib::Yielded;
use drone_cortexm::{reg::prelude::*, reg::Reg as _, thr::prelude::*, thr::ThrNvic};
use drone_stm32_map::periph::dma::ch::*;

use hal::dma::{BufferDescriptor, DMAFuture, Meta, Peripheral, TransferOption, DMA};

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
    number_of_data: M::CDmaCndtr,
    peripheral_address: M::CDmaCpar,
    transfer_complete: M::CDmaIfcrCtcif,
    half_transfer: M::CDmaIfcrChtif,
    transfer_error: M::CDmaIfcrCteif,
    direct_mode_error: M::CDmaIfcrCdmeif,
}

impl<M: DmaChMap> Clone for Reg<M> {
    fn clone(&self) -> Self {
        Self {
            configuration: self.configuration,
            memory0_address: self.memory0_address,
            number_of_data: self.number_of_data,
            peripheral_address: self.peripheral_address,
            transfer_complete: self.transfer_complete,
            half_transfer: self.half_transfer,
            transfer_error: self.transfer_error,
            direct_mode_error: self.direct_mode_error,
        }
    }
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
            number_of_data: reg.dma_cndtr.into_copy(),
            peripheral_address: reg.dma_cpar.into_copy(),
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

pub struct Stream<M: DmaChMap> {
    reg: Reg<M>,
}

impl<M: DmaChMap> Clone for Stream<M> {
    fn clone(&self) -> Self {
        Self { reg: self.reg.clone() }
    }
}

impl<M: DmaChMap> Stream<M> {
    pub fn new<INT: ThrNvic>(raw: DmaChPeriph<M>, int: INT) -> Self {
        let reg: Reg<M> = raw.into();
        let (address_reg, transfer_complete) = (reg.memory0_address, reg.transfer_complete);
        int.add_fn(move || {
            transfer_complete.set_bit();
            let address = address_reg.load_bits() as usize;
            let meta = unsafe { Meta::<u8>::from_raw(address) };
            let data = unsafe { meta.get_data() };
            meta.release();
            unsafe { meta.get_transfer_done() }.map(|f| f(data));
            Yielded::<(), ()>(())
        });
        int.enable_int();
        reg.configuration.tcie().set_bit();
        Self { reg }
    }
}

impl<M: DmaChMap> DMA for Stream<M> {
    type Future = DMABusy<M>;

    fn setup_peripheral(&mut self, channel: u8, periph: &mut dyn Peripheral) {
        periph.enable_dma();
        self.reg.peripheral_address.store_bits(periph.address() as u32);
        self.reg.configuration.modify_reg(|r, v| {
            r.chsel().write(v, channel as u32);
            r.psize().write(v, periph.word_size() as u32 - 1);
        });
    }

    fn is_busy(&self) -> bool {
        self.reg.configuration.en().read_bit()
    }

    fn tx<'a, W, BD, const N: usize>(&'a self, bd: BD, option: TransferOption) -> DMABusy<M>
    where
        W: Copy + Default,
        BD: AsRef<BufferDescriptor<W, N>> + 'a,
    {
        let bytes = bd.as_ref().take();
        self.reg.memory0_address.store_bits(bytes.as_ptr() as *const _ as u32);
        let msize = mem::size_of::<W>() as u32 - 1;
        self.reg.clear_interrupts();
        self.reg.number_of_data.store_bits(option.size.unwrap_or(bytes.len()) as u32);
        self.reg.configuration.modify_reg(|r, v| {
            if option.fixed {
                r.minc().clear(v)
            } else {
                r.minc().set(v)
            }
            r.msize().write(v, msize);
            r.dir().write(v, Direction::MemoryToPeripheral as u32);
            r.en().set(v);
        });
        DMABusy(self.reg.configuration)
    }

    fn rx<'a, W, BD, const N: usize>(&'a self, bd: BD, option: TransferOption) -> DMABusy<M>
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
            if option.circle {
                r.circ().set(v);
            } else {
                r.circ().clear(v);
            }
            r.dir().write(v, Direction::PeripheralToMemory as u32);
            r.en().set(v);
        });
        DMABusy(self.reg.configuration)
    }

    fn stop(&self) {
        self.reg.configuration.tcie().clear_bit();
        if self.reg.configuration.en().read_bit() {
            self.reg.configuration.en().clear_bit();
            while self.reg.configuration.en().read_bit() {}
        }
        let address = self.reg.memory0_address.load_bits();
        if address > 0 {
            unsafe { Meta::<u8>::from_raw(address as usize).release() }
            self.reg.memory0_address.store_bits(0);
        }
    }
}

unsafe impl<M: DmaChMap> Send for Stream<M> {}
unsafe impl<M: DmaChMap> Sync for Stream<M> {}
