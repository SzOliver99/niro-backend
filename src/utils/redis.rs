extern crate redis;
use redis::Commands;

pub struct Redis;

impl Redis {
    pub fn set_token_to_user(
        con: &mut redis::Connection,
        user_id: u32,
        token: &str,
        exp_time: i64,
    ) -> redis::RedisResult<()> {
        con.set::<_, _, String>(token, format!("user:{user_id}"))?;
        con.expire::<_, ()>(token, exp_time)?;

        Ok(())
    }

    pub fn get_user_id_by_token(
        con: &mut redis::Connection,
        token: &str,
    ) -> redis::RedisResult<i32> {
        let is_exists = con.exists::<_, bool>(&token)?;
        println!("{token}");
        if is_exists {
            let redis_value = con.get::<_, String>(&token)?;
            let user_id = redis_value[5..].parse::<i32>().unwrap();
            println!("{user_id}");
            return Ok(user_id);
        }
        Ok(-1) // Not exists
    }
}

use rand::Rng;
pub struct Token;
impl Token {
    pub fn generate_token() -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
        const TOKEN_LEN: usize = 32;

        let mut rng = rand::rng();

        let token: String = (0..TOKEN_LEN)
            .map(|_| {
                let idx = rng.random_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        token
    }

    pub fn generate_six_digit_number() -> String {
        const TOKEN_LEN: usize = 6;

        let mut rng = rand::rng();
        let token: String = (0..TOKEN_LEN)
            .map(|_| {
                let idx: u8 = rng.random_range(0..10);
                idx.to_string()
            })
            .collect();

        token
    }
}
