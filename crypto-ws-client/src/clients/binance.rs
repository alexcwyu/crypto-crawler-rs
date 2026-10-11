use async_trait::async_trait;
use nonzero_ext::nonzero;
use std::{collections::HashMap, num::NonZeroU32};
use tokio_tungstenite::tungstenite::Message;

use crate::{
    WSClient,
    common::{
        command_translator::CommandTranslator,
        message_handler::{MessageHandler, MiscMessage},
        utils::ensure_frame_size,
        ws_client_internal::WSClientInternal,
    },
};
use log::*;
use serde_json::Value;

pub(crate) const EXCHANGE_NAME: &str = "binance";

const SPOT_WEBSOCKET_URL: &str = "wss://stream.binance.com:9443/stream";
const LINEAR_WEBSOCKET_URL: &str = "wss://fstream.binance.com/public/stream";
const LINEAR_MARKET_WEBSOCKET_URL: &str = "wss://fstream.binance.com/market/stream";
const INVERSE_WEBSOCKET_URL: &str = "wss://dstream.binance.com/stream";

// the websocket message size should not exceed 4096 bytes, otherwise
// you'll get `code: 3001, reason: illegal request`
const WS_FRAME_SIZE: usize = 4096;

// WebSocket connections have a limit of 5 incoming messages per second.
//
// See:
//
// * https://binance-docs.github.io/apidocs/spot/en/#limits
// * https://binance-docs.github.io/apidocs/futures/en/#websocket-market-streams
// * https://binance-docs.github.io/apidocs/delivery/en/#websocket-market-streams
const UPLINK_LIMIT: (NonZeroU32, std::time::Duration) =
    (nonzero!(5u32), std::time::Duration::from_secs(1));

// Internal unified client
pub struct BinanceWSClient<const MARKET_TYPE: char> {
    client: WSClientInternal<BinanceMessageHandler>,
    translator: BinanceCommandTranslator,
    market_client: Option<WSClientInternal<BinanceMessageHandler>>,
}

/// Binance Spot market.
///
///   * WebSocket API doc: <https://binance-docs.github.io/apidocs/spot/en/>
///   * Trading at: <https://www.binance.com/en/trade/BTC_USDT>
pub type BinanceSpotWSClient = BinanceWSClient<'S'>;

/// Binance Coin-margined Future and Swap markets.
///
///   * WebSocket API doc: <https://binance-docs.github.io/apidocs/delivery/en/>
///   * Trading at: <https://www.binance.com/en/delivery/btcusd_quarter>
pub type BinanceInverseWSClient = BinanceWSClient<'I'>;

/// Binance USDT-margined Future and Swap markets.
///
///   * WebSocket API doc: <https://binance-docs.github.io/apidocs/futures/en/>
///   * Trading at: <https://www.binance.com/en/futures/BTC_USDT>
pub type BinanceLinearWSClient = BinanceWSClient<'L'>;

impl<const MARKET_TYPE: char> BinanceWSClient<MARKET_TYPE> {
    pub async fn new(tx: std::sync::mpsc::Sender<String>, url: Option<&str>) -> Self {
        let real_url = match url {
            Some(endpoint) => endpoint,
            None => {
                if MARKET_TYPE == 'S' {
                    SPOT_WEBSOCKET_URL
                } else if MARKET_TYPE == 'I' {
                    INVERSE_WEBSOCKET_URL
                } else if MARKET_TYPE == 'L' {
                    LINEAR_WEBSOCKET_URL
                } else {
                    panic!("Unknown market type {MARKET_TYPE}");
                }
            }
        };
        // Binance separated futures depth/BBO and trade/ticker/kline streams in 2026.
        // Explicit URL overrides preserve the single-endpoint client contract.
        let market_client = if MARKET_TYPE == 'L' && url.is_none() {
            Some(
                WSClientInternal::connect(
                    EXCHANGE_NAME,
                    LINEAR_MARKET_WEBSOCKET_URL,
                    BinanceMessageHandler {},
                    Some(UPLINK_LIMIT),
                    tx.clone(),
                )
                .await,
            )
        } else {
            None
        };
        BinanceWSClient {
            market_client,
            client: WSClientInternal::connect(
                EXCHANGE_NAME,
                real_url,
                BinanceMessageHandler {},
                Some(UPLINK_LIMIT),
                tx,
            )
            .await,
            translator: BinanceCommandTranslator { market_type: MARKET_TYPE },
        }
    }

    async fn send_routed(&self, commands: &[String]) {
        let Some(market_client) = &self.market_client else {
            self.client.send(commands).await;
            return;
        };
        let mut public_commands = Vec::new();
        let mut market_commands = Vec::new();
        for command in commands {
            let Ok(value) = serde_json::from_str::<Value>(command) else {
                public_commands.push(command.clone());
                continue;
            };
            let Some(params) = value.get("params").and_then(Value::as_array) else {
                public_commands.push(command.clone());
                continue;
            };
            let (public, market): (Vec<_>, Vec<_>) = params.iter().cloned().partition(|param| {
                let channel = param.as_str().unwrap_or_default();
                channel.contains("@depth")
                    || channel.contains("bookTicker")
                    || channel.contains("@rpiDepth")
            });
            for (params, target) in [(public, &mut public_commands), (market, &mut market_commands)]
            {
                if !params.is_empty() {
                    let mut routed = value.clone();
                    routed["params"] = Value::Array(params);
                    target.push(routed.to_string());
                }
            }
        }
        if !public_commands.is_empty() {
            self.client.send(&public_commands).await;
        }
        if !market_commands.is_empty() {
            market_client.send(&market_commands).await;
        }
    }
}

#[async_trait]
impl<const URL: char> WSClient for BinanceWSClient<URL> {
    async fn subscribe_trade(&self, symbols: &[String]) {
        let topics = symbols
            .iter()
            .map(|symbol| ("aggTrade".to_string(), symbol.to_string()))
            .collect::<Vec<(String, String)>>();
        self.subscribe(&topics).await;
    }

    async fn subscribe_orderbook(&self, symbols: &[String]) {
        let topics = symbols
            .iter()
            .map(|symbol| ("depth@100ms".to_string(), symbol.to_string()))
            .collect::<Vec<(String, String)>>();
        self.subscribe(&topics).await;
    }

    async fn subscribe_orderbook_topk(&self, symbols: &[String]) {
        let topics = symbols
            .iter()
            .map(|symbol| ("depth20".to_string(), symbol.to_string()))
            .collect::<Vec<(String, String)>>();
        self.subscribe(&topics).await;
    }

    async fn subscribe_l3_orderbook(&self, _symbols: &[String]) {
        panic!("{EXCHANGE_NAME} does NOT have the level3 websocket channel");
    }

    async fn subscribe_ticker(&self, symbols: &[String]) {
        let topics = symbols
            .iter()
            .map(|symbol| ("ticker".to_string(), symbol.to_string()))
            .collect::<Vec<(String, String)>>();
        self.subscribe(&topics).await;
    }

    async fn subscribe_bbo(&self, symbols: &[String]) {
        let topics = symbols
            .iter()
            .map(|symbol| ("bookTicker".to_string(), symbol.to_string()))
            .collect::<Vec<(String, String)>>();
        self.subscribe(&topics).await;
    }

    async fn subscribe_candlestick(&self, symbol_interval_list: &[(String, usize)]) {
        let commands =
            self.translator.translate_to_candlestick_commands(true, symbol_interval_list);
        self.send_routed(&commands).await;
    }

    async fn subscribe(&self, topics: &[(String, String)]) {
        let commands = self.translator.translate_to_commands(true, topics);
        self.send_routed(&commands).await;
    }

    async fn unsubscribe(&self, topics: &[(String, String)]) {
        let commands = self.translator.translate_to_commands(false, topics);
        self.send_routed(&commands).await;
    }

    async fn send(&self, commands: &[String]) {
        self.send_routed(commands).await;
    }

    async fn run(&self) {
        if let Some(market_client) = &self.market_client {
            tokio::join!(self.client.run(), market_client.run());
        } else {
            self.client.run().await;
        }
    }

    async fn close(&self) {
        self.client.close().await;
        if let Some(market_client) = &self.market_client {
            market_client.close().await;
        }
    }
}

struct BinanceMessageHandler {}
struct BinanceCommandTranslator {
    market_type: char,
}

impl BinanceCommandTranslator {
    fn topics_to_command(topics: &[(String, String)], subscribe: bool) -> String {
        let raw_topics = topics
            .iter()
            .map(|(topic, symbol)| format!("{}@{}", symbol.to_lowercase(), topic))
            .collect::<Vec<String>>();
        format!(
            r#"{{"id":9527,"method":"{}","params":{}}}"#,
            if subscribe { "SUBSCRIBE" } else { "UNSUBSCRIBE" },
            serde_json::to_string(&raw_topics).unwrap()
        )
    }

    // see https://binance-docs.github.io/apidocs/futures/en/#kline-candlestick-streams
    fn to_candlestick_raw_channel(interval: usize) -> String {
        let interval_str = match interval {
            60 => "1m",
            180 => "3m",
            300 => "5m",
            900 => "15m",
            1800 => "30m",
            3600 => "1h",
            7200 => "2h",
            14400 => "4h",
            21600 => "6h",
            28800 => "8h",
            43200 => "12h",
            86400 => "1d",
            259200 => "3d",
            604800 => "1w",
            2592000 => "1M",
            _ => panic!("Binance has intervals 1m,3m,5m,15m,30m,1h,2h,4h,6h,8h,12h,1d,3d,1w,1M"),
        };
        format!("kline_{interval_str}")
    }
}

impl MessageHandler for BinanceMessageHandler {
    fn handle_message(&mut self, msg: &str) -> MiscMessage {
        let resp = serde_json::from_str::<HashMap<String, Value>>(msg);
        if resp.is_err() {
            error!("{} is not a JSON string, {}", msg, EXCHANGE_NAME);
            return MiscMessage::Other;
        }
        let obj = resp.unwrap();

        if obj.contains_key("error") {
            panic!("Received {msg} from {EXCHANGE_NAME}");
        } else if obj.contains_key("stream") && obj.contains_key("data") {
            MiscMessage::Normal
        } else {
            if let Some(result) = obj.get("result") {
                if serde_json::Value::Null != *result {
                    panic!("Received {msg} from {EXCHANGE_NAME}");
                } else {
                    info!("Received {} from {}", msg, EXCHANGE_NAME);
                }
            } else {
                warn!("Received {} from {}", msg, EXCHANGE_NAME);
            }
            MiscMessage::Other
        }
    }

    fn get_ping_msg_and_interval(&self) -> Option<(Message, u64)> {
        // https://binance-docs.github.io/apidocs/spot/en/#websocket-market-streams
        // https://binance-docs.github.io/apidocs/futures/en/#websocket-market-streams
        // https://binance-docs.github.io/apidocs/delivery/en/#websocket-market-streams
        // The websocket server will send a ping frame every 3 minutes. If the websocket
        // server does not receive a pong frame back from the connection within
        // a 10 minute period, the connection will be disconnected. Unsolicited
        // pong frames are allowed. Send unsolicited pong frames per 3 minutes
        Some((Message::Pong(Vec::new()), 180))
    }
}

impl CommandTranslator for BinanceCommandTranslator {
    fn translate_to_commands(&self, subscribe: bool, topics: &[(String, String)]) -> Vec<String> {
        let max_num_topics = if self.market_type == 'S' {
            // https://binance-docs.github.io/apidocs/spot/en/#websocket-limits
            1024
        } else {
            // https://binance-docs.github.io/apidocs/futures/en/#websocket-market-streams
            // https://binance-docs.github.io/apidocs/delivery/en/#websocket-market-streams
            200
        };
        ensure_frame_size(
            topics,
            subscribe,
            Self::topics_to_command,
            WS_FRAME_SIZE,
            Some(max_num_topics),
        )
    }

    fn translate_to_candlestick_commands(
        &self,
        subscribe: bool,
        symbol_interval_list: &[(String, usize)],
    ) -> Vec<String> {
        let topics = symbol_interval_list
            .iter()
            .map(|(symbol, interval)| {
                let channel = Self::to_candlestick_raw_channel(*interval);
                (channel, symbol.to_lowercase())
            })
            .collect::<Vec<(String, String)>>();
        self.translate_to_commands(subscribe, &topics)
    }
}

#[cfg(test)]
mod tests {
    use crate::common::command_translator::CommandTranslator;

    #[test]
    fn test_one_topic() {
        let translator = super::BinanceCommandTranslator { market_type: 'S' };
        let commands = translator
            .translate_to_commands(true, &[("aggTrade".to_string(), "BTCUSDT".to_string())]);

        assert_eq!(1, commands.len());
        assert_eq!(
            r#"{"id":9527,"method":"SUBSCRIBE","params":["btcusdt@aggTrade"]}"#,
            commands[0]
        );
    }

    #[test]
    fn test_two_topics() {
        let translator = super::BinanceCommandTranslator { market_type: 'S' };
        let commands = translator.translate_to_commands(
            true,
            &[
                ("aggTrade".to_string(), "BTCUSDT".to_string()),
                ("ticker".to_string(), "BTCUSDT".to_string()),
            ],
        );

        assert_eq!(1, commands.len());
        assert_eq!(
            r#"{"id":9527,"method":"SUBSCRIBE","params":["btcusdt@aggTrade","btcusdt@ticker"]}"#,
            commands[0]
        );
    }
}
