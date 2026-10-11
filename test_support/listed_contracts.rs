use crypto_market_type::MarketType;

// Listed futures/options replace expired 2022 fixtures without changing the
// exchange, underlying, settlement family, option side or market type.
// For options, prefer the original strike; if delisted, use the closest listed
// strike. Existing tests assert actual venue/type/message/parser behavior.
#[allow(dead_code)]
pub fn current_expiring_symbol(exchange: &str, market_type: MarketType, symbol: &str) -> String {
    if !matches!(
        market_type,
        MarketType::InverseFuture | MarketType::LinearFuture | MarketType::EuropeanOption
    ) {
        return symbol.to_owned();
    }
    let parts: Vec<&str> = symbol.split('-').collect();
    let prefix = if exchange == "binance" {
        format!("{}_", symbol.split('_').next().expect("contract prefix"))
    } else if exchange == "bybit" {
        let base_end = symbol.find("USD").expect("inverse USD contract prefix");
        symbol[..base_end + 3].to_string()
    } else if exchange == "okx" {
        format!("{}-{}-", parts[0], parts[1])
    } else {
        format!("{}-", parts[0])
    };
    let option_suffix = if market_type == MarketType::EuropeanOption {
        Some(format!("-{}", parts[parts.len() - 1]))
    } else {
        None
    };
    let mut markets = crypto_markets::fetch_markets(exchange, market_type)
        .expect("fetch currently listed contract metadata");
    markets.retain(|market| {
        market.active
            && market.symbol.starts_with(&prefix)
            && option_suffix.as_ref().map_or(true, |suffix| market.symbol.ends_with(suffix))
    });
    if market_type == MarketType::EuropeanOption {
        let original_strike =
            parts[parts.len() - 2].parse::<f64>().expect("numeric option fixture strike");
        markets.sort_by(|left, right| {
            let distance = |symbol: &str| {
                let fields: Vec<&str> = symbol.split('-').collect();
                (fields[fields.len() - 2].parse::<f64>().expect("numeric listed option strike")
                    - original_strike)
                    .abs()
            };
            distance(&left.symbol)
                .total_cmp(&distance(&right.symbol))
                .then_with(|| left.delivery_date.cmp(&right.delivery_date))
        });
    } else {
        let expiry = symbol.rsplit(['_', '-']).next().expect("fixture expiry");
        let prefer_quarterly = expiry
            .get(2..4)
            .is_some_and(|month| matches!(month, "03" | "06" | "09" | "12"))
            || expiry.get(2..5).is_some_and(|month| matches!(month, "MAR" | "JUN" | "SEP" | "DEC"));
        markets.sort_by_key(|market| {
            // Original December fixtures were quarterly contracts; prefer that
            // still-listed family rather than an illiquid weekly expiry.
            let quarterly = prefer_quarterly
                && (market.info.get("alias").and_then(serde_json::Value::as_str)
                    == Some("quarter")
                    || market.info.get("contractType").and_then(serde_json::Value::as_str)
                        == Some("CURRENT_QUARTER"));
            (!quarterly, market.delivery_date.unwrap_or(u64::MAX))
        });
    }
    markets
        .into_iter()
        .next()
        .expect("a currently listed contract with the original underlying and option side")
        .symbol
}
