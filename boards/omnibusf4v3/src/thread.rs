//! The threads.

use drone_cortexm::thr::{self, ThrNvic};
pub use drone_cortexm::thr::{init, init_extended};
pub use drone_stm32_map::thr::*;
use pro_flight::task::Priority;

thr::nvic! {
    /// The thread data.
    thread => pub Thr {};

    /// The thread-local storage.
    local => pub ThrLocal {};

    /// The vector table type.
    vtable => pub Vtable;

    /// A set of thread tokens.
    index => pub Thrs;

    /// Threads initialization token.
    init => pub ThrsInit;

    threads => {
        exceptions => {
            pub hard_fault;
            pub sys_tick;
        };
        interrupts => {
            5: pub rcc;
            7: pub fcs; // exti1
            8: pub bmp280; // exti2
            9: pub max7456; // exti3
            10: pub mpu6000; // exti4
            11: pub dma1_stream0; // BMP280/MAX7456 rx
            16: pub dma1_stream5; // BMP280 tx
            23: pub ins; // exti5-9
            56: pub dma2_stream0; // mpu6000 rx
            57: pub dma2_stream1; // USART3/I2C-2
            58: pub dma2_stream2; // ADC2
            59: pub dma2_stream3; // mpu6000 tx
            61: pub dma2_stream5; // USART1 rx
            67: pub otg_fs;
        }
    };
}

macro_rules! priority {
    ($pri:expr) => {{
        let pri: u8 = $pri.into();
        0x10 + (pri << 4)
    }};
}

pub fn setup_priority(threads: &mut Thrs) {
    threads.otg_fs.set_priority(priority!(Priority::Immediate));
    threads.fcs.set_priority(priority!(Priority::Immediate));
    threads.dma1_stream0.set_priority(priority!(Priority::System));
    threads.dma1_stream5.set_priority(priority!(Priority::System));
    threads.dma2_stream0.set_priority(priority!(Priority::System));
    threads.dma2_stream1.set_priority(priority!(Priority::System));
    threads.dma2_stream3.set_priority(priority!(Priority::System));
    threads.dma2_stream5.set_priority(priority!(Priority::System));
    threads.bmp280.set_priority(priority!(Priority::Sensor));
    threads.mpu6000.set_priority(priority!(Priority::Sensor));
    threads.ins.set_priority(priority!(Priority::Normal));
    threads.max7456.set_priority(priority!(Priority::Telemetry));
}
