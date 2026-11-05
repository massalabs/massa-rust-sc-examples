// crypto: hashes, Massa signature verification, EVM address recovery.
import { Args, bytesToStr, strToBytes } from '@massalabs/massa-web3';
import { deploy, getProvider, read } from './utils.js';

const provider = await getProvider();
const contract = await deploy(provider, 'crypto');

console.log(bytesToStr((await read(contract, 'hashes', new Args().addUint8Array(strToBytes('abc')))).value));

// Sign a message with our account and let the contract verify it.
const message = 'hello from rust';
const signature = await provider.account.sign(strToBytes(message));
const ok = (
  await read(
    contract,
    'verify_massa',
    new Args().addString(provider.account.publicKey.toString()).addString(message).addString(signature.toString()),
  )
).value[0];
console.log(`massa signature valid: ${ok === 1}`);

console.log(bytesToStr((await read(contract, 'address_info', new Args().addString(provider.address))).value));
console.log(bytesToStr((await read(contract, 'address_info', new Args().addString(contract.address))).value));
