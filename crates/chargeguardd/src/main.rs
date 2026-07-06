mod dbus;
mod policy;
mod service;

use chargeguard_core::{OBJECT_PATH, SERVICE_NAME, SysfsPowerSupply};
use service::ChargeGuardService;
use tracing::info;
use zbus::connection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "chargeguardd=info".into()),
        )
        .init();

    let service = ChargeGuardService::new(SysfsPowerSupply::default());
    let _connection = connection::Builder::system()?
        .name(SERVICE_NAME)?
        .serve_at(OBJECT_PATH, service)?
        .build()
        .await?;

    info!(
        service = SERVICE_NAME,
        object = OBJECT_PATH,
        "chargeguardd started"
    );
    tokio::signal::ctrl_c().await?;
    info!("chargeguardd stopping");
    Ok(())
}
