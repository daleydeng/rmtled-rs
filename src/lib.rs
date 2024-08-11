#![no_std]

mod macros;

pub mod sysinit;
pub use sysinit::*;

pub mod ble;
pub mod alloc;

pub(crate) use defmt::info;

#[toml_cfg::toml_config]
pub struct Config {
    #[default(32)]
    heap_size: usize,
}