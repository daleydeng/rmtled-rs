use bt_hci::{controller::ExternalController, transport::SerialTransport};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embedded_io_async::{Read, Write};
use trouble_host::attribute::Uuid;

pub fn ble_create_controller<R: Read, W: Write>(conn_r: R, conn_w: W) -> ExternalController<SerialTransport<NoopRawMutex, R, W>, 10>
{
    ExternalController::new(SerialTransport::new(conn_r, conn_w))
}

pub fn uuid_from_str(value: &str) -> Uuid {
    if value.len() == 4 {
        return u16::from_str_radix(value, 16).unwrap().into();
    }
    let bytes = uuid::Uuid::parse_str(value).unwrap().into_bytes();
    bytes[..].into()
}