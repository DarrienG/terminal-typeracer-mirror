CREATE TABLE IF NOT EXISTS passages (
   passage TEXT PRIMARY KEY,
   passage_len INTEGER
);
CREATE TABLE IF NOT EXISTS passage_stats (
   row_id INTEGER PRIMARY KEY,
   passage TEXT,
   wpm INTEGER,
   accuracy REAL,
   highest_combo INTEGER,
   instant_death INTEGER,
   when_played_secs INTEGER,
   FOREIGN KEY(passage) REFERENCES passage(passage)
);
DROP TABLE IF EXISTS schema_info;
PRAGMA user_version = 1;
