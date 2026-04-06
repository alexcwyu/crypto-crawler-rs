# Crypto-Crawler-rs

> **Last Updated**: 2026-04-07T00:00:00Z
> **Git Hash**: `8091c2c`

A rock-solid Rust library for crawling real-time and historical market data from cryptocurrency exchanges. Built as a Cargo workspace of composable crates, it provides unified WebSocket and REST APIs that normalize exchange-specific protocols into a single `Message` type.

---

## Key Features

| Feature | Detail |
|---|---|
| **Language** | Rust (2021 edition), async via Tokio |
| **Transport** | WebSocket (tokio-tungstenite) + HTTP REST (reqwest) |
| **Data Types** | Trades, BBO, L2/L3 orderbooks, Tickers, Candlesticks, Funding rates, Open interest |
| **Exchanges** | Binance, BitMEX, Bitfinex, Bitget, Bithumb, Bitstamp, Bybit, CoinbasePro, Deribit, dYdX, FTX, Gate, Huobi, Kraken, KuCoin, MEXC, OKX, ZB, ZBG, and more |
| **Market Types** | Spot, LinearFuture, InverseFuture, LinearSwap, InverseSwap, EuropeanOption, QuantoFuture, QuantoSwap |
| **Delivery** | `std::sync::mpsc::Sender<Message>` — channel-based, works with any consumer |
| **Raw Preservation** | Messages are forwarded unparsed; use `crypto-msg-parser` to decode |
| **Multi-language FFI** | C-style FFI from `crypto-crawler` enables Python and C++ bindings |
| **Rate-limit Safety** | Per-exchange cooldowns and file-based distributed locks prevent 429 errors |
| **Symbol Discovery** | Auto-discovers trading symbols every hour; supports explicit symbol lists |
| **License** | Apache-2.0 |

---

## Quick Start

Add `crypto-crawler` to your `Cargo.toml`:

```toml
[dependencies]
crypto-crawler = "4.7.9"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

### Crawl real-time trades

```rust
use crypto_crawler::{crawl_trade, MarketType};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let (tx, rx) = std::sync::mpsc::channel();
    tokio::task::spawn(async move {
        // Crawl all BTC/ETH trades on Binance COIN-margined perpetuals
        crawl_trade("binance", MarketType::InverseSwap, None, tx).await;
    });

    for msg in rx {
        println!("{}", msg); // JSON-serialised Message
        break;
    }
}
```

### Crawl level-2 orderbook snapshots via REST

```rust
use crypto_crawler::{crawl_l2_snapshot, MarketType};

let (tx, rx) = std::sync::mpsc::channel();
std::thread::spawn(move || {
    crawl_l2_snapshot("binance", MarketType::InverseSwap, None, tx);
});

for msg in rx {
    println!("{}", msg);
    break;
}
```

### Subscribe a single symbol (parsed output)

```rust
use crypto_crawler::{subscribe_symbol, MarketType, MessageType};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    subscribe_symbol(
        "binance",
        MarketType::Spot,
        "BTCUSDT",
        &[MessageType::Trade, MessageType::BBO],
        tx,
    ).await;
}
```

---

## Architecture Summary

The project is a **Cargo workspace** of seven independent crates with a strict layered dependency order:

```
crypto-crawler          ← top-level API (crawl_trade, crawl_l2_event, …)
├── crypto-ws-client    ← WebSocket connections per exchange
├── crypto-rest-client  ← REST polling (snapshots, open interest)
├── crypto-markets      ← symbol / market metadata via REST
├── crypto-market-type  ← MarketType enum (Spot, LinearSwap, …)
├── crypto-msg-type     ← MessageType enum + WS command builder
└── crypto-client       ← order-placement REST client (separate concern)
```

Parsing of raw JSON is intentionally **out of scope** — use the companion library [crypto-msg-parser](https://github.com/crypto-crawler/crypto-msg-parser). The CLI tool [carbonbot](https://github.com/crypto-crawler/carbonbot) wraps `crypto-crawler` for production deployments.

---

## Documentation Index

| Document | Description |
|---|---|
| [architecture.md](./architecture.md) | Crate dependency graph, component roles, module layout |
| [workflow.md](./workflow.md) | Crawling pipeline, WebSocket lifecycle, data flow sequence |
| [state-management.md](./state-management.md) | Message and data models, MarketType / MessageType enums |
| [development.md](./development.md) | Setup, build, config reference, troubleshooting |

---

## Links

- **GitHub**: https://github.com/crypto-crawler/crypto-crawler-rs
- **crates.io** (`crypto-crawler`): https://crates.io/crates/crypto-crawler
- **docs.rs**: https://docs.rs/crypto-crawler
- **carbonbot CLI**: https://github.com/crypto-crawler/carbonbot
- **Message parser**: https://github.com/crypto-crawler/crypto-msg-parser
- **Discord**: https://discord.gg/Vych8DNZU2

---

## Tags

`rust` · `async` · `tokio` · `websocket` · `cryptocurrency` · `market-data` · `orderbook` · `binance` · `bitmex` · `okx` · `crawling` · `data-pipeline` · `real-time` · `trading` · `cargo-workspace`
