CREATE TABLE IF NOT EXISTS customers (
	id SERIAL PRIMARY KEY,
	UUID UUID UNIQUE DEFAULT uuid_generate_v4 (),
	full_name VARCHAR(254) NOT NULL,
	phone_number_enc BYTEA NOT NULL,
	phone_number_nonce BYTEA NOT NULL,
	phone_number_hash BYTEA UNIQUE,
	email_enc BYTEA NOT NULL,
	email_nonce BYTEA NOT NULL,
	email_hash BYTEA UNIQUE,
	address_enc BYTEA NOT NULL,
	address_nonce BYTEA NOT NULL,
	comment TEXT NOT NULL DEFAULT '',
	user_id INT REFERENCES users (id) ON DELETE SET NULL,
	created_by VARCHAR(254) NOT NULL
);

-- Indexes for customers
CREATE INDEX IF NOT EXISTS idx_customers_user_id ON customers (user_id);
CREATE INDEX IF NOT EXISTS idx_customers_created_by ON customers (created_by);

CREATE TABLE IF NOT EXISTS customer_leads (
	id SERIAL PRIMARY KEY,
	UUID UUID UNIQUE DEFAULT uuid_generate_v4 (),
	lead_type TEXT NOT NULL,
	inquiry_type TEXT NOT NULL,
	lead_status VARCHAR(20) NOT NULL,
	handle_at TIMESTAMPTZ(0) NOT NULL DEFAULT NOW(),
	customer_id INT NOT NULL REFERENCES customers (id) ON DELETE CASCADE,
	user_id INT REFERENCES users (id) ON DELETE SET NULL,
	created_by VARCHAR(254) NOT NULL,
	CONSTRAINT leads_lead_status_check CHECK (lead_status IN ('Opened', 'InProgress', 'Closed'))
);

-- Indexes for customer_leads
CREATE INDEX IF NOT EXISTS idx_customer_leads_customer_id ON customer_leads (customer_id);
CREATE INDEX IF NOT EXISTS idx_customer_leads_user_id ON customer_leads (user_id);
CREATE INDEX IF NOT EXISTS idx_customer_leads_lead_status ON customer_leads (lead_status);
CREATE INDEX IF NOT EXISTS idx_customer_leads_handle_at ON customer_leads (handle_at);

CREATE INDEX IF NOT EXISTS idx_customer_leads_status_user ON customer_leads (lead_status, user_id);