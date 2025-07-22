use dotenvy::dotenv;

use crate::server::Server;

mod database;
mod extractors;
mod models;
mod scopes;
mod server;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    Server::run(8080).await
}
