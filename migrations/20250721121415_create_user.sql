-- Add migration script here
CREATE TABLE IF NOT EXISTS users(
    id SERIAL PRIMARY KEY,
    email VARCHAR(254) UNIQUE NOT NULL,
    username VARCHAR(254) UNIQUE NOT NULL,
    password VARCHAR(254) NOT NULL,
    first_login BOOLEAN NOT NULL DEFAULT TRUE,
    user_role VARCHAR(10) NOT NULL DEFAULT 'Agent',
    manager_id INT REFERENCES users(id) ON DELETE SET NULL,
    CONSTRAINT users_user_role_check CHECK (user_role IN ('Agent', 'Manager', 'Leader'))
);

CREATE TABLE IF NOT EXISTS user_info(
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users(id),
    full_name TEXT NOT NULL,
    phone_number VARCHAR(254) UNIQUE NOT NULL,
    hufa_code VARCHAR(254) UNIQUE NOT NULL,
    agent_code VARCHAR(254) UNIQUE NOT NULL
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_users_username ON users (username);
CREATE INDEX IF NOT EXISTS idx_user_info_user_id ON user_info (user_id);