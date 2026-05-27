CREATE TABLE user_notification_settings (
    user_id TEXT PRIMARY KEY,
    timezone TEXT NOT NULL,
    notification_hour INTEGER NOT NULL DEFAULT 8,
    notification_days TEXT NOT NULL DEFAULT '0,1,2,3,4,5,6',
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
