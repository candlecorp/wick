CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE blog (
    id UUID NOT NULL DEFAULT uuid_generate_v4(),
    user_id character varying(100) NOT NULL,
    title character varying(280) NOT NULL,
    body character varying(10000) NOT NULL,
    "time" timestamp with time zone NOT NULL
);

ALTER TABLE ONLY blog
    ADD CONSTRAINT blog_pkey PRIMARY KEY (id);
