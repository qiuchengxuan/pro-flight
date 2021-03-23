use drone_core::fib::{new_fn, Yielded};
use drone_cortexm::{reg::prelude::*, thr::ThrNvic};
use drone_stm32_map::periph::dma::ch::*;

pub trait Peripheral {
    fn enable_dma(&mut self);
    fn address(&mut self) -> u32;
    fn word_size(&self) -> usize;
}

pub trait DMA: Send {
    fn start(&mut self);
}

pub trait TxDMA<W>: DMA {
    fn setup_peripheral(&mut self, channel: u8, periph: &mut dyn Peripheral);
}

pub trait RxDMA<W>: DMA {
    fn setup_peripheral(&mut self, channel: u8, periph: &mut dyn Peripheral);
    fn on_transfer_complete(&mut self, f: impl FnMut(&[W]) + Send + 'static);
}

pub trait EmptyDMA<W, M: AsMut<[W]>, R: AsRef<[W]>> {
    type RxDMA: RxDMA<W>;
    type TxDMA: TxDMA<W>;
    fn into_rx(self, buffer: M, circle: bool) -> Self::RxDMA;
    fn into_tx(self, bytes: R, repeat: Option<usize>) -> Self::TxDMA;
}

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
    configuration: M::SDmaCcr,
    memory0_address: M::SDmaCm0Ar,
    number_of_data: M::SDmaCndtr,
    peripheral_address: M::SDmaCpar,
    transfer_complete: M::CDmaIfcrCtcif,
    half_transfer: M::CDmaIfcrChtif,
    transfer_error: M::CDmaIfcrCteif,
    direct_mode_error: M::CDmaIfcrCdmeif,
}

impl<M: DmaChMap> Reg<M> {
    fn clear_interrupts(&mut self) {
        self.transfer_complete.set_bit();
        self.half_transfer.set_bit();
        self.transfer_error.set_bit();
        self.direct_mode_error.set_bit();
    }
}

impl<M: DmaChMap> From<DmaChPeriph<M>> for Reg<M> {
    fn from(reg: DmaChPeriph<M>) -> Self {
        Self {
            configuration: reg.dma_ccr,
            memory0_address: reg.dma_cm0ar,
            number_of_data: reg.dma_cndtr,
            peripheral_address: reg.dma_cpar,
            transfer_complete: reg.dma_ifcr_ctcif.into_copy(),
            half_transfer: reg.dma_ifcr_chtif.into_copy(),
            transfer_error: reg.dma_ifcr_cteif.into_copy(),
            direct_mode_error: reg.dma_ifcr_cdmeif.into_copy(),
        }
    }
}

pub struct Channel<M: DmaChMap, INT, B> {
    reg: Reg<M>,
    int: INT,
    buffer: B,
    num_data: usize,
}

impl<M: DmaChMap, INT: ThrNvic> Channel<M, INT, ()> {
    pub fn new(raw: DmaChPeriph<M>, int: INT) -> Self {
        Self { reg: raw.into(), int, buffer: (), num_data: 1 }
    }
}

impl<M: DmaChMap, INT: ThrNvic, W, B, C> EmptyDMA<W, B, C> for Channel<M, INT, ()>
where
    W: 'static + Copy + Send + Sync,
    B: AsMut<[W]> + Send,
    C: AsRef<[W]> + Send,
{
    type RxDMA = Channel<M, INT, B>;
    type TxDMA = Channel<M, INT, C>;

    fn into_rx(self, buffer: B, circle: bool) -> Channel<M, INT, B> {
        let mut channel = Channel { reg: self.reg, int: self.int, buffer: buffer, num_data: 0 };
        let buf = channel.buffer.as_mut();
        channel.reg.memory0_address.store_bits(buf.as_ptr() as *const _ as u32);
        let msize = core::mem::size_of::<B>() as u32 - 1;
        channel.reg.configuration.modify_reg(|r, v| {
            r.minc().set(v);
            if circle {
                r.circ().set(v);
            }
            r.msize().write(v, msize);
        });

        channel.num_data = buf.len();
        channel
    }

    fn into_tx(self, bytes: C, repeat: Option<usize>) -> Channel<M, INT, C> {
        let mut channel = Channel { reg: self.reg, int: self.int, buffer: bytes, num_data: 0 };
        let buf = channel.buffer.as_ref();
        channel.reg.memory0_address.store_bits(buf.as_ptr() as *const _ as u32);
        let msize = core::mem::size_of::<C>() as u32 - 1;
        channel.reg.configuration.modify_reg(|r, v| {
            if repeat.is_some() {
                r.minc().clear(v);
            } else {
                r.minc().set(v);
            }
            r.msize().write(v, msize);
        });
        channel.num_data = repeat.unwrap_or(buf.len());
        channel
    }
}

impl<M: DmaChMap, INT: ThrNvic, B: Send> DMA for Channel<M, INT, B> {
    fn start(&mut self) {
        self.reg.clear_interrupts();
        self.reg.number_of_data.store_bits(self.num_data as u32);
        self.reg.configuration.modify_reg(|r, v| r.en().set(v));
    }
}

impl<M: DmaChMap, INT: ThrNvic, W, B: AsRef<[W]> + Send> TxDMA<W> for Channel<M, INT, B> {
    fn setup_peripheral(&mut self, channel: u8, periph: &mut dyn Peripheral) {
        periph.enable_dma();
        self.reg.peripheral_address.store_bits(periph.address());
        self.reg.configuration.modify_reg(|r, v| {
            r.chsel().write(v, channel as u32);
            r.dir().write(v, Direction::MemoryToPeripheral as u32);
            r.psize().write(v, periph.word_size() as u32 - 1);
        });
    }
}

impl<M: DmaChMap, INT: ThrNvic, W, B: AsMut<[W]> + Send> RxDMA<W> for Channel<M, INT, B>
where
    W: 'static + Copy + Send + Sync,
{
    fn setup_peripheral(&mut self, channel: u8, periph: &mut dyn Peripheral) {
        periph.enable_dma();
        self.reg.peripheral_address.store_bits(periph.address());
        self.reg.configuration.modify_reg(|r, v| {
            r.chsel().write(v, channel as u32);
            r.dir().write(v, Direction::PeripheralToMemory as u32);
            r.psize().write(v, periph.word_size() as u32 - 1);
        });
    }

    fn on_transfer_complete(&mut self, mut f: impl FnMut(&[W]) + Send + 'static) {
        let buffer = self.buffer.as_mut();
        // TODO: remove unsafe
        let buffer = unsafe { core::slice::from_raw_parts(buffer.as_ptr(), buffer.len()) };
        let transfer_complete = self.reg.transfer_complete;
        self.int.add_fib(new_fn(move || {
            transfer_complete.set_bit();
            f(buffer);
            Yielded::<(), ()>(())
        }));
        self.int.enable_int();
        self.reg.configuration.tcie().set_bit();
    }
}
