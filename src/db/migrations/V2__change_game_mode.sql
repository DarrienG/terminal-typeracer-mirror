ALTER TABLE passage_stats RENAME TO old_passage_stats;
CREATE TABLE game_mode(
   id PRIMARY KEY,
   description TEXT
);
INSERT INTO game_mode VALUES (0, "Default"), (1, "Instant Death");
CREATE TABLE passage_stats(
   row_id INTEGER PRIMARY KEY,
   passage TEXT,
   wpm INTEGER,
   accuracy REAL,
   highest_combo INTEGER,
   game_mode INTEGER,
   when_played_secs INTEGER,
   FOREIGN KEY(passage) REFERENCES passages(passage),
   FOREIGN KEY(game_mode) REFERENCES game_mode(id)
);
INSERT INTO passage_stats(
   row_id,
   passage,
   wpm,
   accuracy,
   highest_combo,
   game_mode,
   when_played_secs
) SELECT
   row_id,
   passage,
   wpm,
   accuracy,
   highest_combo,
   instant_death,
   when_played_secs
FROM old_passage_stats;
DROP TABLE old_passage_stats;
PRAGMA user_version = 2;
