-- Add migration script here
CREATE TABLE IF NOT EXISTS contacts(
    id SERIAL PRIMARY KEY,
    email VARCHAR(254) UNIQUE NOT NULL,
    first_name VARCHAR(254) NOT NULL,
    last_name VARCHAR(254) NOT NULL,
    phone_number VARCHAR(20) UNIQUE NOT NULL,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS contact_history(
    id SERIAL PRIMARY KEY,
    p_type TEXT NOT NULL,
    "time" TIMESTAMP NOT NULL,
    contact_id INTEGER NOT NULL REFERENCES contacts(id) ON DELETE CASCADE
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_contacts_user_id ON contacts (user_id);
CREATE INDEX IF NOT EXISTS idx_contact_history_contact_id ON contact_history (contact_id);