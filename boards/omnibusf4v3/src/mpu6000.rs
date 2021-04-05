use core::fmt::Debug;

use chips::stm32f4::delay::TickDelay;
use drivers::mpu6000::{DmaMPU6000, MPU6000Init};
use drone_core::fib::{new_fn, Yielded};
use drone_cortexm::thr::ThrNvic;
use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;
use hal::dma::{Peripheral, DMA};
pub use mpu6000::SPI_MODE;
use mpu6000::{bus::SpiBus, MPU6000};
use pro_flight::datastructures::measurement::{Acceleration, Measurement};
use stm32f4xx_hal::gpio::ExtiPin;

const SAMPLE_RATE: usize = 1000;

pub struct DmaSpiMPU6000<SPI, CS, INT, RX, TX, THR> {
    pub spi: SPI,
    pub cs: CS,
    pub int: INT,
    pub rx: RX,
    pub tx: TX,
    pub thread: THR,
}

impl<E: Debug, PE: Debug, SPI, CS, INT, RXF, TXF, RX, TX, THR>
    DmaSpiMPU6000<SPI, CS, INT, RX, TX, THR>
where
    SPI: Transfer<u8, Error = E> + Write<u8, Error = E> + Peripheral + Send + 'static,
    CS: OutputPin<Error = PE> + Send + 'static + Unpin,
    INT: ExtiPin + Send + 'static,
    RX: DMA<Future = RXF>,
    TX: DMA<Future = TXF>,
    THR: ThrNvic,
{
    pub fn init(self, handler: impl FnMut(Acceleration, Measurement) + 'static + Send) {
        let mut mpu6000 = MPU6000::new(SpiBus::new(self.spi, self.cs, TickDelay {}));
        if let Some(error) = mpu6000.init(SAMPLE_RATE as u16).err() {
            error!("MPU6000 init failed: {:?}", error);
            return;
        }

        info!("MPU6000 detected");
        let (mut spi, cs, _) = mpu6000.free().free();

        let mut mpu6000 = DmaMPU6000::new(cs, handler);
        let (mut rx, mut tx) = (self.rx, self.tx);
        rx.setup_peripheral(3, &mut spi);
        tx.setup_peripheral(3, &mut spi);
        let mut int = self.int;

        self.thread.add_fib(new_fn(move || {
            int.clear_interrupt_pending_bit();
            mpu6000.trigger(&rx, &tx);
            Yielded::<(), ()>(())
        }));
        self.thread.enable_int()
    }
}
