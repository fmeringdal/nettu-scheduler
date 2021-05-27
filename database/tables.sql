-- https://wiki.postgresql.org/wiki/Referential_Integrity_Tutorial_%26_Hacking_the_Referential_Integrity_tables

CREATE TABLE accounts (
    account_id uuid primary key DEFAULT uuid_generate_v4() NOT NULL,
    secret_api_key varchar(255) NOT NULL UNIQUE,
    public_jwt_key varchar(255),
);

-- https://www.youtube.com/watch?v=NfLRqTTbkkU

-- METADATA
-- https://dba.stackexchange.com/questions/115825/jsonb-with-indexing-vs-hstore
-- how to index ? Try to make table work, otherwise jsonb
-- same table for all metadata or own metadata table for each resource

-- CREATE TABLE metadata {
--     id uuid primary key DEFAULT uuid_generate_v4() NOT NULL,
--     "resource" varchar(),
--     "key" varchar(255) NOT NULL,
--     "value" varchar(255) NOT NULL,
-- }

CREATE TABLE metadata_users {
    id uuid primary key DEFAULT uuid_generate_v4() NOT NULL,
    user_id uuid,
    "key" varchar(255) NOT NULL,
    "value" varchar(255) NOT NULL,
    CONSTRAINT fk_user
        FOREIGN KEY(user_id) 
        REFERENCES users(id),
}