CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) NOT NULL
);

INSERT INTO users (username) VALUES
('alice'),
('bob');
