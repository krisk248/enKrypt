// Small formatting / file helpers.

/** Group a fingerprint hex string into 4-char blocks for readability. */
export function formatFingerprint(fpr: string): string {
  return (fpr.match(/.{1,4}/g) ?? []).join(' ');
}

/** Short 8-hex key id (the trailing octets), for compact display. */
export function shortId(keyId: string): string {
  return keyId.slice(-8);
}

export function formatDate(unixSeconds: number): string {
  if (!unixSeconds) return '—';
  return new Date(unixSeconds * 1000).toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}

export function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / (1024 * 1024)).toFixed(1)} MB`;
}

/** Parse a User ID string "Name (comment) <email>" into parts. */
export function parseUserId(uid: string): { name: string; email: string; comment?: string } {
  const m = uid.match(/^([^(<]*?)\s*(?:\(([^)]*)\))?\s*(?:<([^>]*)>)?\s*$/);
  return {
    name: (m?.[1] ?? uid).trim(),
    comment: m?.[2]?.trim() || undefined,
    email: (m?.[3] ?? '').trim(),
  };
}

/** Trigger a browser download for the given bytes/text via a Blob URL. */
export function download(filename: string, data: Uint8Array | string, mime = 'application/octet-stream') {
  const blob =
    typeof data === 'string' ? new Blob([data], { type: mime }) : new Blob([data as BlobPart], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

/** Best-effort output filename for a decrypted file: strip .gpg/.pgp/.asc. */
export function stripEncryptedExt(name: string): string {
  return name.replace(/\.(gpg|pgp|asc)$/i, '') || 'decrypted.bin';
}

export async function readFileBytes(file: File): Promise<Uint8Array> {
  return new Uint8Array(await file.arrayBuffer());
}

export async function copyToClipboard(text: string): Promise<void> {
  await navigator.clipboard.writeText(text);
}

export async function readClipboard(): Promise<string> {
  return navigator.clipboard.readText();
}
