CREATE TABLE IF NOT EXISTS users(
    user_id BIGINT NOT NULL PRIMARY KEY,
    session_token uuid NOT NULL DEFAULT gen_random_uuid(),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS rooms(
    user_id BIGINT NOT NULL PRIMARY KEY,
    is_open boolean NOT NULL
);

CREATE TABLE IF NOT EXISTS requests(
    request_id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    request_body BYTEA NOT NULL,
    response BYTEA DEFAULT NULL
);

CREATE OR REPLACE FUNCTION notify_response() RETURNS trigger AS $notify_response$
BEGIN
PERFORM PG_NOTIFY('guest', NEW.request_id::text);
RETURN NEW;
END;
$notify_response$
LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER notify_response_trigger
    AFTER UPDATE OF response ON requests
    FOR EACH ROW
EXECUTE FUNCTION notify_response();