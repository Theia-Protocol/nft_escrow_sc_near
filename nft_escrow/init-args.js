const CURVE_TYPE_HORIZONTAL = 'Horizontal';
const CURVE_TYPE_LINEAR = 'Linear';
const CURVE_TYPE_SIGMOIDAL = 'Sigmoidal';

// initialize
const owner_id = "theia_owner.testnet";
const stable_coin_id = "dev-1662102511052-26947222593947";
const stable_coin_decimals = 24;
const curve_type = CURVE_TYPE_LINEAR;
const curve_args = {
    arg_a: 3,
    arg_b: 3
};
// const curve_args = {
//     arg_a: 50,
//     arg_b: 10,
//     arg_c: 100,
//     arg_d: 100
// };
const treasury_id = "theia_owner.testnet";

console.log(JSON.stringify({owner_id, stable_coin_id, stable_coin_decimals, curve_type, curve_args, treasury_id}));