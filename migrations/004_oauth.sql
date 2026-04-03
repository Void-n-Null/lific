-- OAuth 2.1 support for Claude.ai and other MCP clients.

CREATE TABLE IF NOT EXISTS oauth_clients (
    client_id       TEXT PRIMARY KEY,
    client_name     TEXT NOT NULL,
    redirect_uris   TEXT NOT NULL,  -- JSON array
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS oauth_codes (
    code            TEXT PRIMARY KEY,
    client_id       TEXT NOT NULL REFERENCES oauth_clients(client_id),
    redirect_uri    TEXT NOT NULL,
    code_challenge  TEXT NOT NULL,
    code_challenge_method TEXT NOT NULL DEFAULT 'S256',
    expires_at      TEXT NOT NULL,
    used            INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS oauth_tokens (
    access_token    TEXT PRIMARY KEY,
    client_id       TEXT NOT NULL REFERENCES oauth_clients(client_id),
    expires_at      TEXT NOT NULL,
    revoked         INTEGER NOT NULL DEFAULT 0
);
