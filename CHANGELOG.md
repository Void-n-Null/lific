# Changelog

## v1.1.0 — 2026-04-06

Security release closing 6 critical authentication and authorization vulnerabilities.

### Security Fixes

- **Privilege escalation via missing auth check** (LIF-56): `require_admin` and `require_project_lead` returned `Ok(())` when no user was associated with the request (OAuth tokens, legacy API keys). Any unauthenticated-but-authorized request had full admin privileges. Now default-deny.
- **OAuth PKCE bypass** (LIF-58): The `plain` PKCE method was accepted despite OAuth 2.1 requiring S256 only. Sending empty challenge/verifier with `method=plain` fully bypassed PKCE. Removed `plain`; reject empty values.
- **OAuth redirect_uri not validated at token exchange** (LIF-59): The `redirect_uri` from the token request was never compared against the one stored with the authorization code. An attacker who intercepted an auth code could exchange it from any URI. Now validated per OAuth 2.1 Section 4.1.3.
- **OAuth access tokens stored plaintext** (LIF-60): OAuth tokens were stored and looked up by raw value. A database leak exposed all active tokens. Now stored as SHA-256 hashes; raw token returned only once at issuance.
- **MCP identity confusion under concurrency** (LIF-61): A global `Mutex<Option<AuthUser>>` stored the current MCP user. Concurrent requests could overwrite each other's identity, and a panic would poison the mutex permanently. Replaced with serialized request handling via `tokio::sync::Mutex` with poison recovery.
- **Database errors leaked to clients** (LIF-62): Raw SQLite error messages (table names, column names, constraint details, file paths) were returned directly in API responses. Now returns generic "internal server error" and logs details server-side.

### Upgrade Notes

- **OAuth tokens are invalidated**: Existing plaintext OAuth tokens in the database will no longer validate since the lookup now expects SHA-256 hashes. Clients will need to re-authorize. This is intentional — plaintext tokens should not remain valid.
- No database migration required. No config changes.
