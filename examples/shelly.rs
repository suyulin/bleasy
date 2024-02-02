//! This example listens for BLE device events.

use bleasy::{Device, DeviceEvent, Error, ScanConfig, Scanner};
use futures::StreamExt;
use serde_json::json;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

const UUID_RW: Uuid = Uuid::from_u128(0x5F6D4F53_5F52_5043_5F64_6174615F5F5F);
const UUID_READ_NOTIFY: Uuid = Uuid::from_u128(0x5F6D4F53_5F52_5043_5F72_785F63746C5F);
const UUID_W: Uuid = Uuid::from_u128(0x5F6D4F53_5F52_5043_5F74_785F63746C5F);
const DEVICE_NAME: &str = "ShellyPlusPlugS-80646FD63E48";
#[tokio::main]
async fn main() -> Result<(), Error> {
    pretty_env_logger::init();
    let mut scanner = Scanner::new();

    scanner
        .start(
            ScanConfig::default()
                .filter_by_name(|x| x.to_string().eq("ShellyPlusPlugS-80646FD63E48")),
        )
        .await?;

    let device = scanner.device_stream().next().await.unwrap();
    println!("device: {:?}", device.local_name().await);
    let data = json!({
        "id": 1641784978,
        "method": "Wifi.GetStatus"
    });
    let data = serde_json::to_vec(&data).unwrap();
    request_data(&device, &data).await?;
    Ok(())
}
async fn request_data(client: &Device, request_data: &[u8]) -> Result<(), Error> {
    let rw_char = client.characteristic(UUID_RW).await?.unwrap();
    let w_char = client.characteristic(UUID_W).await?.unwrap();
    let read_char = client.characteristic(UUID_READ_NOTIFY).await?.unwrap();

    // Write the length of the request data to the write characteristic
    let data_length = (request_data.len() as u32).to_be_bytes().to_vec();
    w_char.write_request(&data_length).await?;

    // Wait for a second
    sleep(Duration::from_secs(1)).await;

    // Write the request data to the rw characteristic
    rw_char.write_request(request_data).await?;

    // Wait for a second
    sleep(Duration::from_secs(1)).await;

    // Read the response length from the read characteristic
    let response_length = read_char.read().await?;
    let response_length = u32::from_be_bytes(<[u8; 4]>::try_from(&response_length[..4]).unwrap());

    // Read the response data from the rw characteristic
    let mut response_data = Vec::new();
    while response_data.len() < response_length as usize {
        let part = rw_char.read().await?;
        response_data.extend_from_slice(&part);
        sleep(Duration::from_secs(1)).await;
    }

    println!(
        "Response data: {:?}",
        String::from_utf8_lossy(&response_data)
    );

    Ok(())
}
