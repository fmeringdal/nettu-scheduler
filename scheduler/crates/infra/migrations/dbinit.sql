-- DROP SCHEMA public CASCADE;
-- CREATE SCHEMA public;
-- \i ~/Projects/nettu-scheduler/scheduler/crates/infra/migrations/dbinit.sql;
begin;

CREATE EXTENSION IF NOT EXISTS "uuid-ossp"; 

-- TODO: https://stackoverflow.com/questions/30770098/postgresql-increment-if-exist-or-create-a-new-row
-- TODO: Better docs and comments on schema
-- TODO: Better indexing strategy
-- TODO: Have external calendars and events relations for better normalization / consistency
-- TODO: Split schema into multiple files

------------------ DOMAIN
CREATE DOMAIN ext_calendar_provider AS TEXT
    NOT NULL
    CHECK (VALUE in ('google', 'outlook'));
COMMENT ON DOMAIN ext_calendar_provider IS
'external calendar provider names that are supported by nettu scheduler';

CREATE DOMAIN entity_version AS BIGINT
  DEFAULT 1
  NOT NULL
  CHECK (
   VALUE > 0
  );
COMMENT ON DOMAIN entity_version IS
'standard column for entity version';



------------------ PROCEDURES

-- immutable_columns() will make the column names immutable which are passed as
-- parameters when the trigger is created. It raises error code 23601 which is a
-- class 23 integrity constraint violation: immutable column  
create or replace function
  immutable_columns()
  returns trigger
as $$
declare 
	col_name text; 
	new_value text;
	old_value text;
begin
  foreach col_name in array tg_argv loop
    execute format('SELECT $1.%I', col_name) into new_value using new;
    execute format('SELECT $1.%I', col_name) into old_value using old;
  	if new_value is distinct from old_value then
      raise exception 'immutable column: %.%', tg_table_name, col_name using
        errcode = '23601', 
        schema = tg_table_schema,
        table = tg_table_name,
        column = col_name;
  	end if;
  end loop;
  return new;
end;
$$ language plpgsql;

comment on function
  immutable_columns()
is
  'function used in before update triggers to make columns immutable';



------------------ RELATIONS, INDEXES AND TRIGGERS

CREATE TABLE IF NOT EXISTS accounts (
    account_uid uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL,
    secret_api_key text NOT NULL UNIQUE,
    public_jwt_key text,
    settings JSONB NOT NULL
);
CREATE INDEX IF NOT EXISTS account_api_key ON accounts (secret_api_key);

create trigger
    immutable_columns
before
update on accounts
    for each row execute procedure immutable_columns('secret_api_key');

CREATE TABLE IF NOT EXISTS account_integrations (
    account_uid uuid NOT NULL REFERENCES accounts(account_uid) ON DELETE CASCADE,
    client_id text NOT NULL,
    client_secret text NOT NULL,
    redirect_uri text NOT NULL,
    "provider" ext_calendar_provider NOT NULL,
    -- Account can only have one intergration per provider
    PRIMARY KEY(account_uid, "provider")
);
create trigger
    immutable_columns
before
update on account_integrations
    for each row execute procedure immutable_columns('account_uid', 'provider');

CREATE TABLE IF NOT EXISTS users (
    user_uid uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL,
    account_uid uuid NOT NULL REFERENCES accounts(account_uid) ON DELETE CASCADE,
    metadata text[] NOT NULL
);
CREATE INDEX IF NOT EXISTS user_metadata ON users USING GIN (metadata);

create trigger
    immutable_columns
before
update on users
    for each row execute procedure immutable_columns('user_uid', 'account_uid');

CREATE TABLE IF NOT EXISTS user_integrations (
    account_uid uuid NOT NULL,
    user_uid uuid NOT NULL,
    refresh_token text NOT NULL,
    access_token text NOT NULL,
    access_token_expires_ts BIGINT NOT NULL,
    "provider" ext_calendar_provider NOT NULL,
    -- User cannot have multiple integrations to the same provider
    PRIMARY KEY(user_uid, "provider"),
    FOREIGN KEY(account_uid, "provider") REFERENCES account_integrations(account_uid, "provider") ON DELETE CASCADE
);
create trigger
    immutable_columns
before
update on user_integrations
    for each row execute procedure immutable_columns('account_uid', 'user_uid', 'provider');

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
    settings JSON NOT NULL,
    metadata text[] NOT NULL
);
CREATE INDEX IF NOT EXISTS metadata ON calendars USING GIN (metadata);
create trigger
    immutable_columns
before
update on calendars
    for each row execute procedure immutable_columns('calendar_uid', 'user_uid');

CREATE TABLE IF NOT EXISTS externally_synced_calendars (
    calendar_uid uuid NOT NULL REFERENCES calendars(calendar_uid) ON DELETE CASCADE,
    user_uid uuid NOT NULL,
    ext_calendar_id text NOT NULL,
    "provider" ext_calendar_provider NOT NULL,
	PRIMARY KEY(calendar_uid, "provider", ext_calendar_id),
    FOREIGN KEY(user_uid, "provider") REFERENCES user_integrations (user_uid, "provider") ON DELETE CASCADE
);
create trigger
    immutable_columns
before
update on externally_synced_calendars
    for each row execute procedure immutable_columns('calendar_uid', 'user_uid', 'ext_calendar_id', 'provider');

CREATE TABLE IF NOT EXISTS calendar_events (
    event_uid uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL,
    calendar_uid uuid NOT NULL REFERENCES calendars(calendar_uid) ON DELETE CASCADE,
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
create trigger
    immutable_columns
before
update on calendar_events
    -- 'updated' column is also immutable but is is mutated in tests often so it cannot be added here :(
    for each row execute procedure immutable_columns('event_uid', 'calendar_uid');

CREATE TABLE IF NOT EXISTS externally_synced_calendar_events (
    event_uid uuid NOT NULL REFERENCES calendar_events(event_uid) ON DELETE CASCADE,
    calendar_uid uuid NOT NULL REFERENCES calendars(calendar_uid) ON DELETE CASCADE,
    -- user_uid uuid NOT NULL,
    ext_calendar_id text NOT NULL,
    ext_calendar_event_id text NOT NULL,
    "provider" ext_calendar_provider NOT NULL,
	PRIMARY KEY(event_uid, "provider", ext_calendar_id, ext_calendar_event_id),
    FOREIGN KEY(calendar_uid, "provider", ext_calendar_id) REFERENCES externally_synced_calendars (calendar_uid, "provider", ext_calendar_id) ON DELETE CASCADE
);
create trigger
    immutable_columns
before
update on externally_synced_calendar_events
    for each row execute procedure immutable_columns('event_uid', 'calendar_uid', 'ext_calendar_id', 'ext_calendar_event_id', 'provider');

CREATE TABLE IF NOT EXISTS event_reminder_versions (
    event_uid uuid NOT NULL REFERENCES calendar_events(event_uid) ON DELETE CASCADE,
    "version" entity_version NOT NULL,
    PRIMARY KEY(event_uid, "version")
);
create trigger
    immutable_columns
before
update on event_reminder_versions
    for each row execute procedure immutable_columns('event_uid', 'version');

COMMENT ON TABLE event_reminder_versions IS 
'There are three usecases which can generate event reminders. 
1. API call to create an event with reminders. 
2. API call to update an existing event. 
3. A scheduled job from the calendar_event_reminder_generation_jobs table. 

If the update event and scheduled job happens at the same time it is possible that the scheduled job
generates reminders for an outdated calendar event. Therefore the update event usecase
will increment the version number for the calendar event and the scheduled job
will not be able to generate reminders for the old version which no longer will be
in this table';

CREATE TABLE IF NOT EXISTS reminders (
    event_uid uuid NOT NULL REFERENCES calendar_events(event_uid) ON DELETE CASCADE,
    account_uid uuid NOT NULL,
    remind_at BIGINT NOT NULL,
    "version" entity_version NOT NULL,
    identifier text NOT NULL,
    PRIMARY KEY(event_uid, remind_at, identifier),
    FOREIGN KEY(event_uid, "version") REFERENCES event_reminder_versions(event_uid, "version") ON DELETE CASCADE
);
COMMENT ON COLUMN reminders.identifier IS 
'User defined identifier to be able to seperate reminders at same timestamp for 
the same event';

create trigger
    immutable_columns
before
update on reminders
    for each row execute procedure immutable_columns('event_uid', 'account_uid', 'version');

CREATE TABLE IF NOT EXISTS calendar_event_reminder_generation_jobs (
    -- There can only be one job at the time for an event
    event_uid uuid PRIMARY KEY NOT NULL,
    "timestamp" BIGINT NOT NULL,
    "version" entity_version NOT NULL,
    FOREIGN KEY (event_uid, "version") REFERENCES event_reminder_versions(event_uid, "version") ON DELETE CASCADE
);
create trigger
    immutable_columns
before
update on calendar_event_reminder_generation_jobs
    for each row execute procedure immutable_columns('event_uid', 'timestamp', 'version');


CREATE TABLE IF NOT EXISTS schedules (
    schedule_uid uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL,
    user_uid uuid NOT NULL REFERENCES users(user_uid) ON DELETE CASCADE,
    rules JSON NOT NULL,
    timezone text NOT NULL,
    metadata text[] NOT NULL
);
CREATE INDEX IF NOT EXISTS schedule_metadata ON schedules USING GIN (metadata);
create trigger
    immutable_columns
before
update on schedules
    for each row execute procedure immutable_columns('schedule_uid', 'user_uid');


-- TODO: how to make sure user is owner of calendar and schedule  ?
-- TODO: how to make sure user and service is in same account  ?
-- https://stackoverflow.com/questions/66911060/setting-a-composite-foreign-key-to-null-in-postgres
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
create trigger
    immutable_columns
before
update on service_users
    for each row execute procedure immutable_columns('service_uid', 'user_uid');

CREATE TABLE IF NOT EXISTS service_user_external_busy_calendars (
    service_uid uuid NOT NULL,
    user_uid uuid NOT NULL,
    ext_calendar_id text NOT NULL,
    "provider" ext_calendar_provider NOT NULL,
	PRIMARY KEY(service_uid, user_uid, "provider", ext_calendar_id),
    FOREIGN KEY(service_uid, user_uid) REFERENCES service_users (service_uid, user_uid) ON DELETE CASCADE,
    FOREIGN KEY(user_uid, "provider") REFERENCES user_integrations (user_uid, "provider") ON DELETE CASCADE
);
create trigger
    immutable_columns
before
update on service_user_external_busy_calendars
    for each row execute procedure immutable_columns('service_uid', 'user_uid', 'ext_calendar_id', 'provider');

CREATE TABLE IF NOT EXISTS service_user_busy_calendars (
    service_uid uuid NOT NULL,
    user_uid uuid NOT NULL,
    calendar_uid uuid NOT NULL REFERENCES calendars(calendar_uid),
    FOREIGN KEY(service_uid, user_uid) REFERENCES service_users (service_uid, user_uid) ON DELETE CASCADE,
	PRIMARY KEY(service_uid, user_uid, calendar_uid)
);
create trigger
    immutable_columns
before
update on service_user_busy_calendars
    for each row execute procedure immutable_columns('service_uid', 'user_uid', 'calendar_uid');

-- TODO: Trigger to delete
CREATE TABLE IF NOT EXISTS service_reservations (
    service_uid uuid NOT NULL REFERENCES services(service_uid) ON DELETE CASCADE,
    "timestamp" BIGINT NOT NULL,
    "count" BIGINT NOT NULL DEFAULT 1 CHECK ("count" >= 0),
    PRIMARY KEY(service_uid, "timestamp")
);

commit;
