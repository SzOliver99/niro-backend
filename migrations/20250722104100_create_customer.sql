-- Add migration script here
CREATE TABLE IF NOT EXISTS customers(
    id SERIAL PRIMARY KEY,
    email VARCHAR(254) UNIQUE NOT NULL,
    phone_number VARCHAR(20) UNIQUE NOT NULL,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS customer_history(
    id SERIAL PRIMARY KEY,
    p_type TEXT NOT NULL,
    "time" TIMESTAMP NOT NULL,
    customer_id INTEGER NOT NULL REFERENCES customers(id) ON DELETE CASCADE
);