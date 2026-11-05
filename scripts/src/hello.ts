// hello-world: run `main` as an ExecuteSC operation, then deploy and call `hello`.
import { Args, bytesToStr, Mas } from '@massalabs/massa-web3';
import { call, deploy, getProvider, read, wasm } from './utils.js';

const provider = await getProvider();

// 1. ExecuteSC: the module runs once in the context of our account, nothing is deployed.
const op = await provider.executeSC({ byteCode: wasm('hello_world'), maxCoins: Mas.fromString('0.1') });
await op.waitSpeculativeExecution();
for (const e of await op.getSpeculativeEvents()) console.log(`  executeSC event: ${e.data}`);

// 2. Deploy, then call `hello` (write) and read it (read-only, free).
const contract = await deploy(provider, 'hello_world');
await call(contract, 'hello', new Args().addString('Massa'));
const result = await read(contract, 'hello', new Args().addString('read-only caller'));
console.log(`read result: ${bytesToStr(result.value)}`);
