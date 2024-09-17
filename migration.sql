CREATE TABLE IF NOT EXISTS players (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  season INTEGER,
  voted_out DATE
);

CREATE TABLE IF NOT EXISTS drafters (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  season INTEGER
);

CREATE TABLE IF NOT EXISTS drafterDrafts (
  drafter_id INTEGER,
  player_id INTEGER,
  FOREIGN KEY (drafter_id) REFERENCES drafters(id),
  FOREIGN KEY (player_id) REFERENCES players(id),
  PRIMARY KEY (drafter_id, player_id)
);