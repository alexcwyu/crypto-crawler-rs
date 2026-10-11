use super::utils::http_get;
use crate::error::{Error, Result};
use serde_json::Value;
use std::collections::BTreeMap;

/// Public Bybit v5 market-data client. Credentials are retained for API compatibility.
pub struct BybitRestClient {
    _api_key: Option<String>,
    _api_secret: Option<String>,
}

impl BybitRestClient {
    pub fn new(api_key: Option<String>, api_secret: Option<String>) -> Self {
        BybitRestClient { _api_key: api_key, _api_secret: api_secret }
    }

    fn request(path: &str, symbol: &str, parameters: &[(&str, &str)]) -> Result<String> {
        let category =
            if symbol.ends_with("USDT") || symbol.ends_with("PERP") { "linear" } else { "inverse" };
        let mut url = reqwest::Url::parse(&format!("https://api.bybit.com/v5/market/{path}"))
            .map_err(|error| Error(error.to_string()))?;
        url.query_pairs_mut().append_pair("category", category).append_pair("symbol", symbol);
        for (key, value) in parameters {
            url.query_pairs_mut().append_pair(key, value);
        }
        let response = http_get(url.as_str(), &BTreeMap::new())?;
        let value: Value = serde_json::from_str(&response)?;
        if value.get("retCode").and_then(Value::as_i64) != Some(0) {
            return Err(Error(format!("Bybit v5 error response: {response}")));
        }
        Ok(response)
    }

    /// Latest real snapshot: 25 bids and 25 asks, preserving the former 50-row scope.
    /// Schema: https://bybit-exchange.github.io/docs/v5/market/orderbook
    pub fn fetch_l2_snapshot(symbol: &str) -> Result<String> {
        Self::request("orderbook", symbol, &[("limit", "25")])
    }

    /// Genuine v5 five-minute open-interest records.
    pub fn fetch_open_interest(symbol: &str) -> Result<String> {
        Self::request("open-interest", symbol, &[("intervalTime", "5min"), ("limit", "200")])
    }

    /// Genuine v5 account ratios. Unsupported contract families return venue errors.
    pub fn fetch_long_short_ratio(symbol: &str) -> Result<String> {
        Self::request("account-ratio", symbol, &[("period", "5min"), ("limit", "200")])
    }
}
