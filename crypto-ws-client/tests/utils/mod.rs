macro_rules! gen_test_code {
    ($client:ident, $func_name:ident, $symbols:expr) => {
        let (tx, rx) = std::sync::mpsc::channel();
        tokio::task::spawn(async move {
            let ws_client = $client::new(tx, None).await;
            ws_client.$func_name($symbols).await;
            // run for 60 seconds at most
            let _ = tokio::time::timeout(std::time::Duration::from_secs(60), ws_client.run()).await;
            ws_client.close().await;
        });

        let mut messages = Vec::<String>::new();
        let msg = rx.recv_timeout(std::time::Duration::from_secs(90))
            .expect("live exchange did not deliver a message within 90 seconds (metadata plus the original 60-second stream window)");
        messages.push(msg);
        assert!(!messages.is_empty());
    };
}

#[allow(unused_macros)]
macro_rules! gen_test_subscribe_candlestick {
    ($client:ident, $symbol_interval_list:expr) => {
        let (tx, rx) = std::sync::mpsc::channel();
        tokio::task::spawn(async move {
            let ws_client = $client::new(tx, None).await;
            ws_client.subscribe_candlestick($symbol_interval_list).await;
            // run for 60 seconds at most
            let _ = tokio::time::timeout(std::time::Duration::from_secs(60), ws_client.run()).await;
            ws_client.close().await;
        });

        let mut messages = Vec::<String>::new();
        let msg = rx.recv_timeout(std::time::Duration::from_secs(90))
            .expect("live exchange did not deliver a message within 90 seconds (metadata plus the original 60-second stream window)");
        messages.push(msg);
        assert!(!messages.is_empty());
    };
}
