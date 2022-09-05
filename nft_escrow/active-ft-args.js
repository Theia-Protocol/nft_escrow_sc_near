const ONE_DAY = 1_000_000_000 * 60 * 60 * 24;

// 1e24, calculated like this because JS numbers don't work that large
const ONE_NEAR = BigInt(1e12) ** 2n;

// Activate FT Project
const name = "Theia Collection 1";
const symbol = "TCN";
const blank_media_uri = "https://ipfs.io/ipfs/QmZRBnIklexQCvEmQxZ1CDqczgXcy7hatu9eZXW5FZUznj";
const max_supply = "100";
const finder_id = "hosokawa_test1.testnet";
const pre_mint_amount = "10";
const fund_threshold = (ONE_NEAR * 15n).toString();     // 15 USDT
const buffer_period = 0;
const conversion_period = 1800 * 1_000_000_000; // 30 min

console.log(JSON.stringify({name, symbol, blank_media_uri, max_supply, finder_id, pre_mint_amount, fund_threshold, buffer_period, conversion_period}));
