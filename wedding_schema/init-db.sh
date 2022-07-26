#!/bin/bash
set -e

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname wedding <<-EOSQL


DROP TABLE IF EXISTS "invite";
DROP TABLE IF EXISTS "guest";
DROP SEQUENCE IF EXISTS invitee_uid_seq;
DROP SEQUENCE IF EXISTS guestid_uid_seq;

CREATE SEQUENCE invitee_uid_seq INCREMENT 1 MINVALUE 1 MAXVALUE 2147483647 CACHE 1;

CREATE TABLE "public"."invite" (
    "uid" integer DEFAULT nextval('invitee_uid_seq') NOT NULL,
    "random" text DEFAULT md5(random()::text) NOT NULL,
    "email" text,
    "password" text,
    "names" text NOT NULL,
    CONSTRAINT "invitee_pkey" PRIMARY KEY ("uid"),
    CONSTRAINT "invitee_random" UNIQUE ("random")
) WITH (oids = false);

CREATE SEQUENCE guestid_uid_seq INCREMENT 1 MINVALUE 1000 MAXVALUE 9999999 CACHE 1;
CREATE TABLE "public".guest (
    "guestid" integer DEFAULT nextval('guestid_uid_seq') NOT NULL,
    "invite_uid" integer NOT NULL,
    "name" text not null,
    "attending" integer default 0 not null,
    "diet" text[] not null default '{}'
);

GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO wedding;

EOSQL
