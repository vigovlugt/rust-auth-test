CREATE TABLE IF NOT EXISTS user (
	id TEXT NOT NULL PRIMARY KEY,
	email TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS user_session (
	id TEXT NOT NULL PRIMARY KEY,
	expires_at TEXT NOT NULL,
	user_id TEXT NOT NULL REFERENCES user(id)
);

CREATE TABLE IF NOT EXISTS oauth_account (
	provider_id TEXT NOT NULL,
	provider_user_id TEXT NOT NULL,
	user_id TEXT NOT NULL,
	PRIMARY KEY (provider_id, provider_user_id),
	FOREIGN KEY (user_id) REFERENCES user(id)
)