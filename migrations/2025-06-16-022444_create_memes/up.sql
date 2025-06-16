-- Your SQL goes here
CREATE TABLE "memes"(
	"id" SERIAL NOT NULL PRIMARY KEY,
	"spoiler" BOOL NOT NULL,
	"text" TEXT NOT NULL,
	"timestamp" TIMESTAMP NOT NULL,
	"account" TEXT NOT NULL,
	"channel" TEXT NOT NULL,
	"telegram_id" INTEGER UNIQUE
);
