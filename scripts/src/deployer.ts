// deployer: run the Rust deployer as an ExecuteSC with two contracts in the
// operation datastore (the same layout massa-web3 uses with the AS deployer).
import { Args, Mas, StorageCost, strToBytes, U64 } from '@massalabs/massa-web3';
import { getProvider, wasm } from './utils.js';

const provider = await getProvider();
const counter = wasm('counter');
const hello = wasm('hello_world');

const datastore = new Map<Uint8Array, Uint8Array>();
datastore.set(new Uint8Array([0]), new Args().addU64(2n).serialize());
// contract 1: counter(initial = 7, step = 2) with 0.01 MAS sent to the constructor
datastore.set(new Args().addU64(1n).serialize(), counter);
datastore.set(new Args().addU64(1n).addUint8Array(new Uint8Array([0])).serialize(), new Args().addU64(7n).addU64(2n).serialize());
datastore.set(new Args().addU64(1n).addUint8Array(new Uint8Array([1])).serialize(), new Args().addU64(Mas.fromString('0.01')).serialize());
// contract 2: hello-world, no constructor arguments
datastore.set(new Args().addU64(2n).serialize(), hello);

const maxCoins =
  StorageCost.smartContractDeploy(counter.length) + StorageCost.smartContractDeploy(hello.length) + Mas.fromString('0.1');
const op = await provider.executeSC({ byteCode: wasm('deployer'), datastore, maxCoins });
await op.waitSpeculativeExecution();
for (const e of await op.getSpeculativeEvents()) console.log(`  event: ${e.data}`);
