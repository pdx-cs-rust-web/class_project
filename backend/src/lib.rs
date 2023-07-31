use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

use dotenvy::dotenv;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::db::new_pool;

pub mod answer;
pub mod db;
pub mod error;
pub mod handlers;
pub mod layers;
pub mod question;
pub mod routes;
mod template;
mod user;

pub async fn run_backend() {
    dotenv().ok();
    init_logging();

    let addr = get_host_from_env();

    let app = routes::app(new_pool().await).await;

    info!("Listening...");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn get_host_from_env() -> SocketAddr {
    let host = std::env::var("API_HOST").unwrap();
    let api_host = IpAddr::from_str(&host).unwrap();
    let api_port: u16 = std::env::var("API_PORT")
        .expect("Could not find API_PORT in .env file")
        .parse()
        .expect("Can't create a u16 from the given API_PORT string");

    SocketAddr::from((api_host, api_port))
}

fn init_logging() {
    // https://github.com/tokio-rs/axum/blob/main/examples/tracing-aka-logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                "backend=trace,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}


/*AUTH
1) Ability to create a new user with a secure password
2) Ability to login as that user via the password
3) Authenticate already-logged-in-users

Registration Flow
1. User attempts to register by sending a new email address, a password, and a confirm_password
2. We want to check to see if their password == confirm_password
3.  We check to see if there's a user in the database with that email already
4. Create a new user account by adding a new row to the user table
 */
