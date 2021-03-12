use drone_core::fib::{new_fn, Yielded};
use drone_cortexm::{reg::prelude::*, thr::ThrNvic};
use drone_stm32_map::periph::dma::ch::*;

pub trait Peripheral {
    fn enable_dma(&mut self);
    fn address(&self) -> u32;
}

pub trait Transmit {
    fn setup_memory(&mut self, buffer: &'static [u8], repeat: Option<usize>);
    fn on_finished(&mut self, f: impl FnMut() + Send + 'static);
}

pub trait Receive {
    fn setup_memory(&mut self, buffer: &'static mut [u8]) -> &mut Self;
    fn on_finished(&mut self, f: impl FnMut(&[u8]) + Send + 'static);
}

pub trait Channel {
    fn setup_peripheral(&mut self, chan: u8, periph: &mut dyn Peripheral);
    fn start(&mut self);
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
    cr: M::SDmaCcr,
    m0ar: M::SDmaCm0Ar,
    ndtr: M::SDmaCndtr,
    par: M::SDmaCpar,
    tcif: M::CDmaIfcrCtcif,
}

impl<M: DmaChMap> From<DmaChPeriph<M>> for Reg<M> {
    fn from(reg: DmaChPeriph<M>) -> Self {
        Self {
            cr: reg.dma_ccr,
            m0ar: reg.dma_cm0ar,
            ndtr: reg.dma_cndtr,
            par: reg.dma_cpar,
            tcif: reg.dma_ifcr_ctcif.into_copy(),
        }
    }
}

pub struct TxDMA<M: DmaChMap, INT: ThrNvic> {
    reg: Reg<M>,
    int: INT,
    num_data: usize,
}

impl<M: DmaChMap, INT: ThrNvic> TxDMA<M, INT> {
    pub fn new(raw: DmaChPeriph<M>, int: INT) -> Self {
        Self { reg: raw.into(), int, num_data: 1 }
    }
}

impl<M: DmaChMap, INT: ThrNvic> Transmit for TxDMA<M, INT> {
    fn setup_memory(&mut self, bytes: &'static [u8], repeat: Option<usize>) {
        self.reg.m0ar.store_bits(bytes.as_ptr() as *const _ as u32);
        self.reg.cr.modify_reg(|r, v| {
            if repeat.is_some() {
                r.minc().clear(v);
            } else {
                r.minc().set(v);
            }
        });
        self.num_data = repeat.unwrap_or(bytes.len());
    }

    fn on_finished(&mut self, mut f: impl FnMut() + Send + 'static) {
        let tcif = self.reg.tcif;
        self.int.add_fib(new_fn(move || {
            tcif.set_bit();
            f();
            Yielded::<(), ()>(())
        }));
        self.int.enable_int();
        self.reg.cr.modify_reg(|r, v| r.tcie().set(v));
    }
}

impl<M: DmaChMap, INT: ThrNvic> Channel for TxDMA<M, INT> {
    fn setup_peripheral(&mut self, channel: u8, periph: &mut dyn Peripheral) {
        periph.enable_dma();
        self.reg.par.store_bits(periph.address());
        self.reg.cr.modify_reg(|r, v| {
            r.chsel().write(v, channel as u32);
            r.dir().write(v, Direction::MemoryToPeripheral as u32);
        });
    }

    fn start(&mut self) {
        self.reg.ndtr.store_bits(self.num_data as u32);
        self.reg.cr.modify_reg(|r, v| r.en().set(v));
    }
}

pub struct RxDMA<M: DmaChMap, INT: ThrNvic> {
    reg: Reg<M>,
    int: INT,
    buffer: &'static mut [u8],
}

impl<M: DmaChMap, INT: ThrNvic> RxDMA<M, INT> {
    pub fn new(raw: DmaChPeriph<M>, int: INT) -> Self {
        Self { reg: raw.into(), int, buffer: &mut [] }
    }
}

impl<M: DmaChMap, INT: ThrNvic> Receive for RxDMA<M, INT> {
    fn setup_memory(&mut self, buffer: &'static mut [u8]) -> &mut Self {
        self.reg.cr.modify_reg(|r, v| r.minc().set(v));
        self.reg.m0ar.store_bits(buffer.as_ptr() as *const _ as u32);
        self.buffer = buffer;
        self
    }

    fn on_finished(&mut self, mut f: impl FnMut(&[u8]) + Send + 'static) {
        let buffer = // XXX: make DMA real thread-safe
            unsafe { core::slice::from_raw_parts(self.buffer.as_ptr(), self.buffer.len()) };
        let tcif = self.reg.tcif;
        self.int.add_fib(new_fn(move || {
            tcif.set_bit();
            f(buffer);
            Yielded::<(), ()>(())
        }));
        self.int.enable_int();
        self.reg.cr.modify_reg(|r, v| r.tcie().set(v));
    }
}

impl<M: DmaChMap, INT: ThrNvic> Channel for RxDMA<M, INT> {
    fn setup_peripheral(&mut self, chan: u8, periph: &mut dyn Peripheral) {
        periph.enable_dma();
        self.reg.par.store_bits(periph.address());
        self.reg.cr.modify_reg(|r, v| {
            r.chsel().write(v, chan as u32);
            r.dir().write(v, Direction::PeripheralToMemory as u32);
        });
    }

    fn start(&mut self) {
        self.reg.ndtr.store_bits(self.buffer.len() as u32);
        self.reg.cr.modify_reg(|r, v| r.en().set(v));
    }
}

pub struct TRxDMA<TX: Transmit, RX: Receive> {
    pub tx: TX,
    pub rx: RX,
}

impl<TX: Transmit + Channel, RX: Receive + Channel> TRxDMA<TX, RX> {
    pub fn start(&mut self) {
        self.rx.start();
        self.tx.start();
    }
}
