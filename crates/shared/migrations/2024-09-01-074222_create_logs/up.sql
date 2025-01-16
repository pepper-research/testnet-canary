-- Your SQL goes here
CREATE TYPE log_level AS ENUM ('info', 'warn', 'error');
CREATE TYPE log_scope AS ENUM ('other', 'risk', 'dex', 'aaob', 'instruments', 'none');

CREATE TABLE "log"(
	"id" SERIAL NOT NULL PRIMARY KEY,
	"timestamp" TIMESTAMP NOT NULL,
	"level" log_level NOT NULL,
	"message" TEXT NOT NULL,
	"scope" log_scope NOT NULL
);

CREATE INDEX ON "log" ("timestamp");
CREATE INDEX ON "log" ("level");
CREATE INDEX ON "log" ("scope");
