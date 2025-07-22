use actix_web::{
    Error as ActixWebError, FromRequest, HttpRequest, dev::Payload, error::ErrorUnauthorized,
    http::header::HeaderValue, web,
};
use jsonwebtoken::{
    Algorithm, DecodingKey, TokenData, Validation, decode, errors::Error as JwtError,
};
use serde::{Deserialize, Serialize};
use std::{
    env,
    future::{Ready, ready},
};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub id: usize,
    pub exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticationToken {
    pub id: usize,
}

impl FromRequest for AuthenticationToken {
    type Error = ActixWebError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let req = req.clone();

        let authorization_header_option: Option<&HeaderValue> =
            req.headers().get(actix_web::http::header::AUTHORIZATION);

        // No Header was sent
        if authorization_header_option.is_none() {
            return ready(Err(ErrorUnauthorized("No authentication token sent!")));
        }

        let authentication_token: String = authorization_header_option
            .unwrap()
            .to_str()
            .unwrap_or("")
            .to_string();

        // Couldn't convert Header::Authorization to String
        if authentication_token.is_empty() {
            return ready(Err(ErrorUnauthorized(
                "Authentication token has foreign chars!",
            )));
        }

        let secret = &env::var("AUTH_SECRET").expect("DATABASE_URL must be set!");

        let token_result: Result<TokenData<Claims>, JwtError> = decode::<Claims>(
            &authentication_token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::new(Algorithm::HS256),
        );

        match token_result {
            Ok(token) => ready(Ok(AuthenticationToken {
                id: token.claims.id,
            })),
            Err(_e) => ready(Err(ErrorUnauthorized("Invalid authentication token sent!"))),
        }
    }
}
