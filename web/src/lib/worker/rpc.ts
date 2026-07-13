// Typed client-side wrapper around the crypto worker. Every method returns a
// Promise and rejects with a structured { code, message } CryptoError.

import type {
  GenerateOptions,
  KeyBundle,
  KeyInfo,
  DecryptResult,
  VerifyResult,
  CleartextVerifyResult,
} from '../../wasm/enkrypt_core';
import type { RpcRequest, RpcResponse, RpcError } from './protocol';

export class CryptoError extends Error {
  code: string;
  constructor(err: RpcError) {
    super(err.message);
    this.name = 'CryptoError';
    this.code = err.code;
  }
}

const worker = new Worker(new URL('./crypto.worker.ts', import.meta.url), {
  type: 'module',
  name: 'enkrypt-crypto',
});

let seq = 0;
const pending = new Map<
  number,
  { resolve: (v: unknown) => void; reject: (e: CryptoError) => void }
>();

worker.onmessage = (event: MessageEvent<RpcResponse>) => {
  const msg = event.data;
  const entry = pending.get(msg.id);
  if (!entry) return;
  pending.delete(msg.id);
  if (msg.ok) entry.resolve(msg.result);
  else entry.reject(new CryptoError(msg.error));
};

function call<T>(method: string, args: unknown[], transfer: Transferable[] = []): Promise<T> {
  return new Promise<T>((resolve, reject) => {
    const id = ++seq;
    pending.set(id, {
      resolve: resolve as (v: unknown) => void,
      reject,
    });
    const req: RpcRequest = { id, method, args };
    worker.postMessage(req, transfer);
  });
}

// Buffers we transfer are detached on our side, so always pass a fresh copy
// when the caller may still need the original bytes.
const copy = (u: Uint8Array) => u.slice();

export const cryptoApi = {
  version: () => call<string>('version', []),

  generateKey: (opts: GenerateOptions) => call<KeyBundle>('generate_key', [opts]),

  parseKey: (bytes: Uint8Array) => {
    const b = copy(bytes);
    return call<KeyInfo>('parse_key', [b], [b.buffer]);
  },

  exportPublic: (bytes: Uint8Array) => {
    const b = copy(bytes);
    return call<string>('export_public', [b], [b.buffer]);
  },

  exportSecret: (bytes: Uint8Array) => {
    const b = copy(bytes);
    return call<string>('export_secret', [b], [b.buffer]);
  },

  generateRevocation: (
    secretKey: Uint8Array,
    passphrase: string,
    reasonCode: number,
    reason: string,
  ) => {
    const b = copy(secretKey);
    return call<string>('generate_revocation', [b, passphrase, reasonCode, reason], [b.buffer]);
  },

  encrypt: (
    data: Uint8Array,
    recipients: string[],
    signWith: string | null,
    signPassphrase: string | null,
    armor: boolean,
  ) => {
    const b = copy(data);
    return call<Uint8Array>('encrypt', [b, recipients, signWith, signPassphrase, armor], [b.buffer]);
  },

  decrypt: (data: Uint8Array, secretKeys: string[], passphrase: string, verifyKeys: string[]) => {
    const b = copy(data);
    return call<DecryptResult>('decrypt', [b, secretKeys, passphrase, verifyKeys], [b.buffer]);
  },

  signDetached: (data: Uint8Array, secretKey: string, passphrase: string, armor: boolean) => {
    const b = copy(data);
    return call<Uint8Array>('sign_detached', [b, secretKey, passphrase, armor], [b.buffer]);
  },

  verifyDetached: (data: Uint8Array, signature: Uint8Array, publicKeys: string[]) => {
    const b = copy(data);
    const s = copy(signature);
    return call<VerifyResult>('verify_detached', [b, s, publicKeys], [b.buffer, s.buffer]);
  },

  signCleartext: (text: string, secretKey: string, passphrase: string) =>
    call<string>('sign_cleartext', [text, secretKey, passphrase]),

  verifyCleartext: (armored: string, publicKeys: string[]) =>
    call<CleartextVerifyResult>('verify_cleartext', [armored, publicKeys]),

  vaultSeal: (data: Uint8Array, passphrase: string) => {
    const b = copy(data);
    return call<Uint8Array>('vault_seal', [b, passphrase], [b.buffer]);
  },

  vaultOpen: (blob: Uint8Array, passphrase: string) => {
    const b = copy(blob);
    return call<Uint8Array>('vault_open', [b, passphrase], [b.buffer]);
  },
};

export type CryptoApi = typeof cryptoApi;
