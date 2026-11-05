// multi-token: MRC1155 with roles; mint, batch mint, transfer, burn.
import { Args, ArrayTypes, bytesToStr, Mas, U256 } from '@massalabs/massa-web3';
import { call, deploy, getProvider, read } from './utils.js';

const provider = await getProvider();
const contract = await deploy(provider, 'multi-token', new Args().addString('ipfs://rust-1155/{id}.json'));
const me = provider.address;
const other = 'AU12UBnqTHDQALpocVBnkPNy7y5CndUJQTLutaVDDFgMJcq5kQiKq';

await call(contract, 'mint', new Args().addString(me).addU256(7n).addU256(100n).addUint8Array(new Uint8Array()), Mas.fromString('0.02'));
await call(
  contract,
  'mintBatch',
  new Args().addString(me).addArray([1n, 2n], ArrayTypes.U256).addArray([5n, 6n], ArrayTypes.U256).addUint8Array(new Uint8Array()),
  Mas.fromString('0.05'),
);
await call(
  contract,
  'safeTransferFrom',
  new Args().addString(me).addString(other).addU256(7n).addU256(40n).addUint8Array(new Uint8Array()),
  Mas.fromString('0.02'),
);
const balances = new Args(
  (await read(contract, 'balanceOfBatch', new Args().addArray([me, other], ArrayTypes.STRING).addArray([7n, 7n], ArrayTypes.U256))).value,
);
console.log(`balances of id 7: me=${balances.nextU256()} other=${balances.nextU256()}`);
await call(contract, 'set_token_uri', new Args().addU256(7n).addString('7.json'), Mas.fromString('0.01'));
await call(contract, 'set_base_uri', new Args().addString('https://rust-1155.example/'), Mas.fromString('0.01'));
console.log(`uri(7) = ${bytesToStr((await read(contract, 'uri', new Args().addU256(7n))).value)}`);
console.log(`uri(1) = ${bytesToStr((await read(contract, 'uri', new Args().addU256(1n))).value)}`);
await call(contract, 'burn', new Args().addString(me).addU256(1n).addU256(5n));
console.log(`balanceOf(me, 1) = ${U256.fromBytes((await read(contract, 'balanceOf', new Args().addString(me).addU256(1n))).value)}`);
