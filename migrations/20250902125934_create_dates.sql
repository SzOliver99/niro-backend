-- Add migration script here
CREATE TABLE IF NOT EXISTS user_dates(
    id SERIAL PRIMARY KEY,
    uuid UUID UNIQUE DEFAULT uuid_generate_v4(),
    meet_date TIMESTAMP NOT NULL,
    full_name VARCHAR(254) NOT NULL,

    phone_number_enc BYTEA NOT NULL,
    phone_number_nonce BYTEA NOT NULL,
    phone_number_hash BYTEA UNIQUE,

    meet_location VARCHAR(254) NOT NULL,
    meet_type VARCHAR(254) NOT NULL,

    is_completed BOOLEAN NOT NULL DEFAULT FALSE,

    created_by VARCHAR(254) NOT NULL,
    created_at TIMESTAMPTZ(0) NOT NULL DEFAULT NOW(),

    user_id INT REFERENCES users(id) ON DELETE SET NULL
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_user_dates_user_id ON user_dates(user_id);
CREATE INDEX IF NOT EXISTS idx_user_dates_meet_date ON user_dates(meet_date);
CREATE INDEX IF NOT EXISTS idx_user_dates_is_completed ON user_dates(is_completed);
CREATE INDEX IF NOT EXISTS idx_user_dates_created_by ON user_dates(created_by);
CREATE INDEX IF NOT EXISTS idx_user_dates_created_at ON user_dates(created_at);

-- Composite index (if you often filter by both user and completion status)
CREATE INDEX IF NOT EXISTS idx_user_dates_user_completed ON user_dates(user_id, is_completed);
