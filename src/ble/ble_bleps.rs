use embedded_io_async::{Read, Write};
use bleps::{ad_structure::{create_advertising_data, AdStructure, BR_EDR_NOT_SUPPORTED, LE_GENERAL_DISCOVERABLE}, att::Uuid as Uuid, Ble};

pub fn uuid_from_str(value: &str) -> Uuid {
    if value.len() == 4 {
        return Uuid::Uuid16(u16::from_str_radix(value, 16).unwrap());
    }
    let uuid = uuid::Uuid::parse_str(value).unwrap();
    let bytes = uuid.as_bytes();
    bytes[..].into()
}

pub async fn ble_adv<T, F>(ble: &mut Ble<T, F>, uuid: Uuid, name: &str)
where T: Read + Write,
    F: Fn() -> u64,
{
    ble.init().await.unwrap();
    ble.cmd_set_le_advertising_parameters().await.unwrap();
    let Uuid::Uuid16(_) = uuid else {
        panic!("ble_adv: need to be uuid16");
    };

    ble.cmd_set_le_advertising_data(
        create_advertising_data(&[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::ServiceUuids16(&[uuid]),
            AdStructure::CompleteLocalName(name),
        ]).unwrap()
    ).await.unwrap_or_else(|e| panic!("{}", e));
    ble.cmd_set_le_advertise_enable(true).await.unwrap();
}

// #[derive(Default)]
// struct GATTAttrValue {
//     bytes: Vec<u8>,
//     readable: bool,
//     writable: bool,
// }

// impl AttData for GATTAttrValue {
//     fn readable(&self) -> bool {
//         self.readable
//     }

//     fn read(&mut self, offset: usize, data: &mut [u8]) -> Result<usize, AttErrorCode> {
//         let n = self.bytes.len();
//         if offset > n {
//             return Ok(0);
//         }
//         let len = data.len().min(n - offset);
//         if len > 0 {
//             data[..len].copy_from_slice(&self.bytes[offset..offset + len]);
//         }
//         Ok(len)
//     }

//     fn writable(&self) -> bool {
//         self.writable
//     }

//     fn write(&mut self, offset: usize, data: &[u8]) -> Result<(), AttErrorCode> {
//         let n = self.bytes.len();
//         if offset > n {
//             return Ok(());
//         }
//         let len = data.len().min(n - offset);
//         if len > 0 {
//             self.bytes[offset..offset + len].copy_from_slice(&data[..len]);
//         }
//         Ok(())
//     }
// }

// #[self_referencing]
// pub struct GATTOwnedAttr {
//     data: GATTAttrValue,

//     #[borrows(mut data)]
//     #[not_covariant]
//     attr: Attribute<'this>,
// }

// fn ble_build_attr_value(uuid: Uuid, data: GATTAttrValue) -> GATTOwnedAttr {
//     GATTOwnedAttrBuilder {
//         data,
//         attr_builder: |x: &mut GATTAttrValue| Attribute::new(uuid, x),
//     }.build()
// }

// pub fn ble_build_primary_service(uuid: BleUuid) -> GATTOwnedAttr {
//     let data = GATTAttrValue {
//         bytes: uuid.into(),
//         readable: true,
//         writable: false,
//     };
//     ble_build_attr_value(PRIMARY_SERVICE_UUID16, data)
// }

// pub fn ble_build_characteristic(uuid: BleUuid) -> BleOwnedAttr {
//     let data = BleAttrSrvData {
//         bytes: uuid.into(),
//         readable: true,
//         writable: true,
//     };
//     ble_build_attr_bytes(PRIMARY_SERVICE_UUID16, data)
// }