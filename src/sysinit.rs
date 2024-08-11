use crate::mk_static;
use embedded_io::ErrorType;
use esp_hal::{
    clock::Clocks,
    peripherals::{BT, RADIO_CLK},
    rng::Rng,
    timer::{ErasedTimer, OneShotTimer, PeriodicTimer},
};
use esp_wifi::{ble::controller::{asynch::BleConnector, BleConnectorError}, EspWifiInitFor};
use embedded_io_async::{Read, Write};
use defmt::*;

pub fn init_embassy(alarm0: ErasedTimer, clocks: &Clocks) {
    let timers = [OneShotTimer::new(alarm0)];
    let timers = mk_static!([OneShotTimer<ErasedTimer>; 1], timers);
    esp_hal_embassy::init(clocks, timers);
}

pub struct BleConn<T>(T);

impl<T> BleConn<T> {
    pub fn new(v: T) -> Self {
        Self(v)
    }
}

impl<T> ErrorType for BleConn<T> {
    type Error = BleConnectorError;
}

impl Read for BleConn<BleConnector<'_>> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        debug!("Reading...");
        let res = self.0.read(buf).await;
        debug!("R({=[u8]:#04x})", buf);
        res
    }
}

impl Write for BleConn<BleConnector<'_>> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        debug!("W({=[u8]:#04x})", buf);
        let res = self.0.write(buf).await;
        debug!("Writed.");
        res
    }
}

pub struct BleConnReader<T>(T);

impl<T> BleConnReader<T> {
    pub fn new(v: T) -> Self {
        Self(v)
    }
}

impl<T> ErrorType for BleConnReader<T> {
    type Error = BleConnectorError;
}

impl Read for BleConnReader<BleConnector<'_>> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        debug!("Reading...");
        let res = self.0.read(buf).await;
        debug!("R({=[u8]:#04x})", buf);
        res
    }
}

pub struct BleConnWriter<T>(T);

impl<T> BleConnWriter<T> {
    pub fn new(v: T) -> Self {
        Self(v)
    }
}

impl<T> ErrorType for BleConnWriter<T> {
    type Error = BleConnectorError;
}

impl Write for BleConnWriter<BleConnector<'_>> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        debug!("W({=[u8]:#04x})", buf);
        let res = self.0.write(buf).await;
        debug!("Writed.");
        res
    }
}

pub fn create_ble_connector<'a>(
    timer: ErasedTimer,
    clocks: &Clocks,
    rng: Rng,
    radio_clocks: RADIO_CLK,
    bt: BT,
) -> BleConn<BleConnector<'a>> {
    let timer = PeriodicTimer::new(timer);

    let wifi_init = esp_wifi::initialize(
        EspWifiInitFor::Ble,
        timer,
        rng,
        radio_clocks,
        clocks,
    )
    .unwrap();

    BleConn::new(BleConnector::new(&wifi_init, bt))
}

pub fn create_ble_connector_rw<'a>(
    timer: ErasedTimer,
    clocks: &Clocks,
    rng: Rng,
    radio_clocks: RADIO_CLK,
    _: BT,
) -> (BleConnReader<BleConnector<'a>>, BleConnWriter<BleConnector<'a>>) {
    let timer = PeriodicTimer::new(timer);

    let wifi_init = esp_wifi::initialize(
        EspWifiInitFor::Ble,
        timer,
        rng,
        radio_clocks,
        clocks,
    )
    .unwrap();

    let bt_r = unsafe {BT::steal()};
    let bt_w = unsafe {BT::steal()};
    (BleConnReader::new(BleConnector::new(&wifi_init, bt_r)),
    BleConnWriter::new(BleConnector::new(&wifi_init, bt_w)))
}