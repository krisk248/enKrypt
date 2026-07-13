// IndexedDB persistence via `idb`. Two persistence shapes are supported:
//   * plain persistent  — each key row stored individually (secret keys remain
//                          OpenPGP S2K-encrypted armored text; never plaintext).
//   * vault persistent   — the whole keyring is serialised and sealed with
//                          AES-256-GCM (Argon2id) into a single blob.
// Session mode persists nothing.

import { openDB, type IDBPDatabase } from 'idb';
import type { KeyInfo } from '../../wasm/enkrypt_core';

export interface StoredKey {
  fingerprint: string;
  info: KeyInfo;
  publicKey: string;
  secretKey?: string;
  addedAt: number;
  /** App-level expiry (unix seconds); rpgp 0.20 cannot embed key expiry. */
  expiryOverride?: number;
  revoked?: boolean;
}

export type StorageMode = 'session' | 'persistent';

export interface AppMeta {
  storageMode: StorageMode;
  defaultKeyFpr?: string;
  vaultEnabled: boolean;
}

const DB_NAME = 'enkrypt';
const DB_VERSION = 1;

let dbp: Promise<IDBPDatabase> | null = null;

function db() {
  if (!dbp) {
    dbp = openDB(DB_NAME, DB_VERSION, {
      upgrade(database) {
        if (!database.objectStoreNames.contains('keys')) {
          database.createObjectStore('keys', { keyPath: 'fingerprint' });
        }
        if (!database.objectStoreNames.contains('meta')) {
          database.createObjectStore('meta');
        }
      },
    });
  }
  return dbp;
}

export async function loadMeta(): Promise<AppMeta | undefined> {
  return (await db()).get('meta', 'app');
}

export async function saveMeta(meta: AppMeta): Promise<void> {
  await (await db()).put('meta', meta, 'app');
}

export async function loadPlainKeys(): Promise<StoredKey[]> {
  return (await db()).getAll('keys');
}

export async function savePlainKey(key: StoredKey): Promise<void> {
  await (await db()).put('keys', key);
}

export async function deletePlainKey(fingerprint: string): Promise<void> {
  await (await db()).delete('keys', fingerprint);
}

export async function clearPlainKeys(): Promise<void> {
  await (await db()).clear('keys');
}

export async function saveVaultBlob(blob: Uint8Array): Promise<void> {
  await (await db()).put('meta', blob, 'vaultBlob');
}

export async function loadVaultBlob(): Promise<Uint8Array | undefined> {
  return (await db()).get('meta', 'vaultBlob');
}

export async function clearVaultBlob(): Promise<void> {
  await (await db()).delete('meta', 'vaultBlob');
}

/** Nuke everything: all keys, meta and the vault blob. */
export async function wipeAll(): Promise<void> {
  const database = await db();
  await database.clear('keys');
  await database.clear('meta');
}
