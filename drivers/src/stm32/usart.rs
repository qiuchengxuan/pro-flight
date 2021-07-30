use alloc::boxed::Box;

use embedded_hal::serial;
use hal::dma::{BufferDescriptor, Peripheral, TransferOption, DMA};
use hal::serial::Error;
use pro_flight::config::SerialConfig;
use pro_flight::protocol::serial::Receiver;
use stm32f4xx_hal::{
    prelude::*,
    serial::config::{Config, DmaConfig, Parity, StopBits, WordLength},
};

pub fn to_serial_config(config: &SerialConfig) -> Config {
    match config {
        SerialConfig::GNSS(gnss) => {
            Config { baudrate: gnss.baudrate.bps(), dma: DmaConfig::Rx, ..Default::default() }
        }
        SerialConfig::SBUS(sbus) => Config {
            baudrate: sbus.baudrate().bps(),
            stopbits: StopBits::STOP2,
            parity: Parity::ParityEven,
            wordlength: WordLength::DataBits9, // actually 8 data bits with 1 parity bit
            dma: DmaConfig::TxRx,
        },
    }
}

pub fn init<USART, F, D>(mut usart: USART, mut dma: D, channel: u8, mut rx: Box<dyn Receiver>)
where
    USART: Peripheral + core::fmt::Display + serial::Read<u8, Error = Error> + Send + 'static,
    D: DMA<Future = F>,
{
    dma.setup_peripheral(channel, &mut usart);
    let receive_size = rx.receive_size();
    let mut rx_bd = Box::new(BufferDescriptor::<u8, 64>::default());
    let address = rx_bd.try_get_buffer().unwrap().as_ptr();
    debug!("Init {} DMA address at 0x{:x}", usart, address as usize);
    rx_bd.set_callback(move |result| match usart.read() {
        Err(nb::Error::Other(Error::Parity)) => rx.reset(),
        Err(nb::Error::Other(Error::Framing)) => {
            rx.receive(result.into());
            rx.reset();
        }
        _ => rx.receive(result.into()),
    });

    dma.setup_rx(Box::leak(rx_bd), TransferOption::circle().size(receive_size));
}
