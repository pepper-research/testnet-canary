-- Your SQL goes here
CREATE TABLE "orpheus_tick"(
	"id" INT4 NOT NULL PRIMARY KEY,
	"product_id" INT4 NOT NULL,
	"timestamp" TIMESTAMP NOT NULL,
	"price" FLOAT8 NOT NULL,
	"confidence" FLOAT8 NOT NULL
);

CREATE INDEX ON "orpheus_tick" ("product_id");
CREATE INDEX ON "orpheus_tick" ("timestamp");