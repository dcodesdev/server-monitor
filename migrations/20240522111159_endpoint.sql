-- Create the endpoint table
CREATE TABLE endpoint (
  id TEXT PRIMARY KEY NOT NULL DEFAULT (lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)), 2) || '-' || substr('89ab', abs(random() % 4) + 1, 1) || substr(hex(randomblob(2)), 2) || '-' || hex(randomblob(6)))),
  url VARCHAR NOT NULL UNIQUE,
  status TEXT NOT NULL CHECK (status IN ('UP', 'DOWN', 'PENDING')),
  uptime_at TIMESTAMP,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
