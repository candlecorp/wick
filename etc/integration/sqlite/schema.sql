
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS num_types;

CREATE TABLE users (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  email TEXT NOT NULL
);

CREATE TABLE num_types (
  u8 INTEGER,
  i16 INTEGER,
  i32 INTEGER,
  i64 INTEGER,
  db_decimal DECIMAL,
  db_numeric NUMERIC,
  f32 REAL,
  f64 REAL
);

CREATE TABLE date_types (
  not_null_timestamp TIMESTAMP not null,
  null_timestamp TIMESTAMP,
  not_null_timestamptz TIMESTAMPTZ not null,
  null_timestamptz TIMESTAMPTZ
);

