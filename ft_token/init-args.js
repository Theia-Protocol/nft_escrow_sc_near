const ONE_DAY = 1_000_000_000 * 60 * 60 * 24;

// 1e24, calculated like this because JS numbers don't work that large
const ONE_NEAR = BigInt(1e12) ** 2n;

// Fungible token
const owner_id = "theia_owner.testnet";
const name = "USD Tether";
const symbol = "USDT";

console.log(JSON.stringify({owner_id, name, symbol}));