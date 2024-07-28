mod api;
mod models;
mod schema;
mod types;
mod utils;

use actix_governor::{Governor, GovernorConfigBuilder};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use types::AppData;

use actix_web::{
    web::{scope, Data},
    App, HttpServer,
};

use api::{transaction, user};

pub fn get_connection_pool() -> Pool<ConnectionManager<PgConnection>> {
    dotenv::dotenv().ok();
    let db_url: String = std::env::var("DATABASE_URL")
        .map(|val| val.parse().unwrap())
        .expect("DATABASE_URL is not set");

    let manager = ConnectionManager::<PgConnection>::new(db_url);

    Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let app_data = AppData {
        db_pool: get_connection_pool(),
    };


    let governor_conf = GovernorConfigBuilder::default()
        .per_second(60)
        .burst_size(5)
        .finish()
        .unwrap();


    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(app_data.clone()))
            .wrap(Governor::new(&governor_conf))
            .service(scope("/user").service(user::endpoints()))
            .service(scope("/transaction").service(transaction::endpoints()))
    })
    .bind(("0.0.0.0", 8070))?
    .workers(3)
    .run()
    .await
}
