// nft: deploy an MRC721 (enumerable + metadata) with a paid public mint.
import { Args, ArrayTypes, bytesToStr, Mas, U256 } from '@massalabs/massa-web3';
import { call, deploy, getProvider, read } from './utils.js';

const provider = await getProvider();
const price = Mas.fromString('0.5');
const contract = await deploy(
  provider,
  'nft',
  new Args().addString('Rust Punks').addString('RPUNK').addString('ipfs://rust-punks/').addU64(price),
);

// Public mint: send the price plus some margin for storage; the surplus is refunded.
await call(contract, 'mint_next', new Args(), price + Mas.fromString('0.1'));
await call(contract, 'mint_next', new Args(), price + Mas.fromString('0.1'));

const me = provider.address;
const owned = new Args((await read(contract, 'tokens_of', new Args().addString(me))).value).nextArray<bigint>(ArrayTypes.U256);
console.log(`my tokens: ${owned.join(', ')}`);
console.log(`uri(1) = ${bytesToStr((await read(contract, 'uri', new Args().addU256(1n))).value)}`);
console.log(`ownerOf(1) = ${bytesToStr((await read(contract, 'ownerOf', new Args().addU256(1n))).value)}`);
console.log(`totalSupply = ${U256.fromBytes((await read(contract, 'totalSupply')).value)}`);

await call(
  contract,
  'transferFrom',
  new Args().addString(me).addString('AU12UBnqTHDQALpocVBnkPNy7y5CndUJQTLutaVDDFgMJcq5kQiKq').addU256(1n),
  Mas.fromString('0.02'),
);
console.log(`balanceOf(me) = ${U256.fromBytes((await read(contract, 'balanceOf', new Args().addString(me))).value)}`);
await call(contract, 'withdraw', new Args().addString(me));
