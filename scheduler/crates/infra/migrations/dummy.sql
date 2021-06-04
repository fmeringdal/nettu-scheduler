CREATE EXTENSION IF NOT EXISTS "uuid-ossp"; 

-- https://github.com/launchbadge/sqlx/blob/master/examples/postgres/json/src/main.rs

CREATE TABLE IF NOT EXISTS accounts (
    account_uid uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL,
    secret_api_key varchar(255) NOT NULL UNIQUE,
    public_jwt_key varchar(255),
    settings JSONB NOT NULL
);
CREATE TABLE IF NOT EXISTS users (
    user_uid uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL,
    account_uid uuid NOT NULL REFERENCES accounts(account_uid) ON DELETE CASCADE,
    metadata text[] NOT NULL
);
CREATE INDEX IF NOT EXISTS metadata ON users USING GIN (metadata);
CREATE TABLE IF NOT EXISTS calendars (
    calendar_uid uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL,
    user_uid uuid NOT NULL REFERENCES users(user_uid) ON DELETE CASCADE,
    account_uid uuid NOT NULL REFERENCES accounts(account_uid) ON DELETE CASCADE,
    settings JSON NOT NULL,
    metadata text[] NOT NULL
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
    is_service boolean NOT NULL,
    metadata text[] NOT NULL
);

CREATE TABLE IF NOT EXISTS calendar_event_reminder_expansion_jobs (
    job_uid uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL,
    event_uid uuid NOT NULL REFERENCES calendar_events(event_uid) ON DELETE CASCADE,
    "timestamp" BIGINT NOT NULL
);
CREATE TABLE IF NOT EXISTS reminders (
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
    timezone varchar(128) NOT NULL,
    metadata text[] NOT NULL
);
CREATE TABLE IF NOT EXISTS services (
    service_uid uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL,
    account_uid uuid NOT NULL REFERENCES accounts(account_uid) ON DELETE CASCADE,
    metadata text[] NOT NULL
);
CREATE TABLE IF NOT EXISTS service_users (
    service_uid uuid NOT NULL REFERENCES services(service_uid) ON DELETE CASCADE,
    user_uid uuid NOT NULL REFERENCES users(user_uid) ON DELETE CASCADE,
    available_calendar_uid uuid REFERENCES calendars(calendar_uid) ON DELETE CASCADE,
    available_schedule_uid uuid REFERENCES schedules(schedule_uid) ON DELETE CASCADE,
    "buffer" BIGINT NOT NULL, 
    closest_booking_time BIGINT NOT NULL, 
    furthest_booking_time BIGINT, 
	PRIMARY KEY(service_uid, user_uid),
    CHECK (
        (available_calendar_uid IS NOT NULL AND available_schedule_uid IS NULL) OR
        (available_calendar_uid IS NULL AND available_schedule_uid IS NOT NULL) OR
        (available_calendar_uid IS NOT NULL AND available_schedule_uid IS NULL)
    )
);
-- Maybe this can be created better ?
CREATE TABLE IF NOT EXISTS service_user_calendars (
    service_uid uuid NOT NULL REFERENCES services(service_uid) ON DELETE CASCADE,
    user_uid uuid NOT NULL REFERENCES users(user_uid) ON DELETE CASCADE,
    calendar_uid uuid NOT NULL REFERENCES calendars(calendar_uid) ON DELETE CASCADE,
	PRIMARY KEY(service_uid, user_uid, calendar_uid)
);
-- Maybe this can be created better ?
CREATE TABLE IF NOT EXISTS service_user_schedules (
    service_uid uuid NOT NULL REFERENCES services(service_uid) ON DELETE CASCADE,
    user_uid uuid NOT NULL REFERENCES users(user_uid) ON DELETE CASCADE,
    schedule_uid uuid NOT NULL REFERENCES schedules(schedule_uid) ON DELETE CASCADE,
	PRIMARY KEY(service_uid, user_uid, schedule_uid)
);

-- CREATE TABLE IF NOT EXISTS metadata (
--     metadata_id uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL,
-- 	-- PRIMARY KEY(calendar_uid, event_uid, user_uid, schedule_uid, service_uid, "key"),
--     "key" varchar(255) NOT NULL,
--     "value" varchar(255) NOT NULL,
--     calendar_uid uuid REFERENCES calendars(calendar_uid) ON DELETE CASCADE,
--     event_uid uuid REFERENCES calendar_events(event_uid) ON DELETE CASCADE,
--     user_uid uuid REFERENCES users(user_uid) ON DELETE CASCADE,
--     schedule_uid uuid REFERENCES schedules(schedule_uid) ON DELETE CASCADE,
--     service_uid uuid REFERENCES services(service_uid) ON DELETE CASCADE,
--     CHECK (
--         (calendar_uid IS NOT NULL AND event_uid IS NULL AND user_uid IS NULL AND schedule_uid IS NULL AND service_uid IS NULL) OR 
--         (calendar_uid IS NULL AND event_uid IS NOT NULL AND user_uid IS NULL AND schedule_uid IS NULL AND service_uid IS NULL) OR 
--         (calendar_uid IS NULL AND event_uid IS NULL AND user_uid IS NOT NULL AND schedule_uid IS NULL AND service_uid IS NULL) OR 
--         (calendar_uid IS NULL AND event_uid IS NULL AND user_uid IS NULL AND schedule_uid IS NOT NULL AND service_uid IS NULL) OR 
--         (calendar_uid IS NULL AND event_uid IS NULL AND user_uid IS NULL AND schedule_uid IS NULL AND service_uid IS NOT NULL)
--     )
-- );
