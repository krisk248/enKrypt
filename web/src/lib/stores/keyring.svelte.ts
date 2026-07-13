// The central keyring store: in-memory key list plus persistence orchestration
// for the three storage shapes (session / persistent / persistent+vault).

import { cryptoApi } from '../worker/rpc';
import type { KeyInfo, KeyBundle } from '../../wasm/enkrypt_core';
import {
  loadMeta,
  saveMeta,
  loadPlainKeys,
  clearPlainKeys,
  savePlainKey,
  saveVaultBlob,
  loadVaultBlob,
  clearVaultBlob,
  wipeAll,
  type StoredKey,
  type StorageMode,
  type AppMeta,
} from '../db/idb';

const enc = new TextEncoder();
const dec = new TextDecoder();

class KeyringStore {
  keys = $state<StoredKey[]>([]);
  defaultKeyFpr = $state<string | undefined>(undefined);
  storageMode = $state<StorageMode>('session');
  vaultEnabled = $state(false);

  /** True until the user has made the first-run storage choice. */
  needsFirstRun = $state(false);
  /** True when a vault exists but has not been unlocked this session. */
  locked = $state(false);
  ready = $state(false);

  // In-memory only, never persisted.
  private vaultPassphrase: string | null = null;
  private passphraseCache = new Map<string, string>();

  get secretKeys(): StoredKey[] {
    return this.keys.filter((k) => k.secretKey);
  }

  get defaultKey(): StoredKey | undefined {
    return this.keys.find((k) => k.fingerprint === this.defaultKeyFpr);
  }

  secretKeysArmored(): string[] {
    return this.secretKeys.map((k) => k.secretKey!) as string[];
  }
  publicKeysArmored(): string[] {
    return this.keys.map((k) => k.publicKey);
  }

  getKey(fpr: string): StoredKey | undefined {
    return this.keys.find((k) => k.fingerprint === fpr);
  }

  // --- lifecycle ----------------------------------------------------------

  async init() {
    const meta = await loadMeta();
    if (!meta) {
      this.needsFirstRun = true;
      this.ready = true;
      return;
    }
    this.storageMode = meta.storageMode;
    this.defaultKeyFpr = meta.defaultKeyFpr;
    this.vaultEnabled = meta.vaultEnabled;

    if (meta.storageMode === 'persistent') {
      if (meta.vaultEnabled) {
        this.locked = true; // needs unlock before keys load
      } else {
        this.keys = await loadPlainKeys();
      }
    }
    this.ready = true;
  }

  async chooseStorage(mode: StorageMode, vaultPassphrase?: string) {
    this.storageMode = mode;
    this.vaultEnabled = mode === 'persistent' && !!vaultPassphrase;
    this.vaultPassphrase = vaultPassphrase ?? null;
    this.needsFirstRun = false;
    this.locked = false;
    await this.saveMetaNow();
    await this.persist();
  }

  async unlockVault(passphrase: string) {
    const blob = await loadVaultBlob();
    if (!blob) {
      // No blob yet (empty vault) — just accept the passphrase.
      this.vaultPassphrase = passphrase;
      this.locked = false;
      return;
    }
    const plain = await cryptoApi.vaultOpen(blob, passphrase);
    this.keys = JSON.parse(dec.decode(plain)) as StoredKey[];
    this.vaultPassphrase = passphrase;
    this.locked = false;
  }

  // --- persistence --------------------------------------------------------

  private async saveMetaNow() {
    const meta: AppMeta = {
      storageMode: this.storageMode,
      defaultKeyFpr: this.defaultKeyFpr,
      vaultEnabled: this.vaultEnabled,
    };
    await saveMeta(meta);
  }

  /** Write the whole keyring to storage according to the current mode. */
  private async persist() {
    await this.saveMetaNow();
    if (this.storageMode === 'session') return;

    if (this.vaultEnabled) {
      if (!this.vaultPassphrase) return; // locked; cannot write
      const bytes = enc.encode(JSON.stringify(this.keys));
      const sealed = await cryptoApi.vaultSeal(bytes, this.vaultPassphrase);
      await saveVaultBlob(sealed);
    } else {
      await clearPlainKeys();
      for (const k of this.keys) await savePlainKey(k);
    }
  }

  // --- key operations -----------------------------------------------------

  private upsert(key: StoredKey) {
    const idx = this.keys.findIndex((k) => k.fingerprint === key.fingerprint);
    if (idx >= 0) {
      // Merge: keep secret material if the newcomer lacks it.
      const existing = this.keys[idx];
      this.keys[idx] = {
        ...existing,
        ...key,
        secretKey: key.secretKey ?? existing.secretKey,
      };
    } else {
      this.keys.push(key);
    }
    if (!this.defaultKeyFpr && key.secretKey) this.defaultKeyFpr = key.fingerprint;
  }

  async addGenerated(bundle: KeyBundle) {
    this.upsert({
      fingerprint: bundle.info.fingerprint,
      info: bundle.info,
      publicKey: bundle.public_key,
      secretKey: bundle.secret_key,
      addedAt: Date.now(),
    });
    await this.persist();
  }

  /** Parse arbitrary key bytes and return the metadata for a review dialog. */
  async inspect(bytes: Uint8Array): Promise<KeyInfo> {
    return cryptoApi.parseKey(bytes);
  }

  async importBytes(bytes: Uint8Array): Promise<StoredKey> {
    const info = await cryptoApi.parseKey(bytes);
    const publicKey = await cryptoApi.exportPublic(bytes);
    let secretKey: string | undefined;
    if (info.has_secret) {
      secretKey = await cryptoApi.exportSecret(bytes);
    }
    const stored: StoredKey = {
      fingerprint: info.fingerprint,
      info,
      publicKey,
      secretKey,
      addedAt: Date.now(),
    };
    this.upsert(stored);
    await this.persist();
    return stored;
  }

  async remove(fpr: string) {
    this.keys = this.keys.filter((k) => k.fingerprint !== fpr);
    if (this.defaultKeyFpr === fpr) this.defaultKeyFpr = this.secretKeys[0]?.fingerprint;
    this.passphraseCache.delete(fpr);
    await this.persist();
  }

  async setDefault(fpr: string) {
    this.defaultKeyFpr = fpr;
    await this.persist();
  }

  async markRevoked(fpr: string) {
    const k = this.getKey(fpr);
    if (k) {
      k.revoked = true;
      await this.persist();
    }
  }

  // --- settings changes ---------------------------------------------------

  async setStorageMode(mode: StorageMode) {
    if (mode === this.storageMode) return;
    this.storageMode = mode;
    if (mode === 'session') {
      this.vaultEnabled = false;
      this.vaultPassphrase = null;
      await clearVaultBlob();
      await clearPlainKeys();
    }
    await this.persist();
  }

  async enableVault(passphrase: string) {
    this.vaultEnabled = true;
    this.vaultPassphrase = passphrase;
    this.storageMode = 'persistent';
    await clearPlainKeys();
    await this.persist();
  }

  async disableVault() {
    this.vaultEnabled = false;
    this.vaultPassphrase = null;
    await clearVaultBlob();
    await this.persist();
  }

  // --- session passphrase cache ------------------------------------------

  cachePassphrase(fpr: string, pass: string) {
    this.passphraseCache.set(fpr, pass);
  }
  cachedPassphrase(fpr: string): string | undefined {
    return this.passphraseCache.get(fpr);
  }
  clearPassphraseCache() {
    this.passphraseCache.clear();
  }

  // --- danger zone --------------------------------------------------------

  async wipe() {
    await wipeAll();
    this.keys = [];
    this.defaultKeyFpr = undefined;
    this.vaultPassphrase = null;
    this.vaultEnabled = false;
    this.storageMode = 'session';
    this.passphraseCache.clear();
    this.locked = false;
    this.needsFirstRun = true;
  }
}

export const keyring = new KeyringStore();
