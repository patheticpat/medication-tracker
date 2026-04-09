CREATE TABLE log_entries (
    id TEXT PRIMARY KEY,
    medication_id TEXT NOT NULL,
    kind TEXT NOT NULL,  -- 'baseline' oder 'refill'
    amount REAL NOT NULL,
    date TEXT NOT NULL,
    note TEXT,
    FOREIGN KEY (medication_id) REFERENCES medications(id) ON DELETE CASCADE
);

