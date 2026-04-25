CREATE TABLE passkey_challenges (
    user_id TEXT PRIMARY KEY,
    challenge TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    FOREIGN KEY(user_id) REFERENCES users(id)
);
