-- Add migration script here
CREATE TABLE IF NOT EXISTS user_dates(
    id SERIAL PRIMARY KEY,
    meet_date TIMESTAMP NOT NULL,
    full_name VARCHAR(254) NOT NULL,

    phone_number_enc BYTEA NOT NULL,
    phone_number_nonce BYTEA NOT NULL,
    phone_number_hash BYTEA UNIQUE,

    meet_location VARCHAR(254) NOT NULL,
    meet_type VARCHAR(254) NOT NULL,

    is_completed BOOLEAN NOT NULL DEFAULT FALSE,

    created_by VARCHAR(254) NOT NULL,
    created_at TIMESTAMP(0) NOT NULL DEFAULT NOW(),

    user_id INT REFERENCES users(id) ON DELETE SET NULL
);

-- Indexes for performance and search
CREATE INDEX IF NOT EXISTS idx_user_dates_user_id ON user_dates (user_id);
CREATE INDEX IF NOT EXISTS idx_user_dates_meet_date ON user_dates (meet_date);
CREATE INDEX IF NOT EXISTS idx_user_dates_phone_number_hash ON user_dates (phone_number_hash);
CREATE INDEX IF NOT EXISTS idx_user_dates_is_completed ON user_dates (is_completed);