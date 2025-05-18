#![feature(type_alias_impl_trait)]
#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::rng::Rng;
use esp_hal::timer::systimer::SystemTimer;
use panic_rtt_target as _;
use xiao_firmware::ap_configure;

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

    let mut rng = Rng::new(peripherals.RNG);
    let led = Output::new(peripherals.GPIO15, Level::Low, OutputConfig::default());

    ap_configure(
        &spawner,
        peripherals.TIMG0,
        peripherals.RADIO_CLK,
        peripherals.WIFI,
        rng.clone(),
    )
    .await;

    spawner.spawn(led_blinker(led)).unwrap();

    loop {
        // info!("Hello world!");
        Timer::after(Duration::from_secs(1)).await;
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
