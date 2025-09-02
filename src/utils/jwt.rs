use actix_web::web;
use jsonwebtoken::errors::Error as JwtError;
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode,
};
use serde::{Deserialize, Serialize};

use crate::extractors::authentication_token::Claims;

pub async fn generate_jwt_token(id: usize, secret: String) -> String {
    let exp: usize = (chrono::Utc::now() + chrono::Duration::hours(3)).timestamp() as usize;
    let claims: Claims = Claims { id, exp };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_str().as_ref()),
    )
    .unwrap();
    token
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DecodeBody {
    pub token: String,
}

pub async fn _decode_id_from_token(
    body: web::Json<DecodeBody>,
    secret: String,
) -> Result<usize, JwtError> {
    let token_result: Result<TokenData<Claims>, JwtError> = decode::<Claims>(
        &body.token,
        &DecodingKey::from_secret(secret.as_str().as_ref()),
        &Validation::new(Algorithm::HS256),
    );

    match token_result {
        Ok(token) => Ok(token.claims.id),
        Err(e) => Err(e),
    }
}
