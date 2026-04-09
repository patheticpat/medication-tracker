CREATE TABLE medications (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    unit TEXT NOT NULL,
    schedule_kind TEXT NOT NULL,
    schedule_amount REAL NOT NULL,
    schedule_day_of_week INTEGER,  -- nur für weekly, sonst NULL
    warning_threshold INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
