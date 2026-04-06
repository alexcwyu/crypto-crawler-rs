# Development

> **See also**: [README](./README.md) · [Architecture](./architecture.md) · [Workflow](./workflow.md) · [State Management](./state-management.md)

## Prerequisites

| Tool | Minimum Version | Purpose |
|---|---|---|
| Rust toolchain | 1.70+ (2021 edition) | Build and test |
| Cargo | bundled with Rust | Dependency management |
| OpenSSL / native TLS | system package | TLS for WebSocket and REST |
| (optional) cargo-nextest | latest | Faster parallel test runner |

Install Rust via [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---

## Setup & Build

```bash
# Clone the repository
git clone https://github.com/crypto-crawler/crypto-crawler-rs.git
cd crypto-crawler-rs

# Build all crates in release mode
cargo build --release

# Build a single crate
cargo build -p crypto-crawler --release

# Check for compilation errors without producing binaries (fast)
cargo check --all
```

---

## Running Tests

Tests make live network calls to exchange APIs. Run selectively to avoid rate-limit issues.

```bash
# Run all tests (may be slow; hits real exchange APIs)
cargo test --all

# Run tests for a specific crate
cargo test -p crypto-crawler
cargo test -p crypto-ws-client
cargo test -p crypto-market-type

# Run a single test by name
cargo test -p crypto-crawler test_crawl_trade_binance

# Show stdout (useful for debugging live messages)
cargo test -- --nocapture

# Parallel execution with nextest
cargo nextest run -p crypto-crawler

# Enable logging during tests
RUST_LOG=debug cargo test -p crypto-ws-client -- --nocapture
```

---

## Project Structure

```
crypto-crawler-rs/
├── Cargo.toml                      # Workspace manifest (members list)
├── rustfmt.toml                    # Formatting config
├── README.md                       # Project overview
├── dependency-tree.svg             # Visual dependency graph
│
├── crypto-crawler/                 # Top-level crawl API crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                  # Public crawl_* functions
│       ├── msg.rs                  # Message struct
│       ├── utils/                  # REST_LOCKS, CMC rank helpers
│       └── crawlers/
│           ├── mod.rs              # Re-exports + dispatcher helpers
│           ├── utils.rs            # fetch_symbols_retry, crawl_event, crawl_snapshot
│           ├── binance.rs          # Binance-specific logic
│           ├── bitmex.rs
│           ├── deribit.rs
│           ├── huobi.rs
│           ├── kucoin.rs
│           ├── okx.rs
│           ├── zb.rs
│           └── zbg.rs
│
├── crypto-ws-client/               # WebSocket client crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                  # WSClient re-exports
│       ├── common/
│       │   ├── ws_client.rs        # WSClient trait
│       │   ├── ws_client_internal.rs  # Connection + run-loop
│       │   ├── message_handler.rs  # MessageHandler trait, MiscMessage
│       │   ├── connect_async.rs    # Async TCP/TLS connect
│       │   ├── command_translator.rs
│       │   └── utils.rs
│       └── clients/               # One file/dir per exchange
│           ├── binance.rs
│           ├── binance_option.rs
│           ├── bitmex.rs
│           ├── okx.rs
│           └── ...
│
├── crypto-rest-client/             # REST client crate
│   ├── Cargo.toml
│   └── src/
│
├── crypto-markets/                 # Symbol metadata crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── exchanges/             # One file per exchange
│
├── crypto-market-type/             # MarketType enum crate
│   ├── Cargo.toml
│   └── src/lib.rs
│
├── crypto-msg-type/                # MessageType enum + WS command builder
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── exchanges/             # Per-exchange WS command generators
│
└── crypto-client/                  # Order placement REST client
    ├── Cargo.toml
    └── src/
```

---

## Configuration Reference

All configuration is done via **environment variables** at runtime. There is no config file.

| Variable | Default | Description |
|---|---|---|
| `REST_RETRY_COUNT` | `5` | Number of retries in `fetch_symbols_retry` before giving up |
| `RUST_LOG` | unset | Log level — set to `debug`, `info`, `warn`, or `error` (uses `log` crate) |
| `HTTP_PROXY` / `HTTPS_PROXY` | unset | SOCKS5 or HTTP proxy for outbound REST requests (reqwest respects these) |
| `SOCKS5_PROXY` | unset | SOCKS5 proxy URL for WebSocket connections (fast-socks5) |

The per-exchange cooldown durations and maximum retry counts are hardcoded in [`crypto-crawler/src/crawlers/utils.rs`](../crypto-crawler/src/crawlers/utils.rs) — edit that file to tune them.

---

## Adding a New Exchange

1. **`crypto-market-type`** — add the exchange to `get_market_types(exchange)` in [`crypto-market-type/src/lib.rs`](../crypto-market-type/src/lib.rs).

2. **`crypto-markets`** — add a new file under `crypto-markets/src/exchanges/` implementing `fetch_symbols(market_type) -> Result<Vec<String>>`.

3. **`crypto-ws-client`** — add a new file under `crypto-ws-client/src/clients/` implementing the `WSClient` trait and `MessageHandler` trait. Export the new client in `crypto-ws-client/src/lib.rs`.

4. **`crypto-msg-type`** — add WS subscribe/unsubscribe command generation in `crypto-msg-type/src/exchanges/`.

5. **`crypto-rest-client`** — add REST endpoint handlers for snapshot and open interest if the exchange supports them.

6. **`crypto-crawler`** — add the exchange to the relevant `match exchange` arms in [`crypto-crawler/src/lib.rs`](../crypto-crawler/src/lib.rs). If the exchange needs special-casing, add a module under `crypto-crawler/src/crawlers/`.

7. Add the exchange to the cooldown table in `crawlers/utils.rs`.

---

## Code Quality

```bash
# Format all crates
cargo fmt --all

# Lint with Clippy
cargo clippy --all-targets --all-features -- -D warnings

# Check formatting without modifying files
cargo fmt --all -- --check
```

---

## Troubleshooting

### 1. Build fails: `error: failed to run custom build command for openssl-sys`

**Cause**: OpenSSL development headers not installed.

**Fix**:
```bash
# Debian / Ubuntu
sudo apt-get install pkg-config libssl-dev

# Arch / Manjaro
sudo pacman -S openssl

# macOS (Homebrew)
brew install openssl
export PKG_CONFIG_PATH="/opt/homebrew/opt/openssl/lib/pkgconfig"
```

---

### 2. Tests fail with `429 Too Many Requests`

**Cause**: Running too many tests in parallel against live exchange APIs.

**Fix**: Run tests serially or target a single crate:
```bash
cargo test -p crypto-markets -- --test-threads=1
REST_RETRY_COUNT=10 cargo test -p crypto-crawler
```

---

### 3. WebSocket connection drops immediately / `connection reset by peer`

**Cause**: Exchange may require correct `User-Agent`, or IP is blocked / geo-restricted.

**Fix**:
- Set `SOCKS5_PROXY` or `HTTP_PROXY` to route traffic through an allowed region.
- Verify the exchange's WebSocket URL has not changed (check the client file in `crypto-ws-client/src/clients/`).

---

### 4. `fetch_symbols_retry` returns an empty list

**Cause**: The exchange REST endpoint changed, or the exchange is down.

**Fix**:
- Set `RUST_LOG=debug` and rerun — check the logged URL and HTTP response.
- The function retries `REST_RETRY_COUNT` (default 5) times with exponential backoff; increase it for flaky networks:
  ```bash
  REST_RETRY_COUNT=10 cargo run
  ```

---

### 5. `crawl_funding_rate` panics with "does NOT have perpetual swap market"

**Cause**: The exchange or market type does not support perpetual swap funding rates.

**Fix**: Only call `crawl_funding_rate` with exchanges that have swap markets: `binance`, `bitmex`, `huobi`, `okx`. Check [`crypto-crawler/src/lib.rs`](../crypto-crawler/src/lib.rs) for the full match arm.

---

### 6. Symbol validation panics: "Invalid symbols: …"

**Cause**: The symbols passed to a crawl function are not listed by the exchange for that market type.

**Fix**: Pass `None` as the symbols argument to auto-discover all valid symbols, or call `fetch_symbols_retry(exchange, market_type)` first to get the current list:
```rust
let symbols = crypto_crawler::fetch_symbols_retry("binance", MarketType::Spot);
println!("{:?}", symbols);
```

---

### 7. High CPU usage in `run()` loop

**Cause**: A pathological `handle_message` returning `MiscMessage::Other` on every frame without sleeping — or a misconfigured exchange sending excessively frequent heartbeats.

**Fix**: Enable debug logging and inspect frame rates. Reduce subscription scope by providing an explicit, smaller symbol list.

---

## Security Considerations

- **No credentials required** — all crawled data is from **public, unauthenticated** WebSocket and REST endpoints. `crypto-client` (order placement) is the only crate that would require API keys, and it is not used by the crawler.
- **TLS by default** — `tokio-tungstenite` is compiled with `rustls-tls-native-roots`, using system root CAs.
- **No data persistence** — the library writes nothing to disk except the `fslock` lock files in the OS temp directory. These contain no sensitive data.
- **Proxy support** — if routing through a proxy, ensure the proxy endpoint is trusted; raw market data passes through unencrypted if a plain-HTTP proxy is used for REST.
- **Rate-limit locks** — the `fslock` files are world-readable by default. In multi-user environments, ensure the temp directory permissions are appropriate.

---

## See Also

- [`Cargo.toml`](../Cargo.toml) — workspace manifest
- [`crypto-crawler/Cargo.toml`](../crypto-crawler/Cargo.toml) — crawler crate dependencies
- [`crypto-ws-client/Cargo.toml`](../crypto-ws-client/Cargo.toml) — WebSocket client dependencies
- [Architecture](./architecture.md) — crate roles and dependency graph
- [Workflow](./workflow.md) — crawling pipeline and rate-limit details
