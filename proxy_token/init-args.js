const ONE_DAY = 1_000_000_000 * 60 * 60 * 24;

// 1e24, calculated like this because JS numbers don't work that large
const ONE_NEAR = BigInt(1e12) ** 2n;

// Proxy NFT
const owner_id = "hosokawa.testnet";
const name = "Proxy NFT 1";
const symbol = "PNT";
const blank_media_uri = "https://ipfs.io/ipfs/QmV33AikTkQqS6vYokx9kafzCme84RsKpwZxPnV4SwC4xj";
const max_supply = 30;
const description = "Proxy Non fungible token"

console.log(JSON.stringify({owner_id, name, symbol, blank_media_uri, max_supply, description}));