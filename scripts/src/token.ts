// token: deploy an MRC20 and drive it with massa-web3's standard MRC20 wrapper.
import { Args, ArrayTypes, MRC20, Mas } from '@massalabs/massa-web3';
import { call, deploy, getProvider } from './utils.js';

const provider = await getProvider();
const supply = 1_000_000n * 10n ** 9n;
const deployed = await deploy(
  provider,
  'token',
  new Args().addString('Rust Massa Token').addString('RMT').addU8(9n).addU256(supply),
);
const token = new MRC20(provider, deployed.address);

console.log(`${await token.name()} (${await token.symbol()}), ${await token.decimals()} decimals, supply ${await token.totalSupply()}`);
const me = provider.address;
console.log(`my balance: ${await token.balanceOf(me)}`);

// Fresh recipient: the transfer creates its balance entry, the wrapper sends the storage cost.
const recipient = 'AU12UBnqTHDQALpocVBnkPNy7y5CndUJQTLutaVDDFgMJcq5kQiKq';
const op = await token.transfer(recipient, 1_000n * 10n ** 9n);
await op.waitSpeculativeExecution();
console.log(`transferred, recipient balance: ${await token.balanceOf(recipient)}`);

// Owner-only mint, then the custom `airdrop` entry point.
await call(deployed, 'mint', new Args().addString(recipient).addU256(5n), Mas.fromString('0.01'));
await call(
  deployed,
  'airdrop',
  new Args().addArray([recipient, 'AU1dQZLc618t4CNYWYiDnFnq585kifefr1EAcMi5jddJZGQJXJXL'], ArrayTypes.STRING).addU256(7n),
  Mas.fromString('0.02'),
);
console.log(`recipient balance after mint + airdrop: ${await token.balanceOf(recipient)}`);
console.log(`total supply: ${await token.totalSupply()}`);
