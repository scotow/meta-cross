extern crate core;

use std::{
    error::Error,
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use axum::{
    extract::{State, WebSocketUpgrade},
    http::{header, HeaderValue},
    middleware::map_response,
    response::Response,
    routing::get,
    Router, Server,
};
use clap::{ArgAction, Parser};
use lobby::Lobby;
use log::LevelFilter;

use crate::player::Player;

mod asset;
mod game;
mod lobby;
mod player;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let options = Options::parse();
    env_logger::Builder::new()
        .filter_level(options.log_level())
        .init();

    let router = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(Arc::new(Lobby::new()))
        .route("/", get(asset::handler))
        .route("/:asset", get(asset::handler))
        .layer(map_response(|mut resp: Response| async {
            resp.headers_mut().insert(
                header::SERVER,
                HeaderValue::from_static(concat!("Meta-Cross v", env!("CARGO_PKG_VERSION"))),
            );
            resp
        }));

    Server::bind(&SocketAddr::new(options.address, options.port))
        .http1_title_case_headers(true)
        .serve(router.into_make_service())
        .await?;
    Ok(())
}

async fn ws_handler(State(lobby): State<Arc<Lobby>>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |socket| async move {
        lobby.join(Player::new(socket)).await;
    })
}

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Options {
    /// Increase logs verbosity (Error (default), Warn, Info, Debug, Trace).
    #[arg(short = 'v', long = "verbose", action = ArgAction::Count)]
    pub log_level: u8,
    /// HTTP listening address.
    #[arg(short = 'a', long, default_value = "127.0.0.1")]
    pub address: IpAddr,
    /// HTTP listening port.
    #[arg(short = 'p', long, default_value = "8080")]
    pub port: u16,
}

impl Options {
    pub fn log_level(&self) -> LevelFilter {
        match self.log_level {
            0 => LevelFilter::Error,
            1 => LevelFilter::Warn,
            2 => LevelFilter::Info,
            3 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        }
    }
}
