CREATE TABLE IF NOT EXISTS customer_recommendations (
    UUID UUID UNIQUE DEFAULT uuid_generate_v4(),
    full_name VARCHAR(254) NOT NULL,
    phone_number_enc BYTEA NOT NULL,
    phone_number_nonce BYTEA NOT NULL,
    phone_number_hash BYTEA UNIQUE,
    city_enc BYTEA NOT NULL,
    city_nonce BYTEA NOT NULL,
    referral_name VARCHAR(254) NOT NULL,
    user_id INT REFERENCES users (id) ON DELETE SET NULL,
    created_by VARCHAR(254) NOT NULL
);