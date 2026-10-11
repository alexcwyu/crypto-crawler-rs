// Local maintenance addition to the licensed crypto-msg-parser source.
// Genuine Bybit v5 public streams; original legacy decoders remain in bybit.rs.
use super::utils::calc_quantity_and_volume;
use crypto_market_type::MarketType;
use crypto_message::{CandlestickMsg, Order, OrderBookMsg, TradeMsg, TradeSide};
use crypto_msg_type::MessageType;
use serde_json::Value;
use simple_error::SimpleError;

pub(super) fn envelope(msg: &str, prefix: &str) -> Result<Option<Value>, SimpleError> {
    let value: Value = serde_json::from_str(msg).map_err(SimpleError::from)?;
    Ok(
        if value.get("topic").and_then(Value::as_str).is_some_and(|topic| topic.starts_with(prefix))
        {
            Some(value)
        } else {
            None
        },
    )
}
fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, SimpleError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| SimpleError::new(format!("Bybit v5 missing string {key}")))
}
fn number(value: &Value, key: &str) -> Result<f64, SimpleError> {
    text(value, key)?.parse::<f64>().map_err(SimpleError::from)
}
pub(super) fn timestamp(value: &Value) -> Result<i64, SimpleError> {
    if let Some(value) = value.as_i64() {
        return Ok(value);
    }
    value
        .as_str()
        .ok_or_else(|| SimpleError::new("Bybit v5 invalid timestamp"))?
        .parse::<i64>()
        .map_err(SimpleError::from)
}
fn pair(symbol: &str) -> Result<String, SimpleError> {
    crypto_pair::normalize_pair(symbol, "bybit")
        .ok_or_else(|| SimpleError::new(format!("Bybit v5 invalid symbol {symbol}")))
}
fn symbol(value: &Value) -> Result<&str, SimpleError> {
    text(value, "topic")?
        .rsplit('.')
        .next()
        .ok_or_else(|| SimpleError::new("Bybit v5 invalid topic"))
}
fn array(value: &Value) -> Result<&Vec<Value>, SimpleError> {
    value.as_array().ok_or_else(|| SimpleError::new("Bybit v5 expected array"))
}
pub(super) fn parse_trade(
    market_type: MarketType,
    msg: &str,
    value: Value,
) -> Result<Vec<TradeMsg>, SimpleError> {
    let rows = array(&value["data"])?;
    rows.iter()
        .map(|row| {
            let symbol = text(row, "s")?;
            let pair = pair(symbol)?;
            let price = number(row, "p")?;
            let quantity = number(row, "v")?;
            let (quantity_base, quantity_quote, quantity_contract) =
                calc_quantity_and_volume("bybit", market_type, &pair, price, quantity);
            let side = match text(row, "S")? {
                "Buy" => TradeSide::Buy,
                "Sell" => TradeSide::Sell,
                _ => return Err(SimpleError::new("Bybit v5 invalid trade side")),
            };
            Ok(TradeMsg {
                exchange: "bybit".to_string(),
                market_type,
                symbol: symbol.to_string(),
                pair,
                msg_type: MessageType::Trade,
                timestamp: timestamp(&row["T"])?,
                price,
                quantity_base,
                quantity_quote,
                quantity_contract,
                side,
                trade_id: text(row, "i")?.to_string(),
                json: if rows.len() == 1 {
                    msg.to_string()
                } else {
                    serde_json::to_string(row).map_err(SimpleError::from)?
                },
            })
        })
        .collect()
}
pub(super) fn parse_l2(
    market_type: MarketType,
    msg: &str,
    value: Value,
) -> Result<Vec<OrderBookMsg>, SimpleError> {
    let data = &value["data"];
    let symbol = text(data, "s")?;
    let pair = pair(symbol)?;
    let snapshot = match text(&value, "type")? {
        "snapshot" => true,
        "delta" => false,
        _ => return Err(SimpleError::new("Bybit v5 invalid book type")),
    };
    let orders = |key: &str| -> Result<Vec<Order>, SimpleError> {
        array(&data[key])?
            .iter()
            .map(|row| {
                let values = array(row)?;
                if values.len() != 2 {
                    return Err(SimpleError::new("Bybit v5 invalid book row"));
                }
                let price = values[0]
                    .as_str()
                    .ok_or_else(|| SimpleError::new("Bybit v5 invalid book price"))?
                    .parse::<f64>()
                    .map_err(SimpleError::from)?;
                let quantity = values[1]
                    .as_str()
                    .ok_or_else(|| SimpleError::new("Bybit v5 invalid book quantity"))?
                    .parse::<f64>()
                    .map_err(SimpleError::from)?;
                let (quantity_base, quantity_quote, quantity_contract) =
                    calc_quantity_and_volume("bybit", market_type, &pair, price, quantity);
                Ok(Order { price, quantity_base, quantity_quote, quantity_contract })
            })
            .collect()
    };
    Ok(vec![OrderBookMsg {
        exchange: "bybit".to_string(),
        market_type,
        symbol: symbol.to_string(),
        pair: pair.clone(),
        msg_type: MessageType::L2Event,
        timestamp: timestamp(&value["ts"])?,
        seq_id: data.get("u").and_then(Value::as_u64),
        prev_seq_id: None,
        asks: orders("a")?,
        bids: orders("b")?,
        snapshot,
        json: msg.to_string(),
    }])
}
pub(super) fn parse_candlestick(
    market_type: MarketType,
    msg: &str,
    value: Value,
) -> Result<Vec<CandlestickMsg>, SimpleError> {
    let symbol = symbol(&value)?;
    let pair = pair(symbol)?;
    array(&value["data"])?
        .iter()
        .map(|row| {
            let volume = number(row, "volume")?;
            let turnover = number(row, "turnover")?;
            let (volume, quote_volume) = match market_type {
                MarketType::InverseSwap | MarketType::InverseFuture => (turnover, Some(volume)),
                MarketType::LinearSwap | MarketType::LinearFuture | MarketType::Spot => {
                    (volume, Some(turnover))
                }
                _ => return Err(SimpleError::new("Bybit v5 unsupported candle market")),
            };
            Ok(CandlestickMsg {
                exchange: "bybit".to_string(),
                market_type,
                symbol: symbol.to_string(),
                pair: pair.clone(),
                msg_type: MessageType::Candlestick,
                timestamp: timestamp(&value["ts"])?,
                begin_time: timestamp(&row["start"])? / 1000,
                open: number(row, "open")?,
                high: number(row, "high")?,
                low: number(row, "low")?,
                close: number(row, "close")?,
                volume,
                period: text(row, "interval")?.to_string(),
                quote_volume,
                json: msg.to_string(),
            })
        })
        .collect()
}
