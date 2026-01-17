-- Create users table
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    email VARCHAR NOT NULL,
    removed BOOLEAN NOT NULL DEFAULT FALSE
);

-- Create videos table
CREATE TABLE videos (
    id SERIAL PRIMARY KEY,
    title VARCHAR NOT NULL,
    description TEXT NOT NULL,
    removed BOOLEAN NOT NULL DEFAULT FALSE
);

-- Create views table
CREATE TABLE views (
    id SERIAL PRIMARY KEY,
    video_id INTEGER NOT NULL REFERENCES videos(id),
    user_id INTEGER NOT NULL REFERENCES users(id),
    watch_start TIMESTAMP NOT NULL,
    duration INTEGER NOT NULL
);
