-- DROP SCHEMA public CASCADE;
-- CREATE SCHEMA public;
-- \i ~/Projects/nettu-scheduler/scheduler/crates/infra/migrations/dbinit.sql;

CREATE EXTENSION IF NOT EXISTS "uuid-ossp"; 

-- TODO: Create domain types and do type casting in queries
-- TODO: better naming conventions
-- TODO: immutable triggers
-- TODO: Create views
-- TODO: Split schema into multiple files

-- CREATE TYPE ext_calendar_provider AS ENUM ('google', 'outlook');

CREATE TABLE IF NOT EXISTS accounts (
    account_uid uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL,
    secret_api_key text NOT NULL UNIQUE,
    public_jwt_key text,
    settings JSONB NOT NULL
);
CREATE TABLE IF NOT EXISTS account_integrations (
    account_uid uuid NOT NULL REFERENCES accounts(account_uid) ON DELETE CASCADE,
    client_id text NOT NULL,
    client_secret text NOT NULL,
    redirect_uri text NOT NULL,
    "provider" text NOT NULL,
    -- Account can only have one intergration per provider
    PRIMARY KEY(account_uid, "provider")
);

CREATE TABLE IF NOT EXISTS users (
    user_uid uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL,
    account_uid uuid NOT NULL REFERENCES accounts(account_uid) ON DELETE CASCADE,
    metadata text[] NOT NULL
);
CREATE INDEX IF NOT EXISTS user_metadata ON users USING GIN (metadata);

CREATE TABLE IF NOT EXISTS user_integrations (
    account_uid uuid NOT NULL REFERENCES accounts(account_uid) ON DELETE CASCADE,
    user_uid uuid NOT NULL REFERENCES users(user_uid) ON DELETE CASCADE,
    refresh_token text NOT NULL,
    access_token text NOT NULL,
    access_token_expires_ts BIGINT NOT NULL,
    "provider" text NOT NULL,
    -- User cannot have multiple integrations to the same provider
    PRIMARY KEY(user_uid, "provider"),
    FOREIGN KEY(account_uid, "provider") REFERENCES account_integrations(account_uid, "provider") ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS services (
    service_uid uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL,
    account_uid uuid NOT NULL REFERENCES accounts(account_uid) ON DELETE CASCADE,
    multi_person JSON NOT NULL,
    metadata text[] NOT NULL
);
CREATE INDEX IF NOT EXISTS service_metadata ON services USING GIN (metadata);

CREATE TABLE IF NOT EXISTS calendars (
    calendar_uid uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL,
    user_uid uuid NOT NULL REFERENCES users(user_uid) ON DELETE CASCADE,
    account_uid uuid NOT NULL REFERENCES accounts(account_uid) ON DELETE CASCADE,
    -- synced JSON ,
    settings JSON NOT NULL,
    metadata text[] NOT NULL
);
CREATE INDEX IF NOT EXISTS metadata ON calendars USING GIN (metadata);
-- TODO: all columns immutable 
CREATE TABLE IF NOT EXISTS calendar_ext_synced_calendars (
    calendar_uid uuid NOT NULL REFERENCES calendars(calendar_uid) ON DELETE CASCADE,
    user_uid uuid NOT NULL REFERENCES users(user_uid) ON DELETE CASCADE,
    ext_calendar_id text NOT NULL,
    "provider" text NOT NULL,
	PRIMARY KEY(calendar_uid, "provider", ext_calendar_id),
    FOREIGN KEY(user_uid, "provider") REFERENCES user_integrations (user_uid, "provider") ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS calendar_events (
    event_uid uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL,
    calendar_uid uuid NOT NULL REFERENCES calendars(calendar_uid) ON DELETE CASCADE,
    user_uid uuid NOT NULL REFERENCES users(user_uid) ON DELETE CASCADE,
    account_uid uuid NOT NULL REFERENCES accounts(account_uid) ON DELETE CASCADE,
    start_ts BIGINT NOT NULL,
    duration BIGINT NOT NULL,
    end_ts BIGINT NOT NULL,
    busy boolean NOT NULL,
    created BIGINT NOT NULL,
    updated BIGINT NOT NULL,
    recurrence JSON,
    exdates BIGINT[] NOT NULL,
    reminder JSON,
    -- synced_events JSON,
    service_uid uuid REFERENCES services(service_uid) ON DELETE CASCADE,
    metadata text[] NOT NULL
);
CREATE INDEX IF NOT EXISTS event_metadata ON calendar_events USING GIN (metadata);
-- TODO: all columns immutable 
CREATE TABLE IF NOT EXISTS calendar_ext_synced_events (
    event_uid uuid NOT NULL REFERENCES calendar_events(event_uid) ON DELETE CASCADE,
    calendar_uid uuid NOT NULL REFERENCES calendars(calendar_uid) ON DELETE CASCADE,
    user_uid uuid NOT NULL REFERENCES users(user_uid) ON DELETE CASCADE,
    ext_calendar_id text NOT NULL,
    ext_calendar_event_id text NOT NULL,
    "provider" text NOT NULL,
	PRIMARY KEY(event_uid, "provider", ext_calendar_id, ext_calendar_event_id),
    FOREIGN KEY(calendar_uid, "provider", ext_calendar_id) REFERENCES calendar_ext_synced_calendars (calendar_uid, "provider", ext_calendar_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS calendar_event_reminder_expansion_jobs (
    -- TODO: is this reminder_uid needed? Why not pk = (event_uid, timestamp)
    job_uid uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL,
    event_uid uuid NOT NULL REFERENCES calendar_events(event_uid) ON DELETE CASCADE,
    "timestamp" BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS reminders (
    -- TODO: is this reminder_uid needed? Why not pk = (event_uid, remind_at)
    reminder_uid uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL,
    event_uid uuid NOT NULL REFERENCES calendar_events(event_uid) ON DELETE CASCADE,
    account_uid uuid NOT NULL REFERENCES accounts(account_uid) ON DELETE CASCADE,
    remind_at BIGINT NOT NULL,
    "priority" SMALLINT NOT NULL
);

CREATE TABLE IF NOT EXISTS schedules (
    schedule_uid uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL,
    user_uid uuid NOT NULL REFERENCES users(user_uid) ON DELETE CASCADE,
    account_uid uuid NOT NULL REFERENCES accounts(account_uid) ON DELETE CASCADE,
    rules JSON NOT NULL,
    timezone text NOT NULL,
    metadata text[] NOT NULL
);
CREATE INDEX IF NOT EXISTS schedule_metadata ON schedules USING GIN (metadata);


CREATE TABLE IF NOT EXISTS service_users (
    service_uid uuid NOT NULL REFERENCES services(service_uid) ON DELETE CASCADE,
    user_uid uuid NOT NULL REFERENCES users(user_uid) ON DELETE CASCADE,
    available_calendar_uid uuid REFERENCES calendars(calendar_uid) ON DELETE SET NULL,
    available_schedule_uid uuid REFERENCES schedules(schedule_uid) ON DELETE SET NULL,
    buffer_after BIGINT NOT NULL, 
    buffer_before BIGINT NOT NULL, 
    closest_booking_time BIGINT NOT NULL, 
    furthest_booking_time BIGINT, 
    -- google_busy_calendars text[] NOT NULL,
    -- outlook_busy_calendars text[] NOT NULL,
	PRIMARY KEY(service_uid, user_uid),
    CHECK (
        NOT (available_calendar_uid IS NOT NULL AND available_schedule_uid IS NOT NULL)
    )
);

-- TODO: Maybe create subtype for service_user_busy_calendars ??
CREATE TABLE IF NOT EXISTS service_user_external_busy_calendars (
    service_uid uuid NOT NULL REFERENCES services(service_uid) ON DELETE CASCADE,
    user_uid uuid NOT NULL REFERENCES users(user_uid) ON DELETE CASCADE,
    ext_calendar_id text NOT NULL,
    "provider" text NOT NULL,
	PRIMARY KEY(service_uid, user_uid, "provider", ext_calendar_id),
    FOREIGN KEY(service_uid, user_uid) REFERENCES service_users (service_uid, user_uid) ON DELETE CASCADE,
    FOREIGN KEY(user_uid, "provider") REFERENCES user_integrations (user_uid, "provider") ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS service_user_busy_calendars (
    -- maybe add service_user_id here ?? 
    service_uid uuid NOT NULL REFERENCES services(service_uid) ON DELETE CASCADE,
    user_uid uuid NOT NULL REFERENCES users(user_uid) ON DELETE CASCADE,
    calendar_uid uuid NOT NULL REFERENCES calendars(calendar_uid) ON DELETE CASCADE,
    FOREIGN KEY(service_uid, user_uid) REFERENCES service_users (service_uid, user_uid) ON DELETE CASCADE,
	PRIMARY KEY(service_uid, user_uid, calendar_uid)
);

CREATE TABLE IF NOT EXISTS service_reservations (
    reservation_uid uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL,
    service_uid uuid NOT NULL REFERENCES services(service_uid) ON DELETE CASCADE,
    "timestamp" BIGINT NOT NULL
);
