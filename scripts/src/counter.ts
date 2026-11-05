// counter: deploy with (initial, step), increment, read `get` and the `status` struct.
import { Args, U64 } from '@massalabs/massa-web3';
import { call, deploy, getProvider, read } from './utils.js';

const provider = await getProvider();
const contract = await deploy(provider, 'counter', new Args().addU64(10n).addU64(5n));

await call(contract, 'increment');
await call(contract, 'increment', new Args().addU64(3n));

const value = U64.fromBytes((await read(contract, 'get')).value);
console.log(`counter = ${value}`);

// `status` returns a #[derive(Serializable)] struct: fields in order.
const status = new Args((await read(contract, 'status')).value);
console.log('status:', {
  value: status.nextU64(),
  step: status.nextU64(),
  owner: status.nextString(),
  lastCaller: status.nextBool() ? status.nextString() : null,
});

await call(contract, 'set_step', new Args().addU64(100n));
await call(contract, 'reset');
console.log(`counter after reset = ${U64.fromBytes((await read(contract, 'get')).value)}`);
