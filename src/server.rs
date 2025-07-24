use std::env;

use actix_cors::Cors;
use actix_web::{App, HttpServer, http, middleware::Logger};
use env_logger::Env;

use crate::scopes;

pub struct Server;
impl Server {
    pub async fn run(port: u16) -> std::io::Result<()> {
        // Initialize logger if -log flag is passed
        if env::args().any(|arg| arg == "-log") {
            env_logger::init_from_env(Env::default().default_filter_or("info"));
        }

        HttpServer::new(move || {
            let cors = Cors::default()
                // .allowed_origin("http://localhost")
                .allow_any_origin()
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                .allowed_headers(vec![
                    http::header::AUTHORIZATION,
                    http::header::ACCEPT,
                    http::header::CONTENT_TYPE,
                ])
                .max_age(3600);

            App::new()
                .wrap(cors)
                .wrap(Logger::default())
                .service(scopes::user::user_scope())
                .service(scopes::contact::contact_scope())
        })
        .bind(("0.0.0.0", port))?
        .run()
        .await
    }
}
