# crypto-crawler-rs: development workflow and layout

Use branch `feature/ayu_develop`. The actual upstream default branch and all fetched branch/tag identities are recorded in the maintenance sync evidence. Merge existing work; do not reset divergent branches or replace conflicting tags.

## Source synchronization

```sh
git fetch origin
git fetch upstream
git switch feature/ayu_develop
git merge origin/feature/ayu_develop
# Merge the actual default branch documented in the sync result.
```

This maintenance used namespaced remote tags before comparing tag identities. Missing upstream tags/branches were pushed to the fork; divergent branches were merged when conflict-free, and conflicting identities were retained for explicit reconciliation. The feature branch contains the default branch plus the maintenance changes. Source snapshot and architecture are documented in [ayu-develop.md](ayu-develop.md).

## Repository layout

- `docs/ayu-develop.md` and `docs/ayu-develop.html`: engineering guide with architecture, workflow, class and sequence diagrams and implementation links.
- `docs/development.md`: toolchain, layout and contributor workflow.
- `docs/validation.md`: exact executed commands, outcomes and limitations.

## Native Rust toolchain

Rust stable1.99.0 is installed through rustup. Compile all applicable targets and run the existing test suite. Optional Python features require the validated managed interpreter through `PYO3_PYTHON`. Source locks are checked in where appropriate for application/workspace reproducibility; build outputs remain ignored.

## Verification and documentation

Use the commands and results in [validation.md](validation.md). Markdown is the editable source of the guide; HTML embeds rendered SVG diagrams for offline browsing. Keep actual source links valid after moving modules. Preserve existing cases/assertions while correcting incompatible APIs, genuine resource leaks or invalid fixtures. No additional test cases were requested or added.

## Historical reference

The [previous development guide](archive/development-before-maintenance.md) is retained for source history; use current validation commands for this branch.


## Current public protocol and runtime repairs

The branch builds all seven local native members through workspace paths, so checks exercise the maintained source instead of older published duplicates. Binance linear futures route depth/book-ticker channels to the public endpoint and trade/ticker/candlestick channels to the market endpoint; explicit endpoint overrides remain respected.

Bitget classic v2 uses its current public WebSocket endpoint and typed SPOT/USDT-FUTURES/USDC-FUTURES/COIN-FUTURES subscriptions. REST metadata and order books use v2 endpoints. Legacy admitted symbol suffixes are retained at the public boundary and removed only for the corresponding HTTP/WebSocket wire IDs; margin-product identity is retained. Three supplied wire-command cases and the existing linear-symbol classification case passed. All eight existing linear crawling cases passed together after applying the documented fewer-than-50-channel connection recommendation through the existing complete-symbol chunking path. Current metadata reports no COIN-FUTURES contracts. The dated live-venue evidence is retained in the workspace report.

Synchronous message conversion and snapshot loops own OS threads instead of occupying Tokio blocking workers indefinitely at runtime shutdown. Existing streaming test receivers use bounded waits while retaining venue, market-type, message-type and parse assertions. Metadata/order-book HTTP clients use ten-second connection and thirty-second total request bounds so dead services cannot hold a synchronous request forever.

## Historical live integration constraints

The supplied tests include expired 2022 contracts and retired venues. [BitMEX's official September 2026 notice](https://www.bitmex.com/blog/reduce-only-september-site/) retires the original XBTUSD/ETHUSD fixtures; its current instrument API returns XBTUSD as Settled and its live order book is empty. Such live assertions remain failures rather than being replaced by historical replay or a different contract type. The [Bitget current API contract](https://www.bitget.com/docs/catalog/classic-contract-market/classic-contract-market) documents the v2 schema.

Use [validation](validation.md) and [portable executed records](maintenance/index.html) to distinguish native build/unit success from each external venue run. Raw exploratory timeout logs remain in the workspace evidence. The [previous development guide](archive/development-before-maintenance.md) is retained as historical documentation.

Current Coinbase public order-book subscriptions use the documented [`level2_batch` channel](https://docs.cdp.coinbase.com/exchange/websocket-feed/channels), retaining the snapshot/l2update schema with 50 ms batching. The existing low-level command translator still accepts explicit channel names. Full/Level3 access remains distinct and is not replaced by Level2 data. All ten original crawler documentation examples retain their non-empty-message assertions with a 30-second receive deadline.

The current Deribit target passes all 24 existing cases (one pre-existing ignored case). Expired futures select the nearest listed expiry for the same underlying/settlement family. Option fixtures preserve the underlying and call/put side, prefer the original strike and select the closest listed strike if it is no longer admitted; all original venue/type/message/parser assertions remain. OKX candlesticks route to its documented [business endpoint](https://www.okx.com/docs-v5/en/) while market data retains the public endpoint and explicit URL overrides remain single-connection. The licensed current parser patch maps a genuinely absent optional next-period funding forecast to `None`; it never fabricates a zero rate. Full upstream licenses and modification notices are included under `vendor/`.

## Final verification scope

The final all-target build and all 96 native unit cases pass. Live integration checks are distinct: all 86 existing targets were attempted, with external failures and four target timeouts recorded. The final documentation run passes 12 examples and fails the original Coinbase authenticated full/level3 example. See [validation.md](validation.md) for complete and subsequent focused results.

Binance metadata classifies both regular and TradFi perpetual contracts as swaps. Order increments are actual venue price/lot filters. Current quote assets are admitted from the returned metadata rather than a frozen USDT/BUSD list. The original Binance metadata target now passes all 14 active cases; two existing ignored option cases remain unchanged.

## Bybit current instrument metadata

The [native Bybit metadata adapter](../crypto-markets/src/exchanges/bybit.rs) uses v5 category pagination and explicit contract types. It admits actual base/quote/settlement IDs and delivery timestamps; it does not infer maturity from the local calendar. [The licensed pair patch](../vendor/crypto-pair/src/exchanges/bybit.rs) handles current USDC PERP symbols. All ten supplied metadata cases pass. Public VIP0 fee baselines are labeled in `info.feeSource`; authenticated account-specific fees are outside this public metadata API.
