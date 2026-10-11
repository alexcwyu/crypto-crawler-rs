#[macro_use]
mod utils;

use test_case::test_case;

use crypto_crawler::*;
use crypto_market_type::MarketType;
use crypto_msg_type::MessageType;

use utils::parse;

const EXCHANGE_NAME: &str = "binance";

// Futures expire: discover the nearest currently listed BTC contract instead of
// subscribing forever to a December 2022 instrument.
fn current_test_symbol(market_type: MarketType, symbol: &str) -> String {
    if matches!(market_type, MarketType::InverseFuture | MarketType::LinearFuture) {
        let prefix = symbol.split('_').next().expect("BTC contract prefix");
        let mut contracts = crypto_markets::fetch_markets(EXCHANGE_NAME, market_type)
            .expect("fetch currently trading Binance futures");
        contracts.retain(|contract| {
            contract.active && contract.symbol.starts_with(&format!("{prefix}_"))
        });
        contracts.sort_by_key(|contract| {
            let quarterly = contract.info.get("contractType").and_then(serde_json::Value::as_str)
                == Some("CURRENT_QUARTER");
            (!quarterly, contract.delivery_date.unwrap_or(u64::MAX))
        });
        contracts.into_iter().next().expect("at least one current BTC future").symbol
    } else {
        symbol.to_owned()
    }
}

#[test_case(MarketType::Spot)]
#[test_case(MarketType::InverseFuture)]
#[test_case(MarketType::LinearFuture)]
#[test_case(MarketType::InverseSwap)]
#[test_case(MarketType::LinearSwap)]
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_crawl_trade_all(market_type: MarketType) {
    test_all_symbols!(crawl_trade, EXCHANGE_NAME, market_type, MessageType::Trade)
}

#[test_case(MarketType::Spot, "BTCUSDT")]
#[test_case(MarketType::InverseFuture, "BTCUSD_CURRENT")]
#[test_case(MarketType::LinearFuture, "BTCUSDT_CURRENT")]
#[test_case(MarketType::InverseSwap, "BTCUSD_PERP")]
#[test_case(MarketType::LinearSwap, "BTCUSDT")]
// #[test_case(MarketType::EuropeanOption, "BTC-220610-30000-C"; "ignore")]
#[tokio::test(flavor = "multi_thread")]
async fn test_crawl_trade(market_type: MarketType, symbol: &str) {
    let current_symbol = tokio::task::block_in_place(|| current_test_symbol(market_type, symbol));
    let symbol = current_symbol.as_str();
    test_one_symbol!(crawl_trade, EXCHANGE_NAME, market_type, symbol, MessageType::Trade)
}

#[test_case(MarketType::Spot, "BTCUSDT")]
#[test_case(MarketType::InverseFuture, "BTCUSD_CURRENT")]
#[test_case(MarketType::LinearFuture, "BTCUSDT_CURRENT")]
#[test_case(MarketType::InverseSwap, "BTCUSD_PERP")]
#[test_case(MarketType::LinearSwap, "BTCUSDT")]
// #[test_case(MarketType::EuropeanOption, "BTC-220610-30000-C"; "ignore")]
#[tokio::test(flavor = "multi_thread")]
async fn test_crawl_l2_event(market_type: MarketType, symbol: &str) {
    let current_symbol = tokio::task::block_in_place(|| current_test_symbol(market_type, symbol));
    let symbol = current_symbol.as_str();
    test_one_symbol!(crawl_l2_event, EXCHANGE_NAME, market_type, symbol, MessageType::L2Event)
}

#[test_case(MarketType::Spot, "BTCUSDT")]
#[test_case(MarketType::InverseFuture, "BTCUSD_CURRENT")]
#[test_case(MarketType::LinearFuture, "BTCUSDT_CURRENT")]
#[test_case(MarketType::InverseSwap, "BTCUSD_PERP")]
#[test_case(MarketType::LinearSwap, "BTCUSDT")]
// #[test_case(MarketType::EuropeanOption, "BTC-220610-30000-C"; "ignore")]
#[tokio::test(flavor = "multi_thread")]
async fn test_crawl_bbo(market_type: MarketType, symbol: &str) {
    let current_symbol = tokio::task::block_in_place(|| current_test_symbol(market_type, symbol));
    let symbol = current_symbol.as_str();
    test_one_symbol!(crawl_bbo, EXCHANGE_NAME, market_type, symbol, MessageType::BBO)
}

#[test_case(MarketType::Spot, "BTCUSDT")]
#[test_case(MarketType::InverseFuture, "BTCUSD_CURRENT")]
#[test_case(MarketType::LinearFuture, "BTCUSDT_CURRENT")]
#[test_case(MarketType::InverseSwap, "BTCUSD_PERP")]
#[test_case(MarketType::LinearSwap, "BTCUSDT")]
// #[test_case(MarketType::EuropeanOption, "BTC-220610-30000-C"; "ignore")]
#[tokio::test(flavor = "multi_thread")]
async fn test_crawl_l2_topk(market_type: MarketType, symbol: &str) {
    let current_symbol = tokio::task::block_in_place(|| current_test_symbol(market_type, symbol));
    let symbol = current_symbol.as_str();
    test_one_symbol!(crawl_l2_topk, EXCHANGE_NAME, market_type, symbol, MessageType::L2TopK)
}

#[test_case(MarketType::Spot, "BTCUSDT")]
#[test_case(MarketType::InverseFuture, "BTCUSD_CURRENT")]
#[test_case(MarketType::LinearFuture, "BTCUSDT_CURRENT")]
#[test_case(MarketType::InverseSwap, "BTCUSD_PERP")]
#[test_case(MarketType::LinearSwap, "BTCUSDT")]
// #[test_case(MarketType::EuropeanOption, "BTC-220610-30000-C")]
fn test_crawl_l2_snapshot(market_type: MarketType, symbol: &str) {
    let current_symbol = current_test_symbol(market_type, symbol);
    let symbol = current_symbol.as_str();
    test_crawl_restful!(
        crawl_l2_snapshot,
        EXCHANGE_NAME,
        market_type,
        symbol,
        MessageType::L2Snapshot
    )
}

#[test_case(MarketType::Spot)]
#[test_case(MarketType::InverseFuture)]
#[test_case(MarketType::LinearFuture)]
#[test_case(MarketType::InverseSwap)]
#[test_case(MarketType::LinearSwap)]
// #[test_case(MarketType::EuropeanOption)]
fn test_crawl_l2_snapshot_without_symbol(market_type: MarketType) {
    test_crawl_restful_all_symbols!(
        crawl_l2_snapshot,
        EXCHANGE_NAME,
        market_type,
        MessageType::L2Snapshot
    )
}

#[test_case(MarketType::Spot, "BTCUSDT")]
#[test_case(MarketType::InverseFuture, "BTCUSD_CURRENT")]
#[test_case(MarketType::LinearFuture, "BTCUSDT_CURRENT")]
#[test_case(MarketType::InverseSwap, "BTCUSD_PERP")]
#[test_case(MarketType::LinearSwap, "BTCUSDT")]
// #[test_case(MarketType::EuropeanOption, "BTC-220610-30000-C"; "ignore")]
#[tokio::test(flavor = "multi_thread")]
async fn test_crawl_ticker(market_type: MarketType, symbol: &str) {
    let current_symbol = tokio::task::block_in_place(|| current_test_symbol(market_type, symbol));
    let symbol = current_symbol.as_str();
    test_one_symbol!(crawl_ticker, EXCHANGE_NAME, market_type, symbol, MessageType::Ticker)
}

#[test_case(MarketType::InverseSwap, "BTCUSD_PERP")]
#[test_case(MarketType::LinearSwap, "BTCUSDT")]
#[tokio::test(flavor = "multi_thread")]
async fn test_crawl_funding_rate(market_type: MarketType, symbol: &str) {
    test_one_symbol!(
        crawl_funding_rate,
        EXCHANGE_NAME,
        market_type,
        symbol,
        MessageType::FundingRate
    )
}

#[test_case(MarketType::Spot)]
#[test_case(MarketType::InverseFuture)]
#[test_case(MarketType::LinearFuture)]
#[test_case(MarketType::InverseSwap)]
#[test_case(MarketType::LinearSwap)]
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_crawl_candlestick(market_type: MarketType) {
    gen_test_crawl_candlestick!(EXCHANGE_NAME, market_type)
}

// #[test_case(MarketType::Spot, "BTCUSDT")]
// #[test_case(MarketType::InverseFuture, "BTCUSD_CURRENT")]
// #[test_case(MarketType::LinearFuture, "BTCUSDT_CURRENT")]
// #[test_case(MarketType::InverseSwap, "BTCUSD_PERP")]
// #[test_case(MarketType::LinearSwap, "BTCUSDT")]
// #[test_case(MarketType::EuropeanOption, "BTC-220610-30000-C"; "ignore")]
// #[tokio::test(flavor = "multi_thread")]
// async fn test_subscribe_symbol(market_type: MarketType, symbol: &str) {
//     gen_test_subscribe_symbol!(EXCHANGE_NAME, market_type, symbol)
// }
