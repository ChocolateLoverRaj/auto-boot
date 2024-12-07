use std::time::Duration;

use anyhow::{Context, anyhow};
use tokio::{io::AsyncWriteExt, time::Instant};
use tokio_serial::{SerialPortBuilderExt, SerialPortType};

// Set these to what you want to benchmark
const USB_VID: u16 = 0x16c0;
const USB_PID: u16 = 0x27dd;

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
        let port = ports.get(0).context("No available port")?;
        let mut port = tokio_serial::new(&port.port_name, u32::MAX)
            .timeout(Duration::from_secs(1))
            .open_native_async()
            .context("Failed to open serial port")?;
        // Total data transfer: 10MiB
        const BUFFER_LEN: usize = 64;
        let count = 163_840;
        let write_buffer = [u8::default(); BUFFER_LEN];
        let start_time = Instant::now();
        for _ in 0..count {
            port.write_all(&write_buffer).await?;
        }
        let finish_time = Instant::now();
        let duration = finish_time - start_time;
        let total_bytes = BUFFER_LEN * count;
        println!("Duration: {duration:?}. Data: {total_bytes} bytes");
        Ok(())
    }
}
