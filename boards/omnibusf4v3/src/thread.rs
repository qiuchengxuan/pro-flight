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
            7: pub servo; // exti1
            8: pub bmp280; // exti2
            9: pub max7456; // exti3
            10: pub mpu6000; // exti4
            11: pub dma1_stream0;
            16: pub dma1_stream5;
            56: pub dma2_stream0;
            57: pub dma2_stream1;
            58: pub dma2_stream2;
            59: pub dma2_stream3;
            61: pub dma2_stream5;
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

pub fn setup_priority(thread: &mut Thrs) {
    thread.otg_fs.set_priority(priority!(Priority::Immediate));
    thread.dma1_stream0.set_priority(priority!(Priority::System));
    thread.dma1_stream5.set_priority(priority!(Priority::System));
    thread.dma2_stream0.set_priority(priority!(Priority::System));
    thread.dma2_stream1.set_priority(priority!(Priority::Sensor));
    thread.dma2_stream3.set_priority(priority!(Priority::System));
    thread.dma2_stream5.set_priority(priority!(Priority::Sensor));
    thread.servo.set_priority(priority!(Priority::Immediate));
    thread.bmp280.set_priority(priority!(Priority::Sensor));
    thread.mpu6000.set_priority(priority!(Priority::Sensor));
    thread.max7456.set_priority(priority!(Priority::Telemetry));
}
