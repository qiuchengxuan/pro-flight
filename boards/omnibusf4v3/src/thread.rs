//! The threads.

pub use drone_cortexm::thr::{init, init_extended};
pub use drone_stm32_map::thr::*;

use drone_cortexm::thr;

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
            8: pub exti2;
            9: pub exti3;
            10: pub exti4;
            11: pub dma1_stream0;
            16: pub dma1_stream5;
            35: pub spi1;
            51: pub spi3;
            56: pub dma2_stream0;
            58: pub dma2_stream2;
            59: pub dma2_stream3;
            67: pub otg_fs;
        }
    };
}
