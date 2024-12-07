use anyhow::Context;
use tokio::time::Instant;
use uom::si::{
    f64::Information,
    information::{byte, mebibyte},
};

const BULK_OUT_EP: u8 = 0x01;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let di = nusb::list_devices()
        .unwrap()
        .find(|d| d.vendor_id() == 0xc0de && d.product_id() == 0xcafe)
        .expect("no device found");
    let device = di.open().context("error opening device")?;
    let interface = device
        .claim_interface(0)
        .context("error claiming interface")?;

    let bytes_to_send = Information::new::<mebibyte>(10.0).get::<byte>() as usize;
    let data_to_send = vec![u8::default(); bytes_to_send];
    let before = Instant::now();
    let result = interface.bulk_out(BULK_OUT_EP, data_to_send).await;
    let after = Instant::now();
    println!("{result:?}");
    let duration = after - before;
    println!("Sent {bytes_to_send} bytes in {duration:?}");
    Ok(())
}
