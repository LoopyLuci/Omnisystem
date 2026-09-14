import { describe, it, expect } from 'vitest';
import { parseVersion, compareVersions, isNewerVersion, formatBytes } from './version';

describe('parseVersion', () => {
  it('parses a plain semver string', () => {
    expect(parseVersion('1.2.3')).toEqual({ major: 1, minor: 2, patch: 3, prerelease: [] });
  });

  it('strips a leading "v"', () => {
    expect(parseVersion('v0.2.0')).toEqual({ major: 0, minor: 2, patch: 0, prerelease: [] });
  });

  it('parses pre-release identifiers', () => {
    expect(parseVersion('1.2.3-beta.1')).toEqual({
      major: 1,
      minor: 2,
      patch: 3,
      prerelease: ['beta', '1'],
    });
  });

  it('ignores build metadata', () => {
    expect(parseVersion('1.2.3+build.5')).toEqual({ major: 1, minor: 2, patch: 3, prerelease: [] });
  });

  it('returns null for garbage input', () => {
    expect(parseVersion('not-a-version')).toBeNull();
    expect(parseVersion('')).toBeNull();
    expect(parseVersion('1.2')).toBeNull();
  });
});

describe('compareVersions', () => {
  it('orders by major/minor/patch', () => {
    expect(compareVersions('2.0.0', '1.9.9')).toBeGreaterThan(0);
    expect(compareVersions('1.2.0', '1.3.0')).toBeLessThan(0);
    expect(compareVersions('1.2.3', '1.2.3')).toBe(0);
    expect(compareVersions('1.2.4', '1.2.3')).toBeGreaterThan(0);
  });

  it('treats a stable release as newer than its own pre-release', () => {
    expect(compareVersions('1.0.0', '1.0.0-beta.1')).toBeGreaterThan(0);
    expect(compareVersions('1.0.0-beta.1', '1.0.0')).toBeLessThan(0);
  });

  it('orders pre-release identifiers numerically then lexically', () => {
    expect(compareVersions('1.0.0-beta.2', '1.0.0-beta.10')).toBeLessThan(0);
    expect(compareVersions('1.0.0-alpha', '1.0.0-beta')).toBeLessThan(0);
    expect(compareVersions('1.0.0-alpha', '1.0.0-alpha.1')).toBeLessThan(0);
  });

  it('treats unparsable versions as equal rather than throwing', () => {
    expect(compareVersions('garbage', '1.0.0')).toBe(0);
    expect(compareVersions('1.0.0', 'garbage')).toBe(0);
  });
});

describe('isNewerVersion', () => {
  it('detects a genuinely newer version', () => {
    expect(isNewerVersion('0.2.0', '0.3.0')).toBe(true);
    expect(isNewerVersion('0.2.0', '0.2.0')).toBe(false);
    expect(isNewerVersion('0.3.0', '0.2.0')).toBe(false);
  });

  it('never claims an update for equal or unparsable versions', () => {
    expect(isNewerVersion('0.2.0', '0.2.0')).toBe(false);
    expect(isNewerVersion('0.2.0', 'not-a-version')).toBe(false);
  });
});

describe('formatBytes', () => {
  it('formats sub-kilobyte sizes as bytes', () => {
    expect(formatBytes(512)).toBe('512 B');
    expect(formatBytes(0)).toBe('0 B');
  });

  it('formats larger sizes with the right unit', () => {
    expect(formatBytes(1536)).toBe('1.5 KB');
    expect(formatBytes(12_400_000)).toBe('11.8 MB');
    expect(formatBytes(2_147_483_648)).toBe('2.0 GB');
  });

  it('handles invalid input defensively', () => {
    expect(formatBytes(-5)).toBe('0 B');
    expect(formatBytes(NaN)).toBe('0 B');
  });
});
