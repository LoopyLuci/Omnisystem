/**
 * Semantic-version comparison used by the updater UI ($lib/panels/UpdatePanel.svelte
 * and $lib/stores/updater.ts) to decide whether a manifest returned by
 * `@tauri-apps/plugin-updater`'s `check()` actually represents a newer build.
 *
 * The plugin itself already gates `check().available` on a semver comparison
 * against `latest.json`, so in the normal path this module is a defensive
 * second check / display helper rather than the sole gate — but it's kept
 * as plain, dependency-free logic specifically so it's unit-testable without
 * touching the network or Tauri's IPC bridge (see version.test.ts).
 */

export interface ParsedVersion {
  major: number;
  minor: number;
  patch: number;
  /** Pre-release identifiers, e.g. ["beta", "1"] for "1.2.0-beta.1". Empty for a stable release. */
  prerelease: string[];
}

/**
 * Parse a semver-ish version string ("1.2.3", "v1.2.3", "1.2.3-beta.1").
 * Returns null for strings that don't look like a version at all.
 */
export function parseVersion(raw: string): ParsedVersion | null {
  if (!raw) return null;
  const trimmed = raw.trim().replace(/^v/i, '');
  const match = trimmed.match(
    /^(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z-.]+))?(?:\+[0-9A-Za-z-.]+)?$/
  );
  if (!match) return null;
  const [, major, minor, patch, pre] = match;
  return {
    major: Number(major),
    minor: Number(minor),
    patch: Number(patch),
    prerelease: pre ? pre.split('.') : [],
  };
}

function comparePrereleaseIdentifier(a: string, b: string): number {
  const numA = /^\d+$/.test(a) ? Number(a) : null;
  const numB = /^\d+$/.test(b) ? Number(b) : null;
  if (numA !== null && numB !== null) return numA - numB;
  if (numA !== null) return -1; // numeric identifiers sort before alphanumeric
  if (numB !== null) return 1;
  return a < b ? -1 : a > b ? 1 : 0;
}

/**
 * Compare two semver strings. Returns:
 *   negative if a < b, 0 if equal, positive if a > b.
 * Unparsable strings sort as equal to everything (never claims an update from garbage input).
 * Follows semver precedence rules: a version without a pre-release is greater
 * than one with the same major.minor.patch and a pre-release.
 */
export function compareVersions(a: string, b: string): number {
  const va = parseVersion(a);
  const vb = parseVersion(b);
  if (!va || !vb) return 0;

  if (va.major !== vb.major) return va.major - vb.major;
  if (va.minor !== vb.minor) return va.minor - vb.minor;
  if (va.patch !== vb.patch) return va.patch - vb.patch;

  const aHasPre = va.prerelease.length > 0;
  const bHasPre = vb.prerelease.length > 0;
  if (aHasPre && !bHasPre) return -1;
  if (!aHasPre && bHasPre) return 1;
  if (!aHasPre && !bHasPre) return 0;

  const len = Math.max(va.prerelease.length, vb.prerelease.length);
  for (let i = 0; i < len; i++) {
    if (va.prerelease[i] === undefined) return -1;
    if (vb.prerelease[i] === undefined) return 1;
    const cmp = comparePrereleaseIdentifier(va.prerelease[i], vb.prerelease[i]);
    if (cmp !== 0) return cmp;
  }
  return 0;
}

/** True if `candidate` is a strictly newer version than `current`. */
export function isNewerVersion(current: string, candidate: string): boolean {
  return compareVersions(candidate, current) > 0;
}

/** Format bytes for a download-progress readout ("12.4 MB"). */
export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return '0 B';
  if (bytes < 1024) return `${bytes} B`;
  const units = ['KB', 'MB', 'GB'];
  let value = bytes / 1024;
  let unitIndex = 0;
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex++;
  }
  return `${value.toFixed(1)} ${units[unitIndex]}`;
}
