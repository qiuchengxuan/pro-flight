use alloc::boxed::Box;

use hal::dma::{BufferDescriptor, Peripheral, TransferOption, DMA};
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
            wordlength: WordLength::DataBits8,
            dma: DmaConfig::TxRx,
        },
    }
}

pub fn init<F, D>(mut usart: USART, mut dma: D, channel: u8, mut rx: Box<dyn Receiver>)
where
    USART: Peripheral + Display,
    D: DMA<Future = F>,
{
    let receive_size = rx.receive_size();
    let mut rx_bd = Box::new(BufferDescriptor::<u8, 64>::default());
    let address = rx_bd.try_get_buffer().unwrap().as_ptr();
    debug!("Init {} DMA address at 0x{:x}", usart, address as usize);
    rx_bd.set_handler(move |result| rx.receive(result.into()));

    dma.setup_peripheral(channel, &mut usart);
    dma.setup_rx(Box::leak(rx_bd), TransferOption::circle().size(receive_size));
}
