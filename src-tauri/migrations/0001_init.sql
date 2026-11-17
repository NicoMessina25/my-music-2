CREATE TABLE artist (
  id          INTEGER PRIMARY KEY,
  name        TEXT NOT NULL,
  archived_at TEXT
);
CREATE UNIQUE INDEX ux_artist_name ON artist(lower(name));

CREATE TABLE song (
  id           INTEGER PRIMARY KEY,
  title        TEXT NOT NULL,
  artist_id    INTEGER NOT NULL REFERENCES artist(id),
  album        TEXT,
  release_date TEXT,              -- 'YYYY' | 'YYYY-MM' | 'YYYY-MM-DD', NULL si se desconoce
  duration_ms  INTEGER,
  notes        TEXT,
  created_at   TEXT NOT NULL DEFAULT (datetime('now')),
  archived_at  TEXT
);
CREATE UNIQUE INDEX ux_song ON song(lower(title), artist_id);

CREATE TABLE playlist (
  id          INTEGER PRIMARY KEY,
  name        TEXT NOT NULL,
  created_at  TEXT NOT NULL DEFAULT (datetime('now')),
  archived_at TEXT
);

CREATE TABLE playlist_song (
  playlist_id INTEGER NOT NULL REFERENCES playlist(id),
  song_id     INTEGER NOT NULL REFERENCES song(id),
  added_at    TEXT NOT NULL DEFAULT (datetime('now')),
  PRIMARY KEY (playlist_id, song_id)
);

CREATE TABLE source (
  id   INTEGER PRIMARY KEY,
  code TEXT NOT NULL UNIQUE,      -- clave estable usada por el código
  name TEXT NOT NULL,
  url  TEXT
);
INSERT INTO source (code, name, url) VALUES
  ('manual',  'Carga manual', NULL),
  ('spotify', 'Spotify',      'https://open.spotify.com');

CREATE TABLE import (
  id          INTEGER PRIMARY KEY,
  source_id   INTEGER NOT NULL REFERENCES source(id),
  file_name   TEXT,
  imported_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE external_ref (
  source_id            INTEGER NOT NULL REFERENCES source(id),
  external_id          TEXT NOT NULL,
  song_id              INTEGER NOT NULL REFERENCES song(id),
  first_seen_import_id INTEGER REFERENCES import(id),
  last_seen_import_id  INTEGER REFERENCES import(id),
  PRIMARY KEY (source_id, external_id)
);
CREATE INDEX ix_external_ref_song ON external_ref(song_id);