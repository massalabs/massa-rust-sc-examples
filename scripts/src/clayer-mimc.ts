// clayer-mimc: C-Layer only (needs the `assembly_script_hash_mimc` host function).
// Point JSON_RPC_URL at a C-Layer node (e.g. a local sandbox: http://127.0.0.1:23035).
import { Args, Mas, strToBytes } from '@massalabs/massa-web3';
import { call, deploy, getProvider, hex, read } from './utils.js';

const provider = await getProvider();
const contract = await deploy(provider, 'clayer_mimc');

const h1 = (await read(contract, 'mimc', new Args().addUint8Array(strToBytes('Hello World')))).value;
console.log(`mimc("Hello World") = ${hex(h1)}`);
console.log('expected              = 25097868b27b93cd01359a9b5f7e2a733e88b60ce32d1699b4e97b65ce87a2b8');
const h2 = (await read(contract, 'mimc_chain', new Args().addUint8Array(h1).addUint8Array(h1))).value;
console.log(`mimc(h1 || h1) = ${hex(h2)}`);
console.log('expected       = 1f0dd40c969d90a1f9155e51357bacea9740f98695b3f1f13125801680e9d033');

// Writing the commitment creates a datastore entry: send coins to pay its storage.
await call(contract, 'commit', new Args().addUint8Array(strToBytes('my secret')), Mas.fromString('0.01'));
const revealed = (await read(contract, 'reveal', new Args().addUint8Array(strToBytes('my secret')))).value[0];
console.log(`reveal(secret) = ${revealed === 1}`);
