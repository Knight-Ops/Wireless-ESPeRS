use embassy_executor::Spawner;
use embassy_net::Stack;
use embassy_time::Duration;
use picoserve::{AppBuilder, AppRouter, response::File, routing::get_service};
use static_cell::{StaticCell, make_static};

/// Static Cell for AppProps'
pub static APP_ROUTER: StaticCell<picoserve::Router<<AppProps as AppBuilder>::PathRouter>> =
    StaticCell::new();

/// Static Cell for Picoserve Config
pub static CONFIG: StaticCell<picoserve::Config<Duration>> = StaticCell::new();

pub struct AppProps;

impl AppBuilder for AppProps {
    type PathRouter = impl picoserve::routing::PathRouter;

    fn build_app(self) -> picoserve::Router<Self::PathRouter> {
        picoserve::Router::new()
            .route(
                "/",
                get_service(File::html(include_str!("../../xiao-web/dist/index.html"))),
            )
            .route(
                "/wifi",
                get_service(File::html(include_str!("../../xiao-web/dist/wifi.html"))),
            )
            .route(
                "/thread",
                get_service(File::html(include_str!("../../xiao-web/dist/thread.html"))),
            )
            .route(
                "/_app/immutable/assets/bundle.e5VEuPPr.css",
                get_service(File::css(include_str!(
                    "../../xiao-web/dist/_app/immutable/assets/bundle.e5VEuPPr.css"
                ))),
            )
            .route(
                "/_app/immutable/bundle.Ds5NMssA.js",
                get_service(File::javascript(include_str!(
                    "../../xiao-web/dist/_app/immutable/bundle.Ds5NMssA.js"
                ))),
            )
    }
}

const WEB_TASK_POOL_SIZE: usize = 4;

pub async fn start_web_server(stack: Stack<'static>) {
    let app = APP_ROUTER.init(AppProps.build_app());

    let config = CONFIG.init(
        picoserve::Config::new(picoserve::Timeouts {
            start_read_request: Some(Duration::from_secs(5)),
            persistent_start_read_request: Some(Duration::from_secs(5)),
            read_request: Some(Duration::from_secs(2)),
            write: Some(Duration::from_secs(2)),
        })
        .keep_connection_alive(),
    );

    let web_task_futures: [_; WEB_TASK_POOL_SIZE] =
        core::array::from_fn(|id| web_task(id, stack, app, config));
    embassy_futures::join::join_array(web_task_futures).await;
}

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
