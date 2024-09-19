CREATE TABLE IF NOT EXISTS players (
  name TEXT PRIMARY KEY,
  season INTEGER,
  voted_out DATE
);

CREATE TABLE IF NOT EXISTS drafters (
  name TEXT PRIMARY KEY,
  season INTEGER
);

CREATE TABLE IF NOT EXISTS drafterDrafts (
  drafter_id INTEGER,
  player_id INTEGER,
  FOREIGN KEY (drafter_id) REFERENCES drafters(name),
  FOREIGN KEY (player_id) REFERENCES players(name),
  PRIMARY KEY (drafter_id, player_id)
);