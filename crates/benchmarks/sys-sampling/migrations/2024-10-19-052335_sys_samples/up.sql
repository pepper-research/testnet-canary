-- Your SQL goes here

CREATE TABLE "sys_samples"(
	"id" SERIAL NOT NULL PRIMARY KEY,
	"timestamp" TIMESTAMP NOT NULL,
    "total_memory" numeric NOT NULL,
    "total_cpu" numeric NOT NULL,
    "memory_usage" numeric NOT NULL,
    "swap_usage" numeric NOT NULL,
    "cpu_usage" numeric NOT NULL,
    "process_cpu_usage" numeric NOT NULL,
    "process_memory_usage" numeric NOT NULL,
    "network_down" numeric NOT NULL,
    "network_up" numeric NOT NULL
);

CREATE INDEX ON "sys_samples" ("timestamp");