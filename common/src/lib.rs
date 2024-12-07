#![no_std]
use core::ops::Range;

use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

pub const UART_BAUD_RATE: u32 = 230400;
pub const BLOCK_SIZE: u32 = 512;

#[derive(Serialize, Deserialize, Debug, MaxSize)]
pub enum MessageFromMicrocontroller {
    GetSize,
    Read(Range<u32>),
}

#[derive(Serialize, Deserialize, Debug, MaxSize)]
pub enum MessageFromHost {
    Size(u32),
}

pub const USB_BULK_VID: u16 = 0xc0de;
pub const USB_BULK_PID: u16 = 0xcafe;
