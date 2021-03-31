use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;

use crate::routes;

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    // Wrap the pool using web::Data, which is just an Arc smart pointer.
    let db_pool = web::Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(routes::health_check))
            .route("/subscriptions", web::post().to(routes::subscribe))
            // Use app_data instead of data to prevent wrapping in an additional Arc pointer.
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
