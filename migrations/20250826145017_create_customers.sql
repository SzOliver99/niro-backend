-- Add migration script here
CREATE TABLE IF NOT EXISTS customers (
    id SERIAL PRIMARY KEY,
    full_name VARCHAR(254) NOT NULL,

    phone_number_enc BYTEA NOT NULL,
    phone_number_nonce BYTEA NOT NULL,
    phone_number_hash BYTEA UNIQUE,

    email_enc BYTEA NOT NULL,
    email_nonce BYTEA NOT NULL,
    email_hash BYTEA UNIQUE,

    address_enc BYTEA NOT NULL,
    address_nonce BYTEA NOT NULL,

    user_id INT REFERENCES users(id) ON DELETE SET NULL,
    created_by VARCHAR(254) NOT NULL
);

CREATE TABLE IF NOT EXISTS customer_leads(
    id SERIAL PRIMARY KEY,
    lead_type TEXT NOT NULL,
    inquiry_type TEXT NOT NULL,
    lead_status VARCHAR(20) NOT NULL,
    handle_at TIMESTAMP NOT NULL,
    customer_id INT NOT NULL REFERENCES customers(id) ON DELETE CASCADE
    CONSTRAINT leads_lead_status_check CHECK (lead_status IN ('Opened', 'InProgress', 'Closed'))
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_customers_user_id ON customers (user_id);
CREATE INDEX IF NOT EXISTS idx_customer_leads_customer_id ON customer_leads (customer_id);