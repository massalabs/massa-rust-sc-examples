// autonomous: start a self-rescheduling deferred call, watch it tick, stop it.
import { Args, bytesToStr, Mas, U64 } from '@massalabs/massa-web3';
import { call, deploy, getProvider, read, sleep } from './utils.js';

const provider = await getProvider();
// Booking deferred calls costs coins: fund the contract at deployment.
const contract = await deploy(provider, 'autonomous', new Args(), Mas.fromString('5'));

await call(contract, 'start', new Args().addU64(3n));
console.log(`next call id: ${bytesToStr((await read(contract, 'next_call_id')).value)}`);

for (let i = 0; i < 4; i++) {
  await sleep(16_000 * 3); // 3 periods of 16 s
  const ticks = U64.fromBytes((await read(contract, 'ticks')).value);
  console.log(`ticks so far: ${ticks}`);
}
await call(contract, 'stop');
console.log(`next call id after stop: "${bytesToStr((await read(contract, 'next_call_id')).value)}"`);
