use std::net::TcpListener;
use zero_prod::configuration::get_configuration;
use zero_prod::startup::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Panic if we can't read the configuration.
    let configuration = get_configuration().expect("Failed to read the configurations");
    // The port not comes from our settings.
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    // Bubble up the std::io::Error otherwise await the result.
    run(listener)?.await
}
