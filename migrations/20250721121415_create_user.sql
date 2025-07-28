-- Add migration script here
CREATE TABLE IF NOT EXISTS users(
    id SERIAL PRIMARY KEY,
    email VARCHAR(254) UNIQUE NOT NULL,
    username VARCHAR(254) UNIQUE NOT NULL,
    full_name TEXT NOT NULL,
    password VARCHAR(254) UNIQUE NOT NULL,
    first_login BOOLEAN NOT NULL DEFAULT 'yes',
    user_group VARCHAR(10) NOT NULL DEFAULT 'Agent'
)