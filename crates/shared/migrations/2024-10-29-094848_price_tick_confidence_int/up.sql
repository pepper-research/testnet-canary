-- Your SQL goes here
ALTER TABLE "price_tick" DROP COLUMN "confidence";
ALTER TABLE "price_tick" ADD COLUMN "confidence" INT4 NOT NULL;

