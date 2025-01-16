-- This file should undo anything in `up.sql`
ALTER TABLE "price_tick" DROP COLUMN "confidence";
ALTER TABLE "price_tick" ADD COLUMN "confidence" FLOAT8 NOT NULL;

