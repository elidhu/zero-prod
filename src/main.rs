use sqlx::PgPool;

use std::net::TcpListener;
use zero_prod::configuration::get_configuration;
use zero_prod::startup::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Panic if we can't read the configuration.
    let configuration = get_configuration().expect("Failed to read the configurations");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    // The port num comes from our settings.
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    // Bubble up the std::io::Error otherwise await the result.
    run(listener, connection_pool)?.await
}
