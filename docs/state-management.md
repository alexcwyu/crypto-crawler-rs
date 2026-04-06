# State Management

> **See also**: [README](./README.md) · [Architecture](./architecture.md) · [Workflow](./workflow.md) · [Development](./development.md)

## Overview

`crypto-crawler-rs` is a stateless data pipeline at its core. There is no persistent application state, no database, and no in-memory order book maintained by the library. State is limited to:

1. **Connection state** — managed inside `WSClientInternal` (connected / reconnecting).
2. **Symbol list cache** — held in the `crawl_event` / `crawl_snapshot` functions; refreshed hourly.
3. **Rate-limit locks** — `REST_LOCKS` and `WS_LOCKS` maps of `fslock::LockFile` instances.
4. **Message envelope** — the `Message` struct is the only data model crossing the library boundary.

---

## WebSocket Connection State Machine

```mermaid
stateDiagram-v2
    [*] --> Connecting : WSClientInternal::connect()

    Connecting --> Connected : TCP/TLS handshake OK
    Connecting --> Connecting : connect error → retry

    Connected --> Subscribed : subscribe_trade / subscribe_orderbook / …
    Subscribed --> Running : ws_client.run()

    Running --> Running : Normal market data frame → tx.send()
    Running --> Running : Ping from server → send Pong
    Running --> Running : Heartbeat timer → send Ping
    Running --> Reconnecting : error / Reconnect message
    Running --> Closed : close() called

    Reconnecting --> Connecting : reconnect attempt
    Closed --> [*]
```

Source: [`crypto-ws-client/src/common/ws_client_internal.rs`](../crypto-ws-client/src/common/ws_client_internal.rs)

---

## Core Data Model: `Message`

**Source**: [`crypto-crawler/src/msg.rs`](../crypto-crawler/src/msg.rs)

The single output type of every crawler function:

```rust
pub struct Message {
    /// Exchange name (e.g. "binance", "okx")
    pub exchange: String,
    /// Market type (Spot, LinearSwap, InverseSwap, …)
    pub market_type: MarketType,
    /// Message category (Trade, L2Event, BBO, Ticker, …)
    pub msg_type: MessageType,
    /// Trading symbol in exchange-native format (e.g. "BTCUSDT")
    pub symbol: Option<String>,
    /// Unix timestamp (milliseconds) when the message was received
    pub received_at: u64,
    /// Original unmodified JSON string from the exchange
    pub json: String,
}
```

Serialisation:
- `Display` / `serde_json::to_string` → full JSON object
- `to_tsv_string()` → `"{received_at}\t{symbol}\t{json}"` (compact, omits exchange/market/type)
- `from_tsv_string(exchange, market_type, msg_type, s)` → reconstruct from TSV

---

## `MarketType` Enum

**Source**: [`crypto-market-type/src/lib.rs`](../crypto-market-type/src/lib.rs)

```rust
pub enum MarketType {
    Unknown,
    Spot,
    LinearFuture,
    InverseFuture,
    LinearSwap,
    InverseSwap,
    AmericanOption,
    EuropeanOption,
    QuantoFuture,
    QuantoSwap,
    Move,
    BVOL,
}
```

| Variant | Margin | Settlement | Description |
|---|---|---|---|
| `Spot` | Optional | Immediate | Cash market |
| `LinearFuture` | Yes | USDT | USDT-margined dated futures |
| `InverseFuture` | Yes | Coin | Coin-margined dated futures |
| `LinearSwap` | Yes | USDT | USDT-perpetual (no expiry) |
| `InverseSwap` | Yes | Coin | Coin-perpetual (no expiry) |
| `EuropeanOption` | Yes | Coin | Vanilla European options |
| `AmericanOption` | Yes | Coin | Vanilla American options |
| `QuantoFuture` | Yes | Mixed | Quanto-settled futures |
| `QuantoSwap` | Yes | Mixed | Quanto-settled perpetuals |
| `Move` | Yes | USDT | FTX MOVE contracts |
| `BVOL` | Yes | USDT | FTX volatility tokens |

Helper: `get_market_types(exchange) -> Vec<MarketType>` returns the market types an exchange supports.

---

## `MessageType` Enum

**Source**: [`crypto-msg-type/src/lib.rs`](../crypto-msg-type/src/lib.rs)

```rust
pub enum MessageType {
    Other,
    Trade,
    L2Event,
    L2Snapshot,
    L2TopK,
    L3Event,
    L3Snapshot,
    BBO,
    Ticker,
    Candlestick,
    FundingRate,
    OpenInterest,
    LongShortRatio,
    TakerVolume,
}
```

| Variant | Transport | Aggregation | Frequency | Description |
|---|---|---|---|---|
| `Trade` | WebSocket | N/A | Tick | Individual matched trades |
| `L2Event` | WebSocket | By price level | Tick (incremental) | Level-2 orderbook delta |
| `L2Snapshot` | REST | By price level | Poll | Full Level-2 orderbook |
| `L2TopK` | WebSocket | By price level | Interval | Top-K levels snapshot |
| `L3Event` | WebSocket | Raw (per order) | Tick (incremental) | Level-3 orderbook delta |
| `L3Snapshot` | REST | Raw (per order) | Poll | Full Level-3 orderbook |
| `BBO` | WebSocket | N/A | Tick | Best Bid & Offer (Level-1) |
| `Ticker` | WebSocket | N/A | 24 hr window | Rolling OHLCV + volume ticker |
| `Candlestick` | WebSocket | N/A | Configurable interval | OHLCV candlestick bars |
| `FundingRate` | WebSocket | N/A | Per funding period | Perpetual swap funding rate |
| `OpenInterest` | REST | N/A | Poll | Total open interest |
| `LongShortRatio` | REST/WS | N/A | Interval | Long vs short position ratio |
| `TakerVolume` | REST/WS | N/A | Interval | Taker buy / sell volume |

---

## `MiscMessage` Enum (Internal)

**Source**: [`crypto-ws-client/src/common/message_handler.rs`](../crypto-ws-client/src/common/message_handler.rs)

Used internally by the WebSocket run-loop to classify each incoming frame before acting on it:

```rust
pub(crate) enum MiscMessage {
    Normal,             // Forward raw JSON to consumer
    Mutated(String),    // Forward handler-transformed JSON
    WebSocket(Message), // Send this frame back to exchange
    Pong,               // Server acknowledged our ping
    Reconnect,          // Exchange wants us to reconnect
    Other,              // Subscription acks etc. — discard
}
```

---

## Rate-Limit State

**Source**: [`crypto-crawler/src/crawlers/utils.rs`](../crypto-crawler/src/crawlers/utils.rs)

`REST_LOCKS` and `WS_LOCKS` are `once_cell::sync::Lazy` maps:

```
HashMap<exchange_name, HashMap<MarketType, Arc<Mutex<fslock::LockFile>>>>
```

- Each `LockFile` is a file on disk (e.g., `/tmp/crypto_crawler_binance_spot.lock`).
- Processes sharing the same exchange+market combination serialise REST requests through this file lock, preventing concurrent quota exhaustion across OS processes.

---

## See Also

- [`crypto-crawler/src/msg.rs`](../crypto-crawler/src/msg.rs) — `Message` struct implementation
- [`crypto-market-type/src/lib.rs`](../crypto-market-type/src/lib.rs) — `MarketType` + `get_market_types`
- [`crypto-msg-type/src/lib.rs`](../crypto-msg-type/src/lib.rs) — `MessageType` + `get_ws_commands`
- [`crypto-ws-client/src/common/message_handler.rs`](../crypto-ws-client/src/common/message_handler.rs) — `MiscMessage`
- [Workflow](./workflow.md) — how these types flow through the pipeline
