# Local modification notice

Licensed Apache-2.0 crypto-msg-parser 2.9.2, copied from the published crate with its original source and metadata retained. Original LICENSE was retrieved from https://github.com/crypto-crawler/crypto-msg-parser/blob/main/LICENSE (shared upstream project license).

An empty optional OKX `nextFundingRate` now maps to `None` in the existing optional estimated-rate field; nonempty values retain their original strict parse. No current-rate/time values or assertions are replaced. The current venue may omit a next-period forecast. Original library tests remain supplied; no new cases were added.
