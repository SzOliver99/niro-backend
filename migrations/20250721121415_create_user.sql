-- Add migration script here

-- DROP TYPE IF EXISTS user_group;
-- CREATE TYPE user_group as ENUM('User', 'Admin');

CREATE TABLE IF NOT EXISTS users(
    id SERIAL PRIMARY KEY,
    email TEXT NOT NULL,
    username TEXT NOT NULL,
    password TEXT NOT NULL,
    "group" user_group NOT NULL DEFAULT 'User'
)