-- Add migration script here
CREATE TABLE IF NOT EXISTS users(
    id SERIAL PRIMARY KEY,
    email VARCHAR(254) UNIQUE NOT NULL,
    username VARCHAR(254) UNIQUE NOT NULL,
    password VARCHAR(254) UNIQUE NOT NULL,
    first_login BOOLEAN NOT NULL DEFAULT 'yes',
    user_group VARCHAR(10) NOT NULL DEFAULT 'Agent'
);

CREATE TABLE IF NOT EXISTS user_info(
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users(id),
    full_name TEXT NOT NULL,
    phone_number VARCHAR(254) UNIQUE NOT NULL,
    hufa_code VARCHAR(254) UNIQUE NOT NULL,
    agent_code VARCHAR(254) UNIQUE NOT NULL
);