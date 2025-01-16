-- This file should undo anything in `up.sql`
ALTER TABLE "log" DROP COLUMN "level";
ALTER TABLE "log" DROP COLUMN "scope";
ALTER TABLE "log" ADD COLUMN "level" LOG_LEVEL NOT NULL;
ALTER TABLE "log" ADD COLUMN "scope" LOG_SCOPE NOT NULL;

DROP TABLE IF EXISTS "orpheus_tick";
