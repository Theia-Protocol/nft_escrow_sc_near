const ONE_DAY = 1_000_000_000 * 60 * 60 * 24;

// 1e24, calculated like this because JS numbers don't work that large
const ONE_NEAR = BigInt(1e12) ** 2n;

// Nft collection
const owner_id = "hosokawa.testnet";
const name = "NFT Collection 1";
const symbol = "NCC";
const base_uri = "https://ipfs.io/ipfs/QmV33AikTkQqS6vYokx9kafzCme84RsKpwZxPnV4SwC4xj/";
const max_supply = 30;
const media_base_uri = "https://ipfs.io/ipfs/QmV33AikTkQqS6vYokx9kafzCme84RsKpwZxPnV4SwC4xj/";
const description = "Real Non fungible token collection";

console.log(JSON.stringify({owner_id, name, symbol, base_uri, max_supply, media_base_uri, description}));