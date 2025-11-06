CREATE TABLE IF NOT EXISTS recruitment (
    UUID UUID UNIQUE DEFAULT uuid_generate_v4(),
    full_name VARCHAR(254) NOT NULL,
    email_enc BYTEA NOT NULL,
    email_nonce BYTEA NOT NULL,
    email_hash BYTEA NOT NULL,
    phone_number_enc BYTEA NOT NULL,
    phone_number_nonce BYTEA NOT NULL,
    phone_number_hash BYTEA NOT NULL,
    "description" TEXT NOT NULL,
    created_by VARCHAR(254) NOT NULL
);