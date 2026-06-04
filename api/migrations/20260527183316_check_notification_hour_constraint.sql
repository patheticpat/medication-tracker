PRAGMA defer_foreign_keys = ON;

CREATE TABLE user_notification_settings_new (
    user_id TEXT PRIMARY KEY,
    timezone TEXT NOT NULL,
    notification_hour INTEGER NOT NULL DEFAULT 8 CHECK (notification_hour BETWEEN 0 AND 23),
    notification_days TEXT NOT NULL DEFAULT '0,1,2,3,4,5,6',
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

INSERT INTO user_notification_settings_new
    SELECT user_id, timezone, notification_hour, notification_days
    FROM user_notification_settings
    WHERE notification_hour BETWEEN 0 AND 23;

DROP TABLE user_notification_settings;

ALTER TABLE user_notification_settings_new RENAME TO user_notification_settings;
