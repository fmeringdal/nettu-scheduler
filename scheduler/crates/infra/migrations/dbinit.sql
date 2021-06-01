CREATE EXTENSION IF NOT EXISTS "uuid-ossp"; 

CREATE TABLE todos
(
    id integer NOT NULL,
    title text  NOT NULL,
    description text NOT NULL,
    "isFinished" boolean NOT NULL,
    CONSTRAINT todos_pkey PRIMARY KEY (id)
)