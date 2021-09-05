use std::net::TcpListener;

use sqlx::PgPool;

use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let settings = get_configuration().expect("Failed to read configuration");
    let connection = PgPool::connect(&settings.database.connection_string())
        .await
        .expect("Should connect");
    let listener = TcpListener::bind(format!(
        "{}:{}",
        settings.application.host, settings.application.port
    ))?;

    run(listener, connection)?.await
}
