-- This file should undo anything in `up.sql`
CREATE TABLE "orpheus_tick"(
	"id" INT4 NOT NULL PRIMARY KEY,
	"product_id" INT4 NOT NULL,
	"timestamp" TIMESTAMP NOT NULL,
	"price" FLOAT8 NOT NULL,
	"confidence" FLOAT8 NOT NULL
);

DROP TABLE IF EXISTS "price_tick";
