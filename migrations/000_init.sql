CREATE TABLE IF NOT EXISTS users(
    user_id TEXT NOT NULL PRIMARY KEY,
    session_token uuid NOT NULL DEFAULT gen_random_uuid(),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);