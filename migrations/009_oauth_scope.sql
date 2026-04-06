-- Store granted scope on OAuth codes and tokens for enforcement.
ALTER TABLE oauth_codes ADD COLUMN scope TEXT NOT NULL DEFAULT 'mcp';
ALTER TABLE oauth_tokens ADD COLUMN scope TEXT NOT NULL DEFAULT 'mcp';
