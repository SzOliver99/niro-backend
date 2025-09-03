use chacha20poly1305::Key;

use crate::{database::Database, utils::encrypt::HmacSecret};

pub struct WebData {
    pub db: Database,
    pub key: Key,
    pub hmac_secret: HmacSecret,
}
