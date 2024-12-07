use std::{fs::OpenOptions, io::Read, time::Instant};

use anyhow::Context;
use uom::si::{
    f64::{Information, Time},
    information::{byte, mebibyte},
};

/// Benchmarks a block device to see how fast it can read a certain amount of data
fn main() -> anyhow::Result<()> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(false)
        .open("/dev/sda")
        .context("Error opening block device")?;
    let total_read = Information::new::<mebibyte>(10.0);
    let total_read_bytes = total_read.get::<byte>() as usize;
    let mut buffer = vec![
        0;
        Information::new::<mebibyte>(100.0)
            .min(total_read)
            .get::<byte>() as usize
    ];
    let mut bytes_read = 0;
    let start = Instant::now();
    loop {
        let count = file.read(&mut buffer)?;
        bytes_read += count;
        if count == 0 || bytes_read >= total_read_bytes {
            break;
        }
    }
    let bytes_read = Information::new::<byte>(bytes_read as f64);
    let end = Instant::now();
    let read_time = Time::try_from(end - start).unwrap();
    let speed = bytes_read / read_time;
    println!("Read (bytes) {bytes_read:?} in {read_time:?}. Speed (bytes): {speed:?}");
    Ok(())
}
