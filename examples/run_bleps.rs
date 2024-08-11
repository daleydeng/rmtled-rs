#![no_std]
#![no_main]
#![cfg(feature = "bleps")]

use core::cell::RefCell;

use esp_backtrace as _;
use esp_println as _;

use bleps::{attribute_server::{AttributeServer, NotificationData}, gatt};
use embassy_executor::Spawner;

use defmt::{println, info, debug};

use embassy_time::Timer;
use esp_hal::{
    clock::ClockControl, efuse::Efuse, peripherals::Peripherals, prelude::*, rng::Rng, system::SystemControl, timer::{systimer::SystemTimer, timg::TimerGroup}
};

use fixedstr::{str64, str_format};
use rmtled::{ble::{ble_adv, uuid_from_str}, create_ble_connector, init_embassy};
use rmtled::alloc::init_heap;

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

    info!("tracing");

    let ph = Peripherals::take();
    let clocks = ClockControl::max(SystemControl::new(ph.SYSTEM).clock_control).freeze();

    let rng = Rng::new(ph.RNG);

    let alarm0 = SystemTimer::new(ph.SYSTIMER).alarm0.into();
    init_embassy(alarm0, &clocks);

    let timer0 = TimerGroup::new(ph.TIMG0, &clocks, None)
        .timer0
        .into();

    let ble_conn = create_ble_connector(
        timer0,
        &clocks,
        rng,
        ph.RADIO_CLK,
        ph.BT,
    );

    let mac = Efuse::get_mac_address();
    let dev_name = str_format!(str64, "vf-led-{}", hex::encode(&mac[3..6]));
    info!("{}", dev_name.as_str());

    let service_uuid = uuid_from_str(CONFIG.service_uuid);
    let _value_uuid = uuid_from_str(CONFIG.value_uuid);

    let mut ble = bleps::Ble::new(ble_conn, esp_wifi::current_millis);

    loop {
        info!("start advertising");
        ble_adv(&mut ble, service_uuid, dev_name.as_str()).await;

        let mut rf = |_offset: usize, data: &mut [u8]| {
            data[..20].copy_from_slice(&b"Hello Bare-Metal BLE"[..]);
            17
        };
        let mut wf = |offset: usize, data: &[u8]| {
            println!("RECEIVED: {} {:?}", offset, data);
        };

        let mut wf2 = |offset: usize, data: &[u8]| {
            println!("RECEIVED: {} {:?}", offset, data);
        };

        let mut rf3 = |_offset: usize, data: &mut [u8]| {
            data[..5].copy_from_slice(&b"Hola!"[..]);
            5
        };
        let mut wf3 = |offset: usize, data: &[u8]| {
            println!("RECEIVED: Offset {}, data {:?}", offset, data);
        };

        gatt!([service {
            uuid: "937312e0-2354-11eb-9f10-fbc30a62cf38",
            characteristics: [
                characteristic {
                    uuid: "937312e0-2354-11eb-9f10-fbc30a62cf38",
                    read: rf,
                    write: wf,
                },
                characteristic {
                    uuid: "957312e0-2354-11eb-9f10-fbc30a62cf38",
                    write: wf2,
                },
                characteristic {
                    name: "my_characteristic",
                    uuid: "987312e0-2354-11eb-9f10-fbc30a62cf38",
                    notify: true,
                    read: rf3,
                    write: wf3,
                },
            ],
        },]);

        let mut srv = AttributeServer::new(&mut ble, &mut gatt_attributes);

        let counter = RefCell::new(0u8);
        let counter = &counter;

        let mut notifier = || {
            // TODO how to check if notifications are enabled for the characteristic?
            // maybe pass something into the closure which just can query the characteristic
            // value probably passing in the attribute server won't work?

            async {
                Timer::after_secs(1).await;
                let mut data = [0u8; 13];
                data.copy_from_slice(b"Notification0");
                {
                    let mut counter = counter.borrow_mut();
                    data[data.len() - 1] += *counter;
                    *counter = (*counter + 1) % 10;
                }
                NotificationData::new(my_characteristic_handle, &data)
            }
        };

        srv.run(&mut notifier).await.unwrap();
    }
}
