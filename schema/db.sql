CREATE TABLE IF NOT EXISTS todo (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    completed BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT (NOW() at time zone 'utc'),
    updated_at TIMESTAMP DEFAULT (NOW() at time zone 'utc'),
)
