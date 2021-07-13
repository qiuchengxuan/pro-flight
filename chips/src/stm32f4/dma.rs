use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use core::{cmp, mem};

use drone_core::fib::Yielded;
use drone_cortexm::{reg::prelude::*, reg::Reg as _, thr::prelude::*, thr::ThrNvic};
use drone_stm32_map::periph::dma::ch::*;

use hal::dma::{
    BufferDescriptor, DMAFuture, Meta, Peripheral, TransferOption, TransferResult, DMA,
};

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

struct InterruptClear<M: DmaChMap> {
    transfer_complete: M::CDmaIfcrCtcif,
    half_transfer: M::CDmaIfcrChtif,
    transfer_error: M::CDmaIfcrCteif,
    direct_mode_error: M::CDmaIfcrCdmeif,
}

impl<M: DmaChMap> InterruptClear<M> {
    fn clear_all(&self) {
        self.transfer_complete.set_bit();
        self.half_transfer.set_bit();
        self.transfer_error.set_bit();
        self.direct_mode_error.set_bit();
    }
}

impl<M: DmaChMap> Clone for InterruptClear<M> {
    fn clone(&self) -> Self {
        Self {
            transfer_complete: self.transfer_complete,
            half_transfer: self.half_transfer,
            transfer_error: self.transfer_error,
            direct_mode_error: self.direct_mode_error,
        }
    }
}

struct InterruptStatus<M: DmaChMap> {
    transfer_complete: M::CDmaIsrTcif,
    half_transfer: M::CDmaIsrHtif,
}

impl<M: DmaChMap> Clone for InterruptStatus<M> {
    fn clone(&self) -> Self {
        Self { transfer_complete: self.transfer_complete, half_transfer: self.half_transfer }
    }
}

struct Reg<M: DmaChMap> {
    configuration: M::CDmaCcr,
    memory0_address: M::CDmaCm0Ar,
    number_of_data: M::CDmaCndtr,
    peripheral_address: M::CDmaCpar,
    interrupt_clear: InterruptClear<M>,
    interrupt_status: InterruptStatus<M>,
}

impl<M: DmaChMap> Clone for Reg<M> {
    fn clone(&self) -> Self {
        Self {
            configuration: self.configuration,
            memory0_address: self.memory0_address,
            number_of_data: self.number_of_data,
            peripheral_address: self.peripheral_address,
            interrupt_clear: self.interrupt_clear.clone(),
            interrupt_status: self.interrupt_status.clone(),
        }
    }
}

impl<M: DmaChMap> From<DmaChPeriph<M>> for Reg<M> {
    fn from(reg: DmaChPeriph<M>) -> Self {
        Self {
            configuration: reg.dma_ccr.into_copy(),
            memory0_address: reg.dma_cm0ar.into_copy(),
            number_of_data: reg.dma_cndtr.into_copy(),
            peripheral_address: reg.dma_cpar.into_copy(),
            interrupt_clear: InterruptClear {
                transfer_complete: reg.dma_ifcr_ctcif.into_copy(),
                half_transfer: reg.dma_ifcr_chtif.into_copy(),
                transfer_error: reg.dma_ifcr_cteif.into_copy(),
                direct_mode_error: reg.dma_ifcr_cdmeif.into_copy(),
            },
            interrupt_status: InterruptStatus {
                transfer_complete: reg.dma_isr_tcif.into_copy(),
                half_transfer: reg.dma_isr_htif.into_copy(),
            },
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
    persist: bool,
}

impl<M: DmaChMap> Clone for Stream<M> {
    fn clone(&self) -> Self {
        Self { reg: self.reg.clone(), persist: self.persist }
    }
}

impl<M: DmaChMap> Stream<M> {
    pub fn new<INT: ThrNvic>(raw: DmaChPeriph<M>, int: INT) -> Self {
        let reg: Reg<M> = raw.into();
        let address_reg = reg.memory0_address;
        let status = reg.interrupt_status.clone();
        let clear = reg.interrupt_clear.clone();
        int.add_fn(move || {
            let address = address_reg.load_bits() as usize;
            let half = status.half_transfer.read_bit();
            let meta = unsafe { Meta::<u8>::from_raw(address) };
            let buffer = unsafe { meta.get_buffer() };
            let result =
                if half { TransferResult::Half(buffer) } else { TransferResult::Complete(buffer) };
            if half {
                clear.half_transfer.set_bit();
            }
            if status.transfer_complete.read_bit() {
                meta.release();
                clear.transfer_complete.set_bit();
            }
            let handler = unsafe { meta.get_handler() };
            handler.map(|f| f(result));
            Yielded::<(), ()>(())
        });
        int.enable_int();
        reg.configuration.tcie().set_bit();
        Self { reg, persist: false }
    }

    pub unsafe fn set_persist(&mut self) {
        self.persist = true
    }
}

type BD<W, const N: usize> = BufferDescriptor<W, N>;

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

    fn tx<'a, W, const N: usize>(&'a self, bd: &'a BD<W, N>, option: TransferOption) -> DMABusy<M>
    where
        W: Copy + Default,
    {
        let bytes = bd.take();
        self.reg.memory0_address.store_bits(bytes.as_ptr() as *const _ as u32);
        let msize = mem::size_of::<W>() as u32 - 1;
        self.reg.interrupt_clear.clear_all();
        let num_of_data = cmp::min(bytes.len(), option.size.unwrap_or(bytes.len()));
        self.reg.number_of_data.store_bits(num_of_data as u32);
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

    fn rx<'a, W, const N: usize>(
        &'a self,
        bd: &'a mut BD<W, N>,
        option: TransferOption,
    ) -> DMABusy<M>
    where
        W: Copy + Default,
    {
        let buffer = bd.take();
        self.reg.memory0_address.store_bits(buffer.as_ptr() as *const _ as u32);
        let msize = mem::size_of::<W>() as u32 - 1;
        self.reg.interrupt_clear.clear_all();
        let num_of_data = cmp::min(buffer.len(), option.size.unwrap_or(buffer.len()));
        bd.set_size(num_of_data);
        self.reg.number_of_data.store_bits(num_of_data as u32);
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
            if option.enable_half {
                r.htie().set(v);
            } else {
                r.htie().clear(v);
            }
        });
        DMABusy(self.reg.configuration)
    }

    fn setup_rx<W, const N: usize>(mut self, bd: &'static mut BD<W, N>, option: TransferOption)
    where
        W: Copy + Default,
    {
        self.persist = true;
        self.rx(bd, option);
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

impl<M: DmaChMap> Drop for Stream<M> {
    fn drop(&mut self) {
        if !self.persist && self.is_busy() {
            panic!("DMA dropped while busy")
        }
    }
}

unsafe impl<M: DmaChMap> Send for Stream<M> {}
unsafe impl<M: DmaChMap> Sync for Stream<M> {}
