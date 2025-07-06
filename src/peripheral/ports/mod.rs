pub mod sio0;

use modular_bitfield::{bitfield, prelude::*};

#[derive(Clone, Copy)]
#[bitfield]
struct SIO_STAT {
    tx_fifo_not_full: B1,
    rx_fifo_not_empty: B1,
    tx_idle: B1,
    rx_parity_error: B1,
    sio1_rx_fifo_overrun: B1,
    sio1_rx_bad_stop_bit: B1,
    sio1_rx_input_level: B1,
    dsr_input_level: B1,
    sio1_cts_input_level: B1,
    irq: B1,
    always_zero: B1,
    baudrate_timer: B21,
}

#[derive(Clone, Copy)]
#[bitfield]
struct SIO_MODE {
    baudrate_reload_factor: B2,
    char_len: B2,
    parity_enable: B1,
    parity_type: B1,
    sio1_stop_bit_length: B2,
    sio0_clock_polarity: B1,
    always_zero: B7,
}

#[derive(Clone, Copy)]
#[bitfield]
struct SIO_CTRL {
    tx_enable: B1,
    dtr_output_level: B1,
    rx_enable: B1,
    sio1_tx_output_level: B1,
    acknowledge: B1,
    sio1_rts_output_level: B1,
    reset: B1,
    sio1_unknown: B1,
    rx_interrupt_mode: B2,
    tx_interrupt_enable: B1,
    rx_interrupt_enable: B1,
    dsr_interrupt_enable: B1,
    sio0_port_select: B1,
    always_zero: B2,
}