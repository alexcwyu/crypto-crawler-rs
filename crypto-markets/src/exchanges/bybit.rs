use std::collections::{HashMap, HashSet};

use super::utils::http_get;
use crate::{
    Fees, Market, MarketType, Precision, QuantityLimit,
    error::{Error, Result},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PriceFilter {
    tick_size: String,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LotSizeFilter {
    max_order_qty: String,
    min_order_qty: String,
    qty_step: String,
    #[serde(default)]
    min_notional_value: Option<String>,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BybitMarket {
    symbol: String,
    contract_type: String,
    status: String,
    base_coin: String,
    quote_coin: String,
    settle_coin: String,
    delivery_time: String,
    price_filter: PriceFilter,
    lot_size_filter: LotSizeFilter,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Page {
    list: Vec<BybitMarket>,
    #[serde(default)]
    next_page_cursor: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Response {
    ret_code: i64,
    ret_msg: String,
    result: Page,
}

fn category_and_contract(market_type: MarketType) -> (&'static str, &'static str) {
    match market_type {
        MarketType::InverseSwap => ("inverse", "InversePerpetual"),
        MarketType::LinearSwap => ("linear", "LinearPerpetual"),
        MarketType::InverseFuture => ("inverse", "InverseFutures"),
        _ => panic!("Unsupported market_type: {market_type}"),
    }
}

// https://bybit-exchange.github.io/docs/v5/market/instrument
fn fetch_markets_raw(market_type: MarketType) -> Result<Vec<BybitMarket>> {
    let (category, contract) = category_and_contract(market_type);
    let mut markets = Vec::new();
    let mut cursor = String::new();
    let mut cursors = HashSet::new();
    loop {
        let mut url = reqwest::Url::parse("https://api.bybit.com/v5/market/instruments-info")
            .map_err(|e| Error(e.to_string()))?;
        url.query_pairs_mut().append_pair("category", category).append_pair("limit", "1000");
        if !cursor.is_empty() {
            url.query_pairs_mut().append_pair("cursor", &cursor);
        }
        let txt = http_get(url.as_str(), None)?;
        let response: Response = serde_json::from_str(&txt)?;
        if response.ret_code != 0 {
            return Err(Error(format!("Bybit v5 {}: {}", response.ret_code, response.ret_msg)));
        }
        markets.extend(
            response
                .result
                .list
                .into_iter()
                .filter(|market| market.status == "Trading" && market.contract_type == contract),
        );
        cursor = response.result.next_page_cursor;
        if cursor.is_empty() {
            return Ok(markets);
        }
        if !cursors.insert(cursor.clone()) {
            return Err(Error("Bybit v5 repeated pagination cursor".to_string()));
        }
    }
}

pub(crate) fn fetch_symbols(market_type: MarketType) -> Result<Vec<String>> {
    Ok(fetch_markets_raw(market_type)?.into_iter().map(|market| market.symbol).collect())
}

fn number(value: &str, field: &str) -> Result<f64> {
    value.parse::<f64>().map_err(|e| Error(format!("Bybit v5 invalid {field}: {e}")))
}

fn to_market(raw: BybitMarket, market_type: MarketType) -> Result<Market> {
    let delivery_date = if market_type == MarketType::InverseFuture {
        Some(
            raw.delivery_time
                .parse::<u64>()
                .map_err(|e| Error(format!("Bybit v5 deliveryTime: {e}")))?,
        )
    } else {
        None
    };
    // V5 public instrument metadata does not provide account-specific fees.
    // Published VIP0 baselines, not authenticated account quotes:
    // https://www.bybit.com/en/help-center/article/Trading-Fee-Structure
    // https://announcements.bybit.com/en/article/tradfi-perpetuals-lower-fees-across-all-tiers-bltb196506dada4be39/
    let tradfi = raw
        .extra
        .get("underlyingTicker")
        .and_then(Value::as_str)
        .is_some_and(|ticker| !ticker.is_empty());
    let fees = if tradfi {
        Fees { maker: 0.0, taker: 0.000275 }
    } else {
        Fees { maker: 0.0002, taker: 0.00055 }
    };
    let mut info = serde_json::to_value(&raw)?.as_object().unwrap().clone();
    info.insert(
        "feeSource".to_string(),
        Value::String("published VIP0 baseline; account rates may differ".to_string()),
    );
    Ok(Market {
        exchange: "bybit".to_string(),
        market_type,
        symbol: raw.symbol,
        base_id: raw.base_coin.clone(),
        quote_id: raw.quote_coin.clone(),
        settle_id: Some(raw.settle_coin.clone()),
        base: raw.base_coin,
        quote: raw.quote_coin,
        settle: Some(raw.settle_coin),
        active: raw.status == "Trading",
        margin: true,
        fees,
        precision: Precision {
            tick_size: number(&raw.price_filter.tick_size, "tickSize")?,
            lot_size: number(&raw.lot_size_filter.qty_step, "qtyStep")?,
        },
        quantity_limit: Some(QuantityLimit {
            min: Some(number(&raw.lot_size_filter.min_order_qty, "minOrderQty")?),
            max: Some(number(&raw.lot_size_filter.max_order_qty, "maxOrderQty")?),
            notional_min: raw
                .lot_size_filter
                .min_notional_value
                .as_deref()
                .map(|value| number(value, "minNotionalValue"))
                .transpose()?,
            notional_max: None,
        }),
        contract_value: Some(1.0),
        delivery_date,
        info,
    })
}

pub(crate) fn fetch_markets(market_type: MarketType) -> Result<Vec<Market>> {
    fetch_markets_raw(market_type)?.into_iter().map(|raw| to_market(raw, market_type)).collect()
}
