use std::env;

use actix_cors::Cors;
use actix_web::{App, HttpServer, http, middleware::Logger, web};
use base64::{Engine as _, engine::general_purpose};
use chacha20poly1305::Key;
use env_logger::Env;

use crate::{database::Database, scopes, web_data::WebData};

pub struct Server;
impl Server {
    pub async fn run(port: u16) -> std::io::Result<()> {
        // Initialize logger if -log flag is passed
        if env::args().any(|arg| arg == "-log") {
            env_logger::init_from_env(Env::default().default_filter_or("info"));
        }

        let key_b64 = env::var("ENCRYPTION_KEY").expect("ENCRYPTION_KEY must be set!");
        let key_bytes = general_purpose::STANDARD.decode(key_b64).unwrap();

        // Initialize shared DB state once at startup
        let db = Database::create_connection()
            .await
            .expect("Failed to initialize database");
        let key = Key::from_slice(&key_bytes);
        let hmac_secret = env::var("HMAC_SECRET")
            .expect("HMAC_SECRET must be set!")
            .into_bytes();
        let db_data = web::Data::new(WebData {
            db,
            key: *key,
            hmac_secret,
        });

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
                .app_data(db_data.clone())
                .service(scopes::user::user_scope())
                .service(scopes::customer::customer_scope())
                .service(scopes::lead::lead_scope())
                .service(scopes::user_date::dates_scope())
                .service(scopes::contract::contract_scope())
                .service(scopes::intervention_task::intervention_task_scope())
                .service(scopes::recommendation::recommendation_scope())
        })
        .bind(("0.0.0.0", port))?
        .run()
        .await
    }
}
