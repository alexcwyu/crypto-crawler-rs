# Crypto crawler: validation results

All workspace targets compile with the final maintained fixtures and licensed protocol patches. The complete native unit suite passes: **96 passed, 0 failed**. The final complete documentation run has **12 passed, 1 failed**; the failing original Coinbase full/level3 example requires authentication on the current venue.

## Complete existing integration inventory

All **86 supplied integration targets** were attempted. The inventory's completed targets contain **481 passed, 416 failed, 112 pre-existing ignored cases**; four targets timed out (crawler FTX and websocket Binance, FTX and Gate). The coordinator completed, but this is not an all-passing test suite. Its commands predate some subsequent protocol repairs; later checks below validate their own exact scope rather than being added together as a fictional single run.

No new cases were added or original cases disabled. Retired venues, missing authentication, missing currently-listed contract families and original live-arrival deadlines remain observable failures. Original received-message, market-family, option-side and parser assertions are retained. Network requests and receiver waits are bounded; no artificial market events or substitute venues are used.

## Binance metadata follow-up

The complete existing Binance metadata target now passes **14 active cases with 2 original ignored cases**. Actual `TRADIFI_PERPETUAL` records are classified as swaps, not dated futures. Both linear and inverse order increments come from `PRICE_FILTER.tickSize` and `LOT_SIZE.stepSize`; display precision is not an order increment. The exact BTCUSDT tick assertion remains numeric and now matches the current real `0.10` filter. The original symbol test compares current quote metadata and retains every market-type assertion.

The complete 96-case native unit suite was rerun successfully after this repair. [Captured public metadata](maintenance/binance-classification-evidence.json) and [Binance exchange information](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Exchange-Information) document the source of this change.

## Bybit v5 metadata follow-up

The deprecated `/v2/public/symbols` returns HTTP404. The real current [v5 instrument API](https://bybit-exchange.github.io/docs/v5/market/instrument) returns both categories, with cursor pagination and explicit contract types. All **10 original Bybit metadata cases pass** against that API. Delivery timestamps and price/quantity limits come from the real response. Existing exact BTC limit assertions were updated only to match the captured current metadata; no test cases or contract-family assertions were removed.

[Captured public metadata](maintenance/bybit-v5-public-metadata.json) records the BTC instruments used by those assertions. Current USDC perpetual `PERP` symbols retain their base/USDC pair and linear swap identity in the licensed pair patch. This result does not claim completion of Bybit's separate deprecated REST trading or websocket APIs.

V5 public instruments do not expose authenticated account fees. Returned fees are explicitly labeled published VIP0 baselines in `info.feeSource`: [ordinary derivatives](https://www.bybit.com/en/help-center/article/Trading-Fee-Structure) and [TradFi perpetual rates](https://announcements.bybit.com/en/article/tradfi-perpetuals-lower-fees-across-all-tiers-bltb196506dada4be39/). They are estimates; authenticated account rates can differ.

## Subsequent current-protocol results

- Binance REST inverse futures: 6 passed; linear futures: 6 passed, using real currently-listed quarterly metadata.
- Binance crawler final whole target: 44 passed, 3 failed on original inverse trade/ticker arrival deadlines. A narrower nine-case quarterly rerun passed; it does not replace the whole-target result.
- Deribit crawler: all 24 existing active cases passed; one original case remains ignored.
- Deribit websocket final target: 12 passed, 4 failed, 6 original ignored; real option ticker/BBO/candle and inverse candle delivery failed the original live coverage requirement.
- Bitget linear v2 target: all 8 existing cases passed after limiting each real websocket subscription batch to 49 channels. Metadata, REST and websocket wire IDs retain the original public market identity.
- OKX crawler whole target: 47 passed, 7 failed. Five failures concern absent original USDT linear futures; two concern inverse-future trade arrival. OKX REST: 11 passed, 2 failed for the same unavailable market family.
- Coinbase unauthenticated public L2 existing case passed using its real 50ms batch feed. The venue's actual full/level3 response requires authentication; those distinct channels are not silently mapped to L2.

## Implementation and licensing

OKX candle subscriptions use its business websocket while other channels use the public websocket; both close explicitly. Its licensed parser patch accepts an empty optional next funding rate as absent, preserving strict parsing of nonempty values. Current Bitget v2 APIs and Binance's separate public/market websocket routing are implemented. Native conversion and BitMEX snapshot workers use owned threads so Tokio shutdown can finish. Shared fixture selection admits only real listed contracts in the original base, market family and option side.

Both local parser/pair patches retain upstream source and Apache licenses with prominent modification notices. Existing documentation examples now await asynchronous close and retain bounded real-message assertions.

## Exact command records

[Portable command records](maintenance/index.html) retain exact commands, outcomes, log hashes and available tracked source snapshots. Earlier exploratory failures and interrupted runs remain visible. Untracked additions are explicitly outside the tracked-diff snapshot hash.

[Engineering guide and seven offline diagrams](ayu-develop.html) · [Development workflow](development.md)
