CREATE TABLE IF NOT EXISTS booking (
    id    INTEGER PRIMARY KEY NOT NULL,
    name  TEXT NOT NULL UNIQUE,
    resource_id INTEGER NOT NULL,
    start INTEGER NOT NULL,
    end   INTEGER,
    FOREIGN KEY (resource_id) REFERENCES resource(id),
    CHECK (start <= end or end ISNULL)
) STRICT
