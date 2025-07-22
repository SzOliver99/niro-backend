-- Add migration script here
CREATE TABLE IF NOT EXISTS customers(
    id SERIAL PRIMARY KEY,
    email TEXT NOT NULL,
    phonenumber TEXT NOT NULL,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS customer_history(
    id SERIAL PRIMARY KEY,
    ptype TEXT NOT NULL,
    "time" TIMESTAMP NOT NULL,
    customer_id INTEGER NOT NULL REFERENCES customers(id) ON DELETE CASCADE
);