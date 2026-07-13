// Tests for the typed worker RPC layer. The real Web Worker + wasm module is
// replaced with a mock that speaks the same request/response protocol, so we
// exercise the client-side plumbing: id matching, result resolution, and typed
// error rejection — without needing a browser.

import { describe, it, expect, vi, beforeAll } from 'vitest';

class MockWorker {
  onmessage: ((e: { data: unknown }) => void) | null = null;
  constructor(
    public url: unknown,
    public opts: unknown,
  ) {}

  postMessage(msg: { id: number; method: string; args: unknown[] }) {
    // Respond asynchronously, like a real worker.
    queueMicrotask(() => {
      let response: unknown;
      if (msg.method === 'version') {
        response = { id: msg.id, ok: true, result: 'enkrypt-core test' };
      } else if (msg.method === 'decrypt') {
        response = {
          id: msg.id,
          ok: false,
          error: { code: 'wrong_passphrase', message: 'bad pass' },
        };
      } else {
        // Echo the args back so we can assert id/argument routing.
        response = { id: msg.id, ok: true, result: msg.args };
      }
      this.onmessage?.({ data: response });
    });
  }
  terminate() {}
}

let cryptoApi: typeof import('../src/lib/worker/rpc').cryptoApi;
let CryptoError: typeof import('../src/lib/worker/rpc').CryptoError;

beforeAll(async () => {
  vi.stubGlobal('Worker', MockWorker as unknown as typeof Worker);
  const mod = await import('../src/lib/worker/rpc');
  cryptoApi = mod.cryptoApi;
  CryptoError = mod.CryptoError;
});

describe('worker RPC layer', () => {
  it('resolves a call with the worker result', async () => {
    await expect(cryptoApi.version()).resolves.toBe('enkrypt-core test');
  });

  it('rejects with a typed CryptoError carrying the machine-readable code', async () => {
    const err = await cryptoApi
      .decrypt(new Uint8Array([1, 2, 3]), ['sk'], 'nope', [])
      .catch((e) => e);
    expect(err).toBeInstanceOf(CryptoError);
    expect(err.code).toBe('wrong_passphrase');
    expect(err.message).toBe('bad pass');
  });

  it('routes concurrent calls to the correct promise (id matching)', async () => {
    const [v, decErr] = await Promise.all([
      cryptoApi.version(),
      cryptoApi.decrypt(new Uint8Array(), [], '', []).catch((e: InstanceType<typeof CryptoError>) => e.code),
    ]);
    expect(v).toBe('enkrypt-core test');
    expect(decErr).toBe('wrong_passphrase');
  });

  it('passes arguments through to the worker', async () => {
    // parseKey echoes [bytesCopy]; assert it round-trips as a Uint8Array.
    const result = (await cryptoApi.parseKey(new Uint8Array([9, 8, 7]))) as unknown as unknown[];
    expect(Array.isArray(result)).toBe(true);
    expect(Array.from(result[0] as Uint8Array)).toEqual([9, 8, 7]);
  });
});
