-- Bot ownership: link bot users to their human owner.
-- NULL for human users, references users(id) for bots.
ALTER TABLE users ADD COLUMN owner_id INTEGER REFERENCES users(id);
