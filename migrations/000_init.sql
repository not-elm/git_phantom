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
    user_id BIGINT NOT NULL,
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


CREATE OR REPLACE FUNCTION delete_requests() RETURNS trigger AS $delete_requests$
BEGIN
DELETE FROM requests WHERE user_id=NEW.user_id;
RETURN NEW;
END;
$delete_requests$
LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER delete_requests_trigger
    AFTER UPDATE OF is_open ON rooms
    FOR EACH ROW
    WHEN (OLD.is_open=true AND NEW.is_open=false)
    EXECUTE FUNCTION delete_requests();