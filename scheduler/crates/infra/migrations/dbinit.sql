CREATE EXTENSION IF NOT EXISTS "uuid-ossp"; 

CREATE TABLE IF NOT EXISTS accounts (
    account_uid uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL,
    secret_api_key varchar(255) NOT NULL UNIQUE,
    public_jwt_key varchar(1024),
    settings JSONB NOT NULL
);
CREATE TABLE IF NOT EXISTS users (
    user_uid uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL,
    account_uid uuid NOT NULL REFERENCES accounts(account_uid) ON DELETE CASCADE,
    integrations JSON,
    metadata text[] NOT NULL
);
CREATE INDEX IF NOT EXISTS metadata ON users USING GIN (metadata);

CREATE TABLE IF NOT EXISTS services (
    service_uid uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL,
    account_uid uuid NOT NULL REFERENCES accounts(account_uid) ON DELETE CASCADE,
    multi_person JSON NOT NULL,
    metadata text[] NOT NULL
);
CREATE INDEX IF NOT EXISTS metadata ON services USING GIN (metadata);

CREATE TABLE IF NOT EXISTS calendars (
    calendar_uid uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL,
    user_uid uuid NOT NULL REFERENCES users(user_uid) ON DELETE CASCADE,
    account_uid uuid NOT NULL REFERENCES accounts(account_uid) ON DELETE CASCADE,
    synced JSON ,
    settings JSON NOT NULL,
    metadata text[] NOT NULL
);
CREATE INDEX IF NOT EXISTS metadata ON calendars USING GIN (metadata);

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
    synced_events JSON,
    service_uid uuid REFERENCES services(service_uid) ON DELETE CASCADE,
    metadata text[] NOT NULL
);
CREATE INDEX IF NOT EXISTS metadata ON calendar_events USING GIN (metadata);

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
CREATE INDEX IF NOT EXISTS metadata ON schedules USING GIN (metadata);


CREATE TABLE IF NOT EXISTS service_users (
    service_uid uuid NOT NULL REFERENCES services(service_uid) ON DELETE CASCADE,
    user_uid uuid NOT NULL REFERENCES users(user_uid) ON DELETE CASCADE,
    available_calendar_uid uuid REFERENCES calendars(calendar_uid) ON DELETE SET NULL,
    available_schedule_uid uuid REFERENCES schedules(schedule_uid) ON DELETE SET NULL,
    buffer_after BIGINT NOT NULL, 
    buffer_before BIGINT NOT NULL, 
    closest_booking_time BIGINT NOT NULL, 
    furthest_booking_time BIGINT, 
    google_busy_calendars text[] NOT NULL,
	PRIMARY KEY(service_uid, user_uid),
    CHECK (
        NOT (available_calendar_uid IS NOT NULL AND available_schedule_uid IS NOT NULL)
    )
);

CREATE TABLE IF NOT EXISTS service_user_busy_calendars (
    service_uid uuid NOT NULL REFERENCES services(service_uid) ON DELETE CASCADE,
    user_uid uuid NOT NULL REFERENCES users(user_uid) ON DELETE CASCADE,
    calendar_uid uuid NOT NULL REFERENCES calendars(calendar_uid) ON DELETE CASCADE,
	PRIMARY KEY(service_uid, user_uid, calendar_uid)
);

CREATE TABLE IF NOT EXISTS service_reservations (
    reservation_uid uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL,
    service_uid uuid NOT NULL REFERENCES services(service_uid) ON DELETE CASCADE,
    "timestamp" BIGINT NOT NULL
);