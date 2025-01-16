-- Your SQL goes here

CREATE TABLE "samples"(
	"id" SERIAL NOT NULL PRIMARY KEY,
	"timestamp" TIMESTAMP NOT NULL,
	"ping_latency" numeric NOT NULL,
	"nonce_latency" numeric NOT NULL,
	"publish_batch_latency" numeric NOT NULL,
	"confirmation_latency" numeric NOT NULL,
	"e2e_latency" numeric NOT NULL
);

CREATE INDEX ON "samples" ("timestamp");
