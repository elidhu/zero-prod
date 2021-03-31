use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use zero_prod::configuration::{get_configuration, DatabaseSettings};

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

// We need to spawn the app like this because if we don't run it as a background task we will
// launch the app and be awaiting it forever!

async fn spawn_app() -> TestApp {
    // Port zero is an OS level sepcial case which will scan for an available port.
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    // Randomise the database name for test isolation.
    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let connection_pool = configure_database(&configuration.database).await;

    let server =
        zero_prod::startup::run(listener, connection_pool.clone()).expect("Failed to bind address");
    // Launch the server as a background task
    // tokio::spawn returns a handle to the spawned future,
    // but we have no use for it here, hence the non-binding let
    //
    // New dev dependenty - lets add tokio to the party with
    // `cargo add tokio --dev --vers 1`
    let _ = tokio::spawn(server);
    // Return the TestApp
    TestApp {
        address,
        db_pool: connection_pool,
    }
}

/// configure_database Create's a new database amd runs the migrations
pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres.");
    connection
        .execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name))
        .await
        .expect("Failed to create databse.");

    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate databse.");

    connection_pool
}

// `actix_rt::test` is the testing equivalent of `actix_web::main`.
// It also spares you from having to specify the `#[test]` attribute.
//
// Use `cargo add actix-rt --dev --vers 2` to add `actix-rt`
// under `[dev-dependencies]` in Cargo.toml
//
// You can inspect what code gets generated using
// `cargo expand --test health_check` (<- name of the test file)
#[actix_rt::test]
async fn health_check_works() {
    // Arrange
    let test_app = spawn_app().await;
    // We need to bring in `reqwest`
    // to perform HTTP requests against our application.
    //
    // Use `cargo add reqwest --dev --vers 0.11` to add // it under `[dev-dependencies]` in
    // Cargo.toml
    let client = reqwest::Client::new();
    // Act
    let response = client
        .get(format!("{}/health_check", &test_app.address))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[actix_rt::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // Act
    let response = client
        .post(&format!("{}/subscriptions", &test_app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute requets.");

    // Assert
    assert_eq!(200, response.status().as_u16());

    // Query the database and check som values.
    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscriptions.");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[actix_rt::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &test_app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute requets.");
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when payload was {}.",
            error_message
        );
    }
}
