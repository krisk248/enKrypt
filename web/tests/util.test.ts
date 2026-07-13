import { describe, it, expect } from 'vitest';
import { formatFingerprint, shortId, parseUserId, stripEncryptedExt } from '../src/lib/util/format';
import { estimateStrength } from '../src/lib/util/passphrase';

describe('format helpers', () => {
  it('groups a fingerprint into 4-char blocks', () => {
    expect(formatFingerprint('ABCD1234EF567890')).toBe('ABCD 1234 EF56 7890');
  });

  it('takes the short 8-hex key id', () => {
    expect(shortId('4F2E4916D371D530')).toBe('D371D530');
  });

  it('parses a full user id into parts', () => {
    expect(parseUserId('Ada Lovelace (work) <ada@example.com>')).toEqual({
      name: 'Ada Lovelace',
      comment: 'work',
      email: 'ada@example.com',
    });
  });

  it('parses a user id without a comment', () => {
    const p = parseUserId('Bob <bob@x.io>');
    expect(p.name).toBe('Bob');
    expect(p.email).toBe('bob@x.io');
    expect(p.comment).toBeUndefined();
  });

  it('strips OpenPGP extensions for decrypted output names', () => {
    expect(stripEncryptedExt('report.pdf.gpg')).toBe('report.pdf');
    expect(stripEncryptedExt('note.txt.asc')).toBe('note.txt');
    expect(stripEncryptedExt('plain.bin')).toBe('plain.bin');
  });
});

describe('passphrase strength', () => {
  it('scores an empty passphrase as 0', () => {
    expect(estimateStrength('').score).toBe(0);
  });

  it('rates a short simple passphrase as weak', () => {
    expect(estimateStrength('abc').score).toBeLessThanOrEqual(1);
  });

  it('rates a long mixed passphrase as strong', () => {
    expect(estimateStrength('Tr0ub4dor&3-correct-horse').score).toBeGreaterThanOrEqual(3);
  });

  it('penalises common passwords', () => {
    expect(estimateStrength('password123').score).toBeLessThanOrEqual(1);
  });
});
