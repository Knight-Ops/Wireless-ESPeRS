use core::net::Ipv4Addr;
use core::net::SocketAddr;
use core::net::SocketAddrV4;
use core::str::FromStr;

use defmt::{info, warn};
use edge_dhcp::DhcpOption;
use edge_dhcp::MessageType;
use edge_dhcp::Options;
use edge_dhcp::Packet;
use edge_dhcp::io;
use edge_dhcp::io::DEFAULT_SERVER_PORT;
use edge_dhcp::server::Server;
use edge_dhcp::server::ServerOptions;
use edge_nal::UdpReceive;
use edge_nal::UdpSend;
use edge_nal_embassy::Udp;
use edge_nal_embassy::UdpBuffers;
use embassy_executor::Spawner;
use embassy_net::{Ipv4Cidr, Runner, Stack, StackResources, StaticConfigV4};
use embassy_time::{Duration, Timer};
use esp_hal::rng::Rng;
use esp_hal::timer::timg::TimerGroup;
use esp_wifi::wifi::{
    AccessPointConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiState,
};
use panic_rtt_target as _;
use static_cell::make_static;

use crate::web_server::start_web_server;

pub async fn ap_configure(
    spawner: &Spawner,
    timg0: esp_hal::peripherals::TIMG0,
    radio_clk: esp_hal::peripherals::RADIO_CLK,
    wifi: esp_hal::peripherals::WIFI,
    mut rng: Rng,
) {
    let timg0 = TimerGroup::new(timg0);

    let esp_wifi_ctrl = make_static!(esp_wifi::init(timg0.timer0, rng, radio_clk).unwrap());

    let (controller, interfaces) = esp_wifi::wifi::new(esp_wifi_ctrl, wifi).unwrap();

    let device = interfaces.ap;

    let gw_ip_addr_str = "192.168.35.1";
    let gw_ip_addr = Ipv4Addr::from_str(gw_ip_addr_str).expect("failed to parse gateway ip");
    let config = embassy_net::Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(gw_ip_addr, 24),
        gateway: Some(gw_ip_addr),
        dns_servers: Default::default(),
    });
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;
    let (stack, runner) = embassy_net::new(
        device,
        config,
        make_static!(StackResources::<8>::new()),
        seed,
    );

    spawner.spawn(connection(controller)).unwrap();
    spawner.spawn(net_task(runner)).unwrap();
    spawner.spawn(run_dhcp(stack, gw_ip_addr_str)).unwrap();
    start_web_server(spawner, stack).await;
}

#[embassy_executor::task]
async fn run_dhcp(stack: Stack<'static>, gw_ip_addr: &'static str) {
    use edge_nal::UdpBind;

    let ip = Ipv4Addr::from_str(gw_ip_addr).expect("dhcp task failed to parse gw ip");

    let mut buf = [0u8; 1500];

    let mut gw_buf = [Ipv4Addr::UNSPECIFIED];
    info!("DHCP server starting...");

    let buffers = UdpBuffers::<2, 1024, 1024, 10>::new();
    let unbound_socket = Udp::new(stack, &buffers);
    let mut bound_socket = unbound_socket
        .bind(core::net::SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::UNSPECIFIED,
            DEFAULT_SERVER_PORT,
        )))
        .await
        .unwrap();

    info!("DHCP server started");

    loop {
        _ = io::server::run(
            &mut Server::<_, 2>::new_with_et(ip),
            &ServerOptions::new(ip, Some(&mut gw_buf)),
            &mut bound_socket,
            &mut buf,
        )
        .await
        .inspect_err(|e| warn!("DHCP server error"));

        Timer::after(Duration::from_millis(500)).await;
    }
}

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    info!("start connection task");
    loop {
        match esp_wifi::wifi::wifi_state() {
            WifiState::ApStarted => {
                // wait until we're no longer connected
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
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}
