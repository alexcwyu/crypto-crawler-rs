# crypto-pair UTF-8 compatibility patch

Source: crates.io crypto-pair 2.3.20; repository https://github.com/crypto-crawler/crypto-msg-parser/tree/main/crypto-pair. Original Apache-2.0 license retained.

Both Binance date-suffix probes use checked str::get so valid Unicode symbols fall through to quote matching rather than panic at a byte boundary. No algorithm or tests removed.
