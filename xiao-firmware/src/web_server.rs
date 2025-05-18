use embassy_executor::Spawner;
use embassy_net::Stack;
use embassy_time::Duration;
use picoserve::{AppBuilder, AppRouter, response::File, routing::get_service};
use static_cell::make_static;

struct AppProps;

impl AppBuilder for AppProps {
    type PathRouter = impl picoserve::routing::PathRouter;

    fn build_app(self) -> picoserve::Router<Self::PathRouter> {
        picoserve::Router::new()
            .route(
                "/",
                get_service(File::html(include_str!("../../xiao-web/build/index.html"))),
            )
            .route(
                "/wifi",
                get_service(File::html(include_str!("../../xiao-web/build/wifi.html"))),
            )
            .route(
                "/thread",
                get_service(File::html(include_str!("../../xiao-web/build/thread.html"))),
            )
            .route(
                "/_app/immutable/assets/bundle.DzotERNF.css",
                get_service(File::css(include_str!(
                    "../../xiao-web/build/_app/immutable/assets/bundle.DzotERNF.css"
                ))),
            )
            .route(
                "/_app/immutable/bundle.OrYBsVyU.js",
                get_service(File::javascript(include_str!(
                    "../../xiao-web/build/_app/immutable/bundle.OrYBsVyU.js"
                ))),
            )
    }
}

const WEB_TASK_POOL_SIZE: usize = 2;

pub async fn start_web_server(spawner: &Spawner, stack: Stack<'static>) {
    let app = make_static!(AppProps.build_app());

    let config = make_static!(
        picoserve::Config::new(picoserve::Timeouts {
            start_read_request: Some(Duration::from_secs(5)),
            read_request: Some(Duration::from_secs(2)),
            write: Some(Duration::from_secs(2)),
        })
        .keep_connection_alive()
    );

    for id in 0..WEB_TASK_POOL_SIZE {
        spawner.must_spawn(web_task(id, stack, app, config));
    }
}

#[embassy_executor::task(pool_size = WEB_TASK_POOL_SIZE)]
async fn web_task(
    id: usize,
    stack: embassy_net::Stack<'static>,
    app: &'static AppRouter<AppProps>,
    config: &'static picoserve::Config<Duration>,
) -> ! {
    let port = 80;
    let mut tcp_rx_buffer = [0; 1024];
    let mut tcp_tx_buffer = [0; 1024];
    let mut http_buffer = [0; 2048];

    picoserve::listen_and_serve(
        id,
        app,
        config,
        stack,
        port,
        &mut tcp_rx_buffer,
        &mut tcp_tx_buffer,
        &mut http_buffer,
    )
    .await
}
