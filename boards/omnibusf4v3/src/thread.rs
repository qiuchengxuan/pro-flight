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
            8: pub bmp280;
            9: pub max7456;
            10: pub mpu6000;
            11: pub dma1_stream0;
            16: pub dma1_stream5;
            56: pub dma2_stream0;
            58: pub dma2_stream2;
            59: pub dma2_stream3;
            67: pub otg_fs;
        }
    };
}

macro_rules! priority {
    ($pri:expr) => {{
        let pri: u8 = $pri.into();
        20u8 + pri
    }};
}

pub fn setup_priority(thread: &mut Thrs) {
    thread.dma_1_stream_0.set_priority(priority!(Priority::System));
    thread.dma_1_stream_5.set_priority(priority!(Priority::System));
    thread.dma_2_stream_0.set_priority(priority!(Priority::System));
    thread.dma_2_stream_3.set_priority(priority!(Priority::System));
    thread.bmp_280.set_priority(priority!(Priority::Sensor));
    thread.mpu_6000.set_priority(priority!(Priority::Sensor));
    thread.max_7456.set_priority(priority!(Priority::Telemetry));
    thread.otg_fs.set_priority(priority!(Priority::Interactive));
}
