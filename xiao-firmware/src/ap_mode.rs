#![no_std]

use core::net::Ipv4Addr;
use core::net::SocketAddr;
use core::net::SocketAddrV4;
use core::str::FromStr;

use defmt::{info, warn};
use embassy_executor::Spawner;
use embassy_net::udp::UdpSocket;
use embassy_net::{Ipv4Cidr, Runner, Stack, StackResources, StaticConfigV4};
use embassy_sync::watch::{Receiver, Watch};
use embassy_time::{Duration, Timer};
use esp_hal::rng::Rng;
use esp_hal::timer::timg::TimerGroup;
use esp_wifi::EspWifiController;
use esp_wifi::wifi::WifiError;
use esp_wifi::wifi::WifiMode;
use esp_wifi::wifi::{
    AccessPointConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiState,
};
use panic_rtt_target as _;
use static_cell::StaticCell;

use embassy_futures::yield_now;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

/// Cancellation message for stopping the networking task.
#[derive(Clone, Copy)]
pub enum WifiStatus {
    Enabled(WifiMode),
    Disabled,
}

/// Static channel for cancellation (capacity 1, one-shot style)
pub static STATUS_WATCHER: Watch<CriticalSectionRawMutex, WifiStatus, 2> = Watch::new();

/// Static Cell for StackResources
pub static AP_STACK_RESOURCES: StaticCell<StackResources<8>> = StaticCell::new();
pub static STA_STACK_RESOURCES: StaticCell<StackResources<8>> = StaticCell::new();

/// Static Cell for ESP-WiFi Stack
pub static ESP_WIFI_STACK: StaticCell<EspWifiController<'static>> = StaticCell::new();

/// Unified networking task for both AP and Station modes, with cancellation support.
#[embassy_executor::task]
pub async fn networking_task(
    timg0: esp_hal::peripherals::TIMG0,
    radio_clk: esp_hal::peripherals::RADIO_CLK,
    wifi: esp_hal::peripherals::WIFI,
    mut rng: Rng,
) {
    let mut status_receiver = STATUS_WATCHER
        .receiver()
        .expect("Failed to get status receiver");
    let timg0 = TimerGroup::new(timg0);
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;
    let esp_wifi_ctrl_inner =
        esp_wifi::init(timg0.timer0, rng, radio_clk).expect("Failed to init ESP WiFi");

    let esp_wifi_ctrl = ESP_WIFI_STACK.init(esp_wifi_ctrl_inner);

    let (mut controller, interfaces) =
        esp_wifi::wifi::new(esp_wifi_ctrl, wifi).expect("Failed to create WiFi interfaces");

    let mut ap_device = Some(interfaces.ap);
    let mut sta_device = Some(interfaces.sta);
    loop {
        match status_receiver.get().await {
            WifiStatus::Enabled(mode) => {
                info!("WiFi status changed: {:?}", mode);

                // Wifi Controller will be started at the beginning of AP mode
                if mode == WifiMode::Sta {
                    controller
                        .start_async()
                        .await
                        .expect("Failed to start controller")
                }

                // Set up the mode of the controller, assigning DHCP or a static IP
                match mode {
                    WifiMode::Ap => {
                        let gw_ip_addr_str = "192.168.35.1";
                        let gw_ip_addr =
                            Ipv4Addr::from_str(gw_ip_addr_str).expect("Failed to parse gateway ip");
                        let config = embassy_net::Config::ipv4_static(StaticConfigV4 {
                            address: Ipv4Cidr::new(gw_ip_addr, 24),
                            gateway: Some(gw_ip_addr),
                            dns_servers: Default::default(),
                        });

                        let resources =
                            match AP_STACK_RESOURCES.try_init_with(|| StackResources::new()) {
                                Some(resources) => resources,
                                None => {
                                    warn!("Attempted to re-enter AP Mode");
                                    status_receiver.changed().await;
                                    continue;
                                }
                            };
                        let (stack, runner) = embassy_net::new(
                            ap_device.take().expect("Failed to take AP device"),
                            config,
                            resources,
                            seed,
                        );

                        embassy_futures::select::select(
                            async {
                                core::future::join!(
                                    connection(&mut controller, mode),
                                    net_task_async(runner),
                                    run_dhcp_async(stack, gw_ip_addr_str),
                                    start_web_server_async(stack),
                                )
                                .await;
                            },
                            async {
                                let _ = status_receiver.changed().await;
                                info!("Networking task cancelled!");
                            },
                        )
                        .await;
                    }
                    WifiMode::Sta => {
                        let config = embassy_net::Config::dhcpv4(Default::default());
                        let resources =
                            match STA_STACK_RESOURCES.try_init_with(|| StackResources::new()) {
                                Some(resources) => resources,
                                None => {
                                    warn!("Attempted to re-enter STA Mode");
                                    status_receiver.changed().await;
                                    continue;
                                }
                            };
                        let (stack, runner) = embassy_net::new(
                            sta_device.take().expect("Failed to take STA device"),
                            config,
                            resources,
                            seed,
                        );

                        embassy_futures::select::select(
                            async {
                                core::future::join!(
                                    connection(&mut controller, mode),
                                    net_task_async(runner),
                                    start_web_server_async(stack),
                                )
                                .await;
                            },
                            async {
                                let _ = status_receiver.changed().await;
                                info!("Networking task cancelled!");
                            },
                        )
                        .await;
                    }
                    WifiMode::ApSta => {
                        warn!("Unsupported WifiMode::ApSta");
                        status_receiver.changed().await;
                    }
                }

                // Disable WiFi when the status changed
                controller
                    .stop_async()
                    .await
                    .expect("Failed to stop controller");
            }
            WifiStatus::Disabled => {
                info!("WiFi status changed: Disabled");
                status_receiver.changed().await;
            }
        }
    }
}

/// Contains AI-generated content.
/// Modified connection task to accept mode.
async fn connection(controller: &mut WifiController<'static>, mode: WifiMode) {
    info!("start connection task");
    loop {
        match mode {
            WifiMode::Ap => {
                match esp_wifi::wifi::wifi_state() {
                    WifiState::ApStarted => {
                        controller.wait_for_event(WifiEvent::ApStop).await;
                        Timer::after(Duration::from_millis(5000)).await
                    }
                    _ => {}
                }
                if !matches!(controller.is_started(), Ok(true)) {
                    let client_config = Configuration::AccessPoint(AccessPointConfiguration {
                        ssid: "esp-wifi".try_into().unwrap(),
                        ..Default::default()
                    });
                    controller.set_configuration(&client_config).unwrap();
                    info!("Starting wifi");
                    controller.start_async().await.unwrap();
                    info!("Wifi started!");
                }
            }
            WifiMode::Sta => {
                // TODO: Add station mode connection logic here
                yield_now().await;
            }
            _ => {
                warn!("Unsupported WifiMode");
                return;
            }
        }
    }
}

/// Contains AI-generated content.
/// Async version of net_task.
async fn net_task_async(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await;
}

/// Contains AI-generated content.
/// Async version of run_dhcp, refactored to use embassy-net UDP sockets.
async fn run_dhcp_async(stack: Stack<'static>, gw_ip_addr: &'static str) {
    use core::str::FromStr;
    use embassy_net::IpEndpoint;
    use embassy_net::Ipv4Address;
    use embassy_net::udp::PacketMetadata;
    use embassy_net::udp::UdpSocket;
    use embassy_time::Timer;

    const DHCP_SERVER_PORT: u16 = 67; // Standard DHCP server port

    // Parse and check gateway IP, but do not keep unused variables
    let _ = Ipv4Address::from_str(gw_ip_addr).expect("dhcp task failed to parse gw ip");
    info!("DHCP server starting...");

    loop {
        // TODO: Implement DHCP server logic using embassy-net's UDP socket API.
        Timer::after(Duration::from_millis(500)).await;
    }
}

/// Contains AI-generated content.
/// Async version of start_web_server.
async fn start_web_server_async(stack: Stack<'static>) {
    crate::web_server::start_web_server(stack).await;
}
