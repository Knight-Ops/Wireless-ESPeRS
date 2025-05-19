#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]
#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::rng::Rng;
use esp_hal::timer::systimer::SystemTimer;
use esp_wifi::wifi::WifiMode;
use panic_rtt_target as _;
use xiao_firmware::{STATUS_WATCHER, WifiStatus, networking_task};

extern crate alloc;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 96 * 1024);

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    info!("Embassy initialized!");

    let rng = Rng::new(peripherals.RNG);
    let led = Output::new(peripherals.GPIO15, Level::Low, OutputConfig::default());

    spawner
        .spawn(networking_task(
            peripherals.TIMG0,
            peripherals.RADIO_CLK,
            peripherals.WIFI,
            rng.clone(),
        ))
        .unwrap();

    spawner.spawn(led_blinker(led)).unwrap();

    let sender = STATUS_WATCHER.sender();
    Timer::after(Duration::from_secs(5)).await;
    sender.send(WifiStatus::Enabled(WifiMode::Ap));
    loop {
        Timer::after(Duration::from_secs(60)).await;
        sender.send(WifiStatus::Disabled);
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-beta.0/examples/src/bin
}

#[embassy_executor::task]
async fn led_blinker(mut led: Output<'static>) {
    loop {
        led.toggle();
        Timer::after(Duration::from_millis(2000)).await;
    }
}
