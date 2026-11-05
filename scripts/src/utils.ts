import 'dotenv/config';
import { readFileSync } from 'node:fs';
import {
  Account,
  Args,
  JsonRpcProvider,
  Mas,
  Operation,
  OperationStatus,
  Provider,
  ReadSCData,
  SmartContract,
} from '@massalabs/massa-web3';

/** Provider for the account in `PRIVATE_KEY`, on `JSON_RPC_URL` (buildnet by default). */
export async function getProvider(): Promise<JsonRpcProvider> {
  const account = await Account.fromEnv();
  const url = process.env.JSON_RPC_URL ?? 'https://buildnet.massa.net/api/v2';
  const provider = JsonRpcProvider.fromRPCUrl(url, account);
  const balance = await provider.balance(false);
  console.log(`account ${account.address} on ${url}: ${Mas.toString(balance)} MAS`);
  return provider;
}

/** Bytecode produced by `cargo massa build` (`build/<crate>.wasm`). */
export function wasm(crate: string): Uint8Array {
  const path = new URL(`../../build/${crate.replace(/-/g, '_')}.wasm`, import.meta.url);
  try {
    return new Uint8Array(readFileSync(path));
  } catch {
    throw new Error(`${path.pathname} not found: run \`cargo massa build\` at the repository root first`);
  }
}

/**
 * Deploy a contract and wait for its *final* execution (a few slots), so that
 * subsequent reads of its datastore (e.g. massa-web3's MRC20 wrapper) see it.
 *
 * `coins` are sent to the new contract before its `constructor` runs: they pay
 * the storage of what the constructor writes (0.0001 MAS per byte) and remain
 * on the contract. 0.1 MAS covers every example here.
 */
export async function deploy(
  provider: Provider,
  crate: string,
  constructorArgs: Args = new Args(),
  coins: bigint = Mas.fromString('0.1'),
): Promise<SmartContract> {
  const bytecode = wasm(crate);
  console.log(`deploying ${crate} (${bytecode.length} bytes)...`);
  const contract = await SmartContract.deploy(provider, bytecode, constructorArgs, {
    coins,
    waitFinalExecution: true,
  });
  console.log(`  deployed at ${contract.address}`);
  return contract;
}

/** Call an entry point, wait for its speculative execution and print its events. */
export async function call(
  contract: SmartContract,
  func: string,
  args: Args | Uint8Array = new Args(),
  coins: bigint = 0n,
): Promise<Operation> {
  const op = await contract.call(func, args, { coins });
  const status = await op.waitSpeculativeExecution();
  const events = await op.getSpeculativeEvents();
  console.log(`call ${func} -> ${OperationStatus[status]} (${op.id})`);
  for (const e of events) {
    console.log(`  event: ${e.data}`);
  }
  if (status !== OperationStatus.SpeculativeSuccess) {
    throw new Error(`${func} failed`);
  }
  return op;
}

/** Read-only call; prints events and returns the raw result. */
export async function read(
  contract: SmartContract,
  func: string,
  args: Args | Uint8Array = new Args(),
): Promise<ReadSCData> {
  const result = await contract.read(func, args);
  if (result.info.error) {
    throw new Error(`read ${func} failed: ${result.info.error}`);
  }
  for (const e of result.info.events) {
    console.log(`  read ${func} event: ${e.data}`);
  }
  return result;
}

export function hex(bytes: Uint8Array): string {
  return Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('');
}

export const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
