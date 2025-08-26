-- Add migration script here
CREATE TABLE IF NOT EXISTS customers(
    id SERIAL PRIMARY KEY,
    full_name VARCHAR(254) NOT NULL,
    phone_number VARCHAR(20) UNIQUE NOT NULL,
    email VARCHAR(254) UNIQUE NOT NULL,
    "address" VARCHAR(254) NOT NULL,
    user_id INT REFERENCES users(id) ON DELETE SET NULL
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