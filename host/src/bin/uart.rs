use std::{io::SeekFrom, os::unix::fs::MetadataExt, time::Duration};

use anyhow::{Context, anyhow};
use common::{MessageFromHost, MessageFromMicrocontroller, UART_BAUD_RATE};
use tokio::{
    fs::{OpenOptions, metadata},
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
};
use tokio_serial::{SerialPortBuilderExt, SerialPortType, SerialStream};

// TODO: not fixed
const IMG_PATH: &str = "/home/rajas/Documents/code-runner/target/debug/build/code-runner-7038142294e01598/out/code-runner-uefi.img";
// const IMG_PATH: &str = "/home/rajas/Documents/auto-boot/disk.img";

async fn receive_message(port: &mut SerialStream) -> anyhow::Result<MessageFromMicrocontroller> {
    let length = port
        .read_u32()
        .await
        .context("Failed to read message length")?;
    let mut buffer = vec![u8::default(); length as usize];
    // println!("Reading exact: {length}");
    port.read_exact(&mut buffer)
        .await
        .context("Failed to read message")?;
    let message = postcard::from_bytes(&buffer).context("Failed to parse message")?;
    Ok(message)
}

async fn send_message(port: &mut SerialStream, message: &MessageFromHost) -> anyhow::Result<()> {
    let buffer = postcard::to_allocvec(&message)?;
    port.write_u32(buffer.len() as u32).await?;
    port.write_all(&buffer).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ports = tokio_serial::available_ports()?
        .into_iter()
        .filter(|port| match &port.port_type {
            // UART to USB converter
            // 10c4:ea60 Silicon Labs CP210x UART Bridge
            // This can be changed
            SerialPortType::UsbPort(info) => info.vid == 0x10c4 && info.pid == 0xea60,
            _ => false,
        })
        .collect::<Vec<_>>();
    if ports.len() > 1 {
        println!("{ports:?}");
        Err(anyhow!("Too many ports. Don't know which one to choose."))
    } else {
        let port = ports.get(0).context("No available port")?;
        let mut port = tokio_serial::new(&port.port_name, UART_BAUD_RATE)
            .timeout(Duration::from_secs(1))
            .open_native_async()
            .context("Failed to open serial port")?;

        loop {
            match receive_message(&mut port).await? {
                MessageFromMicrocontroller::GetSize => {
                    let metadata = metadata(IMG_PATH).await?;
                    let size = metadata.size() as u32;
                    println!("Requested size. Sending size: {size}");
                    send_message(&mut port, &MessageFromHost::Size(size)).await?;
                }
                MessageFromMicrocontroller::Read(range) => {
                    // println!("Reading {range:?}");
                    let mut file = OpenOptions::new()
                        .read(true)
                        .write(false)
                        .open(IMG_PATH)
                        .await?;
                    file.seek(SeekFrom::Start(range.start as u64)).await?;
                    let mut buffer = vec![u8::default(); (range.end - range.start) as usize];
                    file.read_exact(&mut buffer).await?;
                    port.write_all(&buffer).await?;
                }
                message => {
                    println!("Unknown message: {message:?}")
                }
            }
        }
        // Ok(())
    }
}
