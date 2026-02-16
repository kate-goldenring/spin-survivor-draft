PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS players (
  name TEXT NOT NULL PRIMARY KEY,
  season INTEGER,
  voted_out DATE
);

CREATE TABLE IF NOT EXISTS drafters (
  name TEXT NOT NULL PRIMARY KEY,
  season INTEGER
);

CREATE TABLE IF NOT EXISTS drafterDrafts (
  drafter_id INTEGER,
  player_id INTEGER,
  FOREIGN KEY (drafter_id) REFERENCES drafters(name) ON DELETE CASCADE,
  FOREIGN KEY (player_id) REFERENCES players(name)  ON DELETE CASCADE,
  PRIMARY KEY (drafter_id, player_id)
);