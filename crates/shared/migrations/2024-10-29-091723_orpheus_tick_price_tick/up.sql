-- Your SQL goes here
DROP TABLE IF EXISTS "orpheus_tick";
CREATE TABLE "price_tick"(
	"id" INT4 NOT NULL PRIMARY KEY,
	"product_index" INT4 NOT NULL,
	"timestamp" TIMESTAMP NOT NULL,
	"price" FLOAT8 NOT NULL,
	"confidence" FLOAT8 NOT NULL
);

