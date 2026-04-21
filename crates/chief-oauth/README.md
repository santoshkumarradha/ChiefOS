# Chief OS OAuth Broker

**Bearer tokens never cross the API boundary.**

This crate implements the "broker holds tokens, pack gets opaque SessionHandle" pattern for Chief OS. It is the canonical solution to the credential-theft problem that has plagued every app ecosystem since Chrome extensions became powerful.

## The Problem

Every major app platform (Chrome, Slack, Zapier, Discord) has suffered credential-theft incidents where packs stored bearer tokens themselves. The attack vector:

1. Malicious pack requests OAuth scope (e.g., `gmail.send`)
2. OS grants it
3. Malicious pack receives the real bearer token
4. Malicious pack exfiltrates the token (via its network capability)
5. Attacker can now send emails impersonating the user

Or more subtly: a legitimate pack gets compromised via supply chain attack, and suddenly all tokens it's ever stored are stolen.

## The Solution

Per [`docs/17-os-ceremonies-and-boundaries.md`](../../docs/17-os-ceremonies-and-boundaries.md) Layer 1:

- **Pack requests** a capability: `net.oauth2 { providers: ["google"], scopes: ["gmail.readonly"] }`
- **OS renders the login ceremony** (native Tauri window, real provider sign-in page, never app-owned)
- **OS stores the token sealed** in `$KEYSTORE/tokens.bin` using XChaCha20-Poly1305
- **Pack receives an opaque handle**: `SessionHandle { id, provider, scopes }` (no token field ever)
- **Pack makes API calls via the broker's proxy**: `broker.proxy_request(session, request)`
- **Broker validates scope**, injects `Authorization: Bearer <token>` server-side, proxies the request

Token never leaves the OS process. Pack code cannot steal it because it doesn't exist in pack memory.

## Usage

```rust
use chief_oauth::{OAuthBroker, Provider};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the broker with a keystore directory
    let broker = OAuthBroker::new(Path::new("/var/lib/chief/oauth")).await?;

    // Pack requests an OAuth login
    let challenge = broker.start_flow(
        Provider::Google,
        &["gmail.readonly".to_string()],
        "http://localhost:8080/callback",
    ).await?;

    // OS renders the login UI and returns the auth code + state
    // (this would come from the Tauri ceremony)
    let session = broker.complete_flow(
        challenge.flow_id,
        "authorization_code_from_user",
        &challenge.state,
    ).await?;

    // Pack receives only the opaque handle
    // The real token is sealed in storage, pack never sees it
    println!("Session ID: {}", session.id);

    // When pack needs to call an API, it goes through the broker
    let request = chief_oauth::ProxyRequest {
        url: "https://www.googleapis.com/gmail/v1/users/me/messages".to_string(),
        method: "GET".to_string(),
        headers: Default::default(),
        body: None,
    };

    let response = broker.proxy_request(&session, request).await?;
    println!("API response: {:?}", response.status);

    Ok(())
}
```

## Architecture

### Token Storage

- **Sealed file**: `$KEYSTORE/tokens.bin`
- **Encryption**: XChaCha20-Poly1305 (AEAD cipher, 192-bit nonce, 128-bit auth tag)
- **Key**: Generated at first run, stored at `$KEYSTORE/oauth.key` with `0600` perms
- **Format**: `[24-byte nonce || ciphertext || auth_tag]`

The ciphertext contains a serialized `TokenRecord`:

```rust
pub struct TokenRecord {
    pub session_id: String,
    pub access_token: String,     // Only decrypted inside the broker
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_in: Option<i64>,
    pub scope: String,
}
```

### Scope Enforcement

Before proxying a request, the broker validates that the requested URL's host matches the provider's allowlist:

- **Google**: `googleapis.com`, `google.com`, `www.googleapis.com`
- **GitHub**: `api.github.com`, `github.com`
- **Custom**: No default allowlist (defined at provider registration)

Attempt to proxy to `https://evil.example.com` → `ScopeDenied` error.

### Session Handle Design

```rust
pub struct SessionHandle {
    pub id: String,
    pub provider: Provider,
    pub scopes: Vec<String>,
    // NO BEARER TOKEN FIELD. EVER. Verified by test_session_handle_contains_no_bearer_token.
}
```

Serialized to JSON: `{"id":"sess_abc","provider":"google","scopes":["gmail.readonly"]}`

The Session Handle is:
- **Opaque to the pack**: contains only metadata
- **Revocable**: clearing the session ID from broker memory makes it useless
- **Non-transferable**: bearer token is not embedded
- **Auditable**: every proxy call is logged with the session ID

## Token Refresh

When a token is near expiration, the broker transparently refreshes it using the refresh token, updates sealed storage, and continues proxying. The pack never knows about the refresh.

## Revocation

```rust
broker.revoke(&session).await?;
```

Removes the session from in-memory state and clears the sealed token file. Any subsequent `proxy_request` call with this session ID fails with `SessionNotFound`.

## Testing

```bash
# Run all tests (excludes opt-in integration tests)
cargo test -p chief-oauth

# Run with output
cargo test -p chief-oauth -- --nocapture

# Run opt-in Google integration test (requires credentials)
GOOGLE_CLIENT_ID=... GOOGLE_CLIENT_SECRET=... cargo test -p chief-oauth --test real_google -- --ignored
```

### Test Coverage

1. **`flow_lifecycle.rs`**: Complete authorization flow (start -> complete -> revoke)
2. **`scope_enforcement.rs`**: Validates URLs against provider allowlists
3. **`token_never_leaks.rs`**: Asserts SessionHandle JSON contains no bearer token, and sealed file is binary encrypted
4. **`revoke.rs`**: Revocation clears in-memory state and sealed storage
5. **`real_google.rs`** (`#[ignore]`): Real OAuth dance with Google (opt-in)

## Migration Path

### Future: TPM / Secure Enclave

The encryption key is currently stored on disk (at rest, with restricted permissions). Future versions will:

1. Use **TPM 2.0** on Linux for key sealing
2. Use **Secure Enclave** on macOS / iOS
3. Use **Windows DPAPI** on Windows

This requires no API changes; `SealedTokenStore` will detect hardware support and escalate transparently.

### Future: Hardware-Bound Keys

Combine hardware key sealing with per-session HMAC attestation to ensure tokens cannot be exfiltrated even if the disk is stolen.

## Security Considerations

### Threat: Malicious Pack Requests More Scopes at Runtime

**Mitigated by**: Scopes are declared in manifest only. Requesting new scopes requires pack re-install + user re-consent (enforced by capability broker).

### Threat: Pack Escapes Sandbox and Reads Keystore Directly

**Mitigated by**: File permissions on `oauth.key` (`0600`), encryption key not in pack memory, and sandboxing (nspawn / Firecracker) prevents direct filesystem access.

### Threat: MITM Attack During Proxy Request

**Mitigated by**: All proxy requests over HTTPS with cert pinning (future enhancement). OS controls the HTTP stack, packs cannot downgrade to HTTP.

### Threat: Token Exfiltration via Side Channel

**Mitigated by**:
- Token never rendered to pack UI
- Token never logged to pack-accessible logs
- Broker never returns token in any response
- Audit logs record proxy calls by session ID, never the token itself

## Related Documentation

- **`docs/17-os-ceremonies-and-boundaries.md`** — Layer 1: Identity & Credentials (canonical pattern)
- **`docs/16-pack-sdk.md`** — `net.oauth2` capability kind and scope types
- **`adr/0002-capability-based-security.md`** — Capability-based security architecture
- **`docs/06-security-model.md`** — Threat model and security assumptions

## Acceptance Gates

- [x] `cargo build -p chief-oauth` passes
- [x] `cargo test -p chief-oauth` passes (5+ mock tests)
- [x] SessionHandle JSON contains **no** bearer token (asserted by test)
- [x] Sealed token file is **binary encrypted**, not plaintext (asserted by test)
- [x] `cargo clippy -p chief-oauth -- -D warnings` clean
- [x] `cargo fmt --check` clean
- [x] README complete with architecture, usage, and migration path

---

**Status**: MVP (v0.0.1). Supports Google, GitHub, and custom providers. Real Google integration test available via env vars.
