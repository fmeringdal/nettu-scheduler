CREATE OR REPLACE FUNCTION manage_updated_at(_tbl regclass) RETURNS VOID AS $$
BEGIN
    EXECUTE format('CREATE TRIGGER set_updated_at BEFORE UPDATE ON %s
                    FOR EACH ROW EXECUTE PROCEDURE set_updated_at()', _tbl);
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION set_updated_at() RETURNS trigger AS $$
BEGIN
    IF (
                NEW IS DISTINCT FROM OLD AND
                NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at
        ) THEN
        NEW.updated_at := CURRENT_TIMESTAMP;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

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