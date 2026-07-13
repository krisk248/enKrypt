/// <reference lib="webworker" />
//
// Crypto worker: owns the WebAssembly instance and runs every OpenPGP operation
// off the main thread so the UI never freezes during keygen or large-file work.
// Communicates via a tiny typed request/response protocol (see ./rpc.ts).

import init, * as wasm from '../../wasm/enkrypt_core.js';
import type { RpcRequest, RpcResponse } from './protocol';

// Instantiate the wasm module once; every request awaits this.
const ready: Promise<void> = init().then(() => {
  wasm.start(); // install the panic hook
});

function normalizeError(err: unknown): { code: string; message: string } {
  if (err && typeof err === 'object' && 'code' in err && 'message' in err) {
    return {
      code: String((err as { code: unknown }).code),
      message: String((err as { message: unknown }).message),
    };
  }
  return { code: 'internal', message: err instanceof Error ? err.message : String(err) };
}

/** Collect transferable ArrayBuffers from a result to avoid a copy back. */
function collectTransfer(result: unknown): Transferable[] {
  if (result instanceof Uint8Array) return [result.buffer];
  if (result && typeof result === 'object' && 'data' in result) {
    const d = (result as { data: unknown }).data;
    if (d instanceof Uint8Array) return [d.buffer];
  }
  return [];
}

self.onmessage = async (event: MessageEvent<RpcRequest>) => {
  const { id, method, args } = event.data;
  try {
    await ready;
    const fn = (wasm as unknown as Record<string, (...a: unknown[]) => unknown>)[method];
    if (typeof fn !== 'function') {
      throw { code: 'internal', message: `Unknown method: ${method}` };
    }
    const result = fn(...args);
    const response: RpcResponse = { id, ok: true, result };
    (self as DedicatedWorkerGlobalScope).postMessage(response, collectTransfer(result));
  } catch (err) {
    const response: RpcResponse = { id, ok: false, error: normalizeError(err) };
    (self as DedicatedWorkerGlobalScope).postMessage(response);
  }
};
