// vault: cross-contract calls. Deploys a token and a vault, deposits through
// transferFrom (allowance), then lets the vault deploy its embedded counter.
import { Args, Mas, MRC20, U256, U64 } from '@massalabs/massa-web3';
import { call, deploy, getProvider, read } from './utils.js';

const provider = await getProvider();
const me = provider.address;
const token = await deploy(provider, 'token', new Args().addString('Vault Token').addString('VLT').addU8(9n).addU256(1_000_000n));
const vault = await deploy(provider, 'vault', new Args().addString(token.address));
const mrc20 = new MRC20(provider, token.address);

// Allow the vault to pull 500 tokens, then deposit 300 (coins cover the token's storage).
await (await mrc20.increaseAllowance(vault.address, 500n)).waitSpeculativeExecution();
await call(vault, 'deposit', new Args().addU256(300n), Mas.fromString('0.05'));
console.log(`vault balance of me: ${U256.fromBytes((await read(vault, 'balance_of', new Args().addString(me))).value)}`);
console.log(`token balance of vault: ${await mrc20.balanceOf(vault.address)}`);
await call(vault, 'withdraw', new Args().addU256(100n), Mas.fromString('0.05'));
console.log(`token balance of me: ${await mrc20.balanceOf(me)}`);

// The vault creates a child contract from the bytecode it embeds (a Rust contract deploying a Rust
// contract). The coins pay the child's storage (0.0001 MAS per byte, about 2 MAS); the rest is forwarded.
await call(vault, 'spawn_counter', new Args().addU64(40n), Mas.fromString('2.5'));
await call(vault, 'bump_counter');
console.log(`child counter (via the vault): ${U64.fromBytes((await read(vault, 'bump_counter')).value)}`);
console.log(`local_call of the child's get in the vault's context: ${U64.fromBytes((await read(vault, 'peek_counter')).value)}`);
