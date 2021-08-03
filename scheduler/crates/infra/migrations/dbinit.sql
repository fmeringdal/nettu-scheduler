-- DROP SCHEMA public CASCADE;
-- CREATE SCHEMA public;
-- \i ~/Projects/nettu-scheduler/scheduler/crates/infra/migrations/dbinit.sql;

CREATE EXTENSION IF NOT EXISTS "uuid-ossp"; 

-- TODO: Version on reminder jobs
-- TODO: Version on event
-- TODO: Create indexes
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
CREATE INDEX IF NOT EXISTS account_api_key ON accounts (secret_api_key);

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
    settings JSON NOT NULL,
    metadata text[] NOT NULL
);
CREATE INDEX IF NOT EXISTS metadata ON calendars USING GIN (metadata);
-- TODO: all columns immutable 
-- TODO: name -> externally_synced_calendars 
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
    service_uid uuid REFERENCES services(service_uid) ON DELETE CASCADE,
    metadata text[] NOT NULL
);
CREATE INDEX IF NOT EXISTS event_metadata ON calendar_events USING GIN (metadata);
-- TODO: all columns immutable 
-- TODO: name -> externally_synced_events
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

CREATE TABLE IF NOT EXISTS event_reminder_versions (
    event_uid uuid NOT NULL REFERENCES calendar_events(event_uid) ON DELETE CASCADE,
    "version" BIGINT NOT NULL,
    PRIMARY KEY(event_uid, "version")
);

-- Removes reminders from the reminders table when the 
-- event_reminder_versions.version is updated
create or replace function
    remove_old_reminders()
    returns trigger
as $$
begin
    DELETE FROM reminders
    WHERE
        event_uid = old.event_uid;
    DELETE FROM calendar_event_reminder_expansion_jobs
    WHERE
        event_uid = old.event_uid;
    return new;
end;
$$ language plpgsql;

CREATE TRIGGER remove_old_reminders
    BEFORE UPDATE ON event_reminder_versions
FOR EACH ROW
    WHEN (OLD.version IS DISTINCT FROM NEW.version)
EXECUTE PROCEDURE remove_old_reminders();

COMMENT ON TABLE event_reminder_versions IS 
'There are three usecases which can generate event reminders. The first is an 
api call to create an event with reminders. The second is an api call
from to update an existing event. The third is a scheduled job from the 
calendar_event_reminder_expansion_jobs table. If the update event and
scheduled job happens at the same time it is possible that the scheduled job
generates reminders for an outdated calendar event. Therefore the update event usecase
will increment the version number for the calendar event and the scheduled job
will not be able to generate reminders for the old version which no longer will be
in this table';

CREATE TABLE IF NOT EXISTS reminders (
    event_uid uuid NOT NULL REFERENCES calendar_events(event_uid) ON DELETE CASCADE,
    account_uid uuid NOT NULL REFERENCES accounts(account_uid) ON DELETE CASCADE,
    remind_at BIGINT NOT NULL,
    "version" BIGINT NOT NULL,
    identifier text NOT NULL,
    PRIMARY KEY(event_uid, remind_at, identifier),
    FOREIGN KEY(event_uid, "version") REFERENCES event_reminder_versions(event_uid, "version")
);
COMMENT ON COLUMN reminders.identifier IS 
'User defined identifier to be able to seperate reminders at same timestamp for 
the same event';

CREATE TABLE IF NOT EXISTS calendar_event_reminder_expansion_jobs (
    -- There can only be one job at the time for an event
    event_uid uuid PRIMARY KEY NOT NULL REFERENCES calendar_events(event_uid) ON DELETE CASCADE,
    "timestamp" BIGINT NOT NULL,
    "version" BIGINT NOT NULL,
    FOREIGN KEY (event_uid, "version") REFERENCES event_reminder_versions(event_uid, "version")
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
	PRIMARY KEY(service_uid, user_uid),
    CHECK (
        NOT (available_calendar_uid IS NOT NULL AND available_schedule_uid IS NOT NULL)
    )
);

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
    service_uid uuid NOT NULL REFERENCES services(service_uid) ON DELETE CASCADE,
    user_uid uuid NOT NULL REFERENCES users(user_uid) ON DELETE CASCADE,
    calendar_uid uuid NOT NULL REFERENCES calendars(calendar_uid) ON DELETE CASCADE,
    FOREIGN KEY(service_uid, user_uid) REFERENCES service_users (service_uid, user_uid) ON DELETE CASCADE,
	PRIMARY KEY(service_uid, user_uid, calendar_uid)
);

-- TODO: maybe just add a count column ?
CREATE TABLE IF NOT EXISTS service_reservations (
    reservation_uid uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL,
    service_uid uuid NOT NULL REFERENCES services(service_uid) ON DELETE CASCADE,
    "timestamp" BIGINT NOT NULL
);
