// @vitest-environment jsdom
import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import {
  updateState,
  resetUpdateState,
  runUpdateCheck,
  downloadAndInstallUpdate,
  restartToApplyUpdate,
  type UpdateLike,
} from './updater';

describe('updater store', () => {
  beforeEach(() => {
    resetUpdateState();
  });

  it('starts idle', () => {
    expect(get(updateState).status).toBe('idle');
  });

  it('transitions to up-to-date when check() resolves null', async () => {
    await runUpdateCheck(async () => null);
    const s = get(updateState);
    expect(s.status).toBe('up-to-date');
    expect(s.error).toBeNull();
    expect(s.lastCheckedAt).not.toBeNull();
  });

  it('transitions to available with version/changelog info when an update exists', async () => {
    const fakeUpdate: UpdateLike = {
      version: '0.3.0',
      currentVersion: '0.2.0',
      body: 'Fixed things.',
      downloadAndInstall: async () => {},
    };
    await runUpdateCheck(async () => fakeUpdate);
    const s = get(updateState);
    expect(s.status).toBe('available');
    expect(s.availableVersion).toBe('0.3.0');
    expect(s.currentVersion).toBe('0.2.0');
    expect(s.releaseNotes).toBe('Fixed things.');
  });

  it('transitions to error when check() throws', async () => {
    await runUpdateCheck(async () => {
      throw new Error('network unreachable');
    });
    const s = get(updateState);
    expect(s.status).toBe('error');
    expect(s.error).toBe('network unreachable');
  });

  it('refuses to download when no update is pending', async () => {
    await downloadAndInstallUpdate();
    const s = get(updateState);
    expect(s.status).toBe('error');
    expect(s.error).toMatch(/no update/i);
  });

  it('tracks download progress and ends ready-to-restart', async () => {
    const fakeUpdate: UpdateLike = {
      version: '0.3.0',
      currentVersion: '0.2.0',
      body: null,
      downloadAndInstall: async (onEvent) => {
        onEvent?.({ event: 'Started', data: { contentLength: 1000 } });
        onEvent?.({ event: 'Progress', data: { chunkLength: 400 } });
        onEvent?.({ event: 'Progress', data: { chunkLength: 600 } });
        onEvent?.({ event: 'Finished' });
      },
    };
    await runUpdateCheck(async () => fakeUpdate);
    await downloadAndInstallUpdate();
    const s = get(updateState);
    expect(s.status).toBe('ready-to-restart');
    expect(s.downloadProgress).toBe(100);
    expect(s.totalBytes).toBe(1000);
    expect(s.downloadedBytes).toBe(1000);
  });

  it('surfaces a download failure as an error state', async () => {
    const fakeUpdate: UpdateLike = {
      version: '0.3.0',
      currentVersion: '0.2.0',
      body: null,
      downloadAndInstall: async () => {
        throw new Error('disk full');
      },
    };
    await runUpdateCheck(async () => fakeUpdate);
    await downloadAndInstallUpdate();
    const s = get(updateState);
    expect(s.status).toBe('error');
    expect(s.error).toBe('disk full');
  });

  it('calls the provided relaunch function to restart', async () => {
    let called = false;
    await restartToApplyUpdate(async () => {
      called = true;
    });
    expect(called).toBe(true);
  });
});
