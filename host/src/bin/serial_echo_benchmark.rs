use std::time::Duration;

use anyhow::{Context, anyhow};
use rand::Rng;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, split},
    time::Instant,
    try_join,
};
use tokio_serial::{SerialPortBuilderExt, SerialPortType};

// Set these to what you want to benchmark
const USB_VID: u16 = 0x16c0;
const USB_PID: u16 = 0x27dd;
// const USB_VID: u16 = 0x10c4;
// const USB_PID: u16 = 0xea60;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ports = tokio_serial::available_ports()?
        .into_iter()
        .filter(|port| match &port.port_type {
            SerialPortType::UsbPort(info) => info.vid == USB_VID && info.pid == USB_PID,
            _ => false,
        })
        .collect::<Vec<_>>();
    if ports.len() > 1 {
        println!("{ports:?}");
        Err(anyhow!("Too many ports. Don't know which one to choose."))
    } else {
        // Total data transfer: 10MiB
        const BUFFER_LEN: usize = 64;
        let count = 163_840;
        // let count = 100;
        let mut write_buffer = [u8::default(); BUFFER_LEN];
        rand::thread_rng().fill(&mut write_buffer);
        let port = ports.get(0).context("No available port")?;
        let port = tokio_serial::new(&port.port_name, 921600)
            .timeout(Duration::from_secs(1))
            .open_native_async()
            .context("Failed to open serial port")?;
        let (mut port_read, mut port_write) = split(port);
        let read_future = async {
            let mut buffer = [u8::default(); BUFFER_LEN];
            for _ in 0..count {
                port_read.read_exact(&mut buffer).await?;
                if buffer == write_buffer {
                    Ok(())
                } else {
                    Err(anyhow!("Didn't receive the same data that was sent."))
                }?;
            }
            let done_receiving_time = Instant::now();
            println!("Received");
            Ok::<_, anyhow::Error>(done_receiving_time)
        };
        let write_future = async {
            let start_time = Instant::now();
            for _ in 0..count {
                port_write.write_all(&write_buffer).await?;
            }
            println!("Wrote");
            Ok(start_time)
        };
        let (start_writing_time, done_receiving_time) = try_join!(write_future, read_future)?;
        let duration = done_receiving_time - start_writing_time;
        let total_bytes = BUFFER_LEN * count;
        println!("Duration: {duration:?}. Data: {total_bytes} bytes");
        Ok(())
    }
}
