#![no_std]
#![no_main]
#![cfg(feature = "trouble")]

#[macro_use]
extern crate alloc;

use esp_backtrace as _;
use esp_println as _;

use rmtled::{alloc::init_heap, ble::{ble_create_controller, uuid_from_str}, create_ble_connector_rw, init_embassy};

use embassy_futures::{join::{join, join3}, select::{select, select3}};
use embassy_time::{Duration, Timer};
use pin_utils::pin_mut;
use static_cell::StaticCell;

use embassy_executor::Spawner;

use defmt::{debug, info, println};
use esp_hal::{
    clock::ClockControl, efuse::Efuse, peripherals::Peripherals, prelude::*, rng::Rng, system::SystemControl, timer::{systimer::SystemTimer, timg::TimerGroup}
};

use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use esp_println as _;
use trouble_host::{advertise::{AdStructure, Advertisement, BR_EDR_NOT_SUPPORTED, LE_GENERAL_DISCOVERABLE}, attribute::{AttributeTable, CharacteristicProp, Service}, gatt::GattEvent, Address, BleHost, BleHostResources, PacketQos};

const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 3;

#[toml_cfg::toml_config]
pub struct Config<'a> {
    #[default("180A")]
    service_uuid: &'static str,
    #[default("180A")]
    value_uuid: &'static str,
}

#[main]
async fn main(_spawner: Spawner) {
    init_heap();

    let ph = Peripherals::take();
    let clocks = ClockControl::max(SystemControl::new(ph.SYSTEM).clock_control).freeze();

    let mut rng = Rng::new(ph.RNG);

    let alarm0 = SystemTimer::new(ph.SYSTIMER).alarm0.into();
    init_embassy(alarm0, &clocks);

    let timer0 = TimerGroup::new(ph.TIMG0, &clocks, None)
        .timer0
        .into();

    let (ble_conn_r, ble_conn_w) = create_ble_connector_rw(
        timer0,
        &clocks,
        rng,
        ph.RADIO_CLK,
        ph.BT,
    );

    let mac = Efuse::get_mac_address();
    let dev_name = format!("vf-led-{}", hex::encode(&mac[3..6]));
    info!("{}", dev_name.as_str());

    let service_uuid = uuid_from_str(CONFIG.service_uuid);
    let value_uuid = uuid_from_str(CONFIG.value_uuid);

    let ble_controller = ble_create_controller(ble_conn_r, ble_conn_w);

    static RESOURCES: StaticCell<BleHostResources<CONNECTIONS_MAX, L2CAP_CHANNELS_MAX, 27>> = StaticCell::new();
    let host_resources = RESOURCES.init(BleHostResources::new(PacketQos::None));
    let mut adapter = BleHost::new(ble_controller, host_resources);

    let peripheral_address = Address::random(mac);
    adapter.set_random_address(peripheral_address);
    let mut table: AttributeTable<'_, NoopRawMutex, 10> = AttributeTable::new();

    let id = b"Trouble";
    let appearance = [0x80, 0x07];
    let mut value = [rng.random() as u8; 1];
    let mut expected = value[0].wrapping_add(1);

    let handle = {
        let mut svc = table.add_service(Service::new(0x1800));
        let _ = svc.add_characteristic_ro(0x2a00, id);
        let _ = svc.add_characteristic_ro(0x2a01, &appearance[..]);
        svc.build();

        // Generic attribute service (mandatory)
        table.add_service(Service::new(0x1801));

        // Custom service
        let mut svc = table.add_service(Service::new(service_uuid.clone()));

        svc.add_characteristic(
            value_uuid.clone(),
            &[CharacteristicProp::Read, CharacteristicProp::Write, CharacteristicProp::Notify],
            &mut value,
        )
        .build()
    };

    let server = adapter.gatt_server(&table);

    let adapter_fut = adapter.run();
    pin_mut!(adapter_fut);

    let server_fut = async {
        let mut writes = 0;
        loop {
            match server.next().await {
                Ok(GattEvent::Write {
                    connection: _,
                    handle,
                }) => {
                    let _ = table.get(handle, |value| {
                        defmt::assert_eq!(expected, value[0]);
                        expected += 1;
                        writes += 1;
                    });
                    if writes == 2 {
                        println!("expected value written twice, test pass");
                        break;
                    }
                }
                Ok(_) => {},
                Err(e) => {
                    println!("Error processing GATT events: BleConnectorError ");
                    return Err(e);
                }
            }
        }
        Ok(())
    };
    pin_mut!(server_fut);


    let adv_fut = async {
        loop {
            let mut adv_data = [0; 31];
            AdStructure::encode_slice(
                &[AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED)],
                &mut adv_data[..],
            ).unwrap();

            let mut scan_data = [0; 31];
            AdStructure::encode_slice(
                &[AdStructure::CompleteLocalName(dev_name.as_bytes())],
                &mut scan_data[..],
            ).unwrap();

            let mut acceptor = adapter.advertise(&Default::default(), Advertisement::ConnectableScannableUndirected {
                adv_data: &adv_data[..],
                scan_data: &scan_data[..],
            }).await.unwrap();

            let conn = acceptor.accept().await.unwrap();
            let mut counter = 0;
            loop {
                Timer::after(Duration::from_secs(1)).await;
                counter += 1;
                if !conn.is_connected() {
                    break
                }
                info!("updating value {}", counter);
                server.notify(handle, &conn, &[counter]).await.unwrap()

            }
        }
    };
    pin_mut!(adv_fut);

    select3(adapter_fut, server_fut, adv_fut).await;
}
