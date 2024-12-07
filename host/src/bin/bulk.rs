use anyhow::Context;
use common::{USB_BULK_PID, USB_BULK_VID};
use nusb::transfer::RequestBuffer;

const BULK_OUT_EP: u8 = 0x01;
const BULK_IN_EP: u8 = 0x81;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let di = nusb::list_devices()
        .unwrap()
        .find(|d| d.vendor_id() == USB_BULK_VID && d.product_id() == USB_BULK_PID)
        .expect("no device found");
    let device = di.open().context("error opening device")?;
    let interface = device
        .claim_interface(0)
        .context("error claiming interface")?;

    let result = interface.bulk_out(BULK_OUT_EP, b"hello world".into()).await;
    println!("{result:?}");
    let result = interface.bulk_in(BULK_IN_EP, RequestBuffer::new(64)).await;
    println!("{result:?}");
    Ok(())
}
