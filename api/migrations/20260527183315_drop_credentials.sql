-- Daten sichern
CREATE TABLE users_new (
    id        TEXT PRIMARY KEY,
    auth0_sub TEXT UNIQUE
);

CREATE TABLE medications_new (
    id                 TEXT PRIMARY KEY,
    user_id            TEXT NOT NULL,
    name               TEXT NOT NULL,
    unit               TEXT NOT NULL,
    schedule_kind      TEXT NOT NULL,
    schedule_amount    REAL NOT NULL,
    schedule_day_of_week INTEGER,
    warning_threshold  INTEGER NOT NULL,
    snoozed            BOOLEAN NOT NULL DEFAULT FALSE,
    FOREIGN KEY (user_id) REFERENCES users_new(id)
);

CREATE TABLE log_entries_new (
    id            TEXT PRIMARY KEY,
    medication_id TEXT NOT NULL,
    kind          TEXT NOT NULL,
    amount        REAL NOT NULL,
    date          TEXT NOT NULL,
    note          TEXT,
    FOREIGN KEY (medication_id) REFERENCES medications_new(id) ON DELETE CASCADE
);

CREATE TABLE push_subscriptions_new (
    id         TEXT PRIMARY KEY,
    user_id    TEXT NOT NULL,
    endpoint   TEXT NOT NULL UNIQUE,
    p256dh     TEXT NOT NULL,
    auth       TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users_new(id) ON DELETE CASCADE
);

CREATE TABLE user_notification_settings_new (
    user_id            TEXT PRIMARY KEY,
    timezone           TEXT NOT NULL,
    notification_hour  INTEGER NOT NULL DEFAULT 8,
    notification_days  TEXT NOT NULL DEFAULT '0,1,2,3,4,5,6',
    FOREIGN KEY (user_id) REFERENCES users_new(id) ON DELETE CASCADE
);

-- Daten rüberretten
INSERT INTO users_new (id) SELECT id FROM users;
INSERT INTO medications_new SELECT * FROM medications;
INSERT INTO log_entries_new SELECT * FROM log_entries;
INSERT INTO push_subscriptions_new SELECT * FROM push_subscriptions;
INSERT INTO user_notification_settings_new SELECT * FROM user_notification_settings;

-- Alte Tabellen löschen (inkl. credentials und passkey_challenges)
DROP TABLE credentials;
DROP TABLE passkey_challenges;
DROP TABLE user_notification_settings;
DROP TABLE push_subscriptions;
DROP TABLE log_entries;
DROP TABLE medications;
DROP TABLE users;

-- Umbenennen
ALTER TABLE users_new RENAME TO users;
ALTER TABLE medications_new RENAME TO medications;
ALTER TABLE log_entries_new RENAME TO log_entries;
ALTER TABLE push_subscriptions_new RENAME TO push_subscriptions;
ALTER TABLE user_notification_settings_new RENAME TO user_notification_settings;
