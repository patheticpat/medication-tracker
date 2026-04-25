CREATE TABLE credentials (
    credential_id TEXT PRIMARY KEY,  -- Base64-kodierte Credential ID
    user_id TEXT NOT NULL,
    passkey TEXT NOT NULL,  -- serialisierter passkey
    display_name TEXT,  -- optional display name
    counter INTEGER DEFAULT 0,
    added_at INTEGER NOT NULL,
    last_used_at INTEGER,
    FOREIGN KEY(user_id) REFERENCES users(id)
);
