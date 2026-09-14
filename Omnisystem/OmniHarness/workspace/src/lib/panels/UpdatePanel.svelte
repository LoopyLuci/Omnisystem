<script lang="ts">
  import { onMount } from 'svelte';
  import { getVersion } from '@tauri-apps/api/app';
  import {
    updateState,
    runUpdateCheck,
    downloadAndInstallUpdate,
    restartToApplyUpdate,
  } from '$lib/stores/updater';
  import { formatBytes } from '$lib/utils/version';
  import { addToast } from '$lib/stores/toast';

  let installedVersion = '';

  onMount(async () => {
    try {
      installedVersion = await getVersion();
    } catch {
      installedVersion = '';
    }
  });

  async function checkNow() {
    await runUpdateCheck();
    const s = $updateState;
    if (s.status === 'up-to-date') {
      addToast("You're on the latest version.", 'success');
    } else if (s.status === 'error') {
      addToast(`Update check failed: ${s.error}`, 'error');
    }
  }

  async function install() {
    await downloadAndInstallUpdate();
    if ($updateState.status === 'error') {
      addToast(`Update failed: ${$updateState.error}`, 'error');
    }
  }

  async function restart() {
    try {
      await restartToApplyUpdate();
    } catch (e) {
      addToast(`Restart failed: ${e}`, 'error');
    }
  }
</script>

<div class="update-panel">
  <div class="panel-header">
    <h2>🔄 Updates</h2>
  </div>

  <div class="content">
    <div class="version-row">
      <span class="lbl">Installed version</span>
      <span class="val">{installedVersion || $updateState.currentVersion || '—'}</span>
    </div>

    {#if $updateState.status === 'idle'}
      <p class="hint">Check GitHub Releases for a newer build of Omnisystem Workspace.</p>
      <button class="btn-primary" on:click={checkNow}>Check for Updates</button>
    {:else if $updateState.status === 'checking'}
      <p class="hint">Checking for updates…</p>
    {:else if $updateState.status === 'up-to-date'}
      <p class="status-ok">✅ You're on the latest version.</p>
      <button class="btn-sm" on:click={checkNow}>Check Again</button>
    {:else if $updateState.status === 'available'}
      <div class="available-card">
        <p class="status-available">
          🆕 Version <strong>{$updateState.availableVersion}</strong> is available
          (you have {$updateState.currentVersion}).
        </p>
        {#if $updateState.releaseNotes}
          <div class="changelog">
            <h4>What's new</h4>
            <pre>{$updateState.releaseNotes}</pre>
          </div>
        {/if}
        <button class="btn-primary" on:click={install}>Download &amp; Install</button>
      </div>
    {:else if $updateState.status === 'downloading'}
      <div class="download-card">
        <p class="hint">Downloading update…</p>
        <div class="progress-track">
          <div class="progress-fill" style="width: {$updateState.downloadProgress}%"></div>
        </div>
        <p class="progress-label">
          {$updateState.downloadProgress}%
          {#if $updateState.totalBytes}
            ({formatBytes($updateState.downloadedBytes)} / {formatBytes($updateState.totalBytes)})
          {/if}
        </p>
      </div>
    {:else if $updateState.status === 'ready-to-restart'}
      <p class="status-ok">✅ Update downloaded. Restart to finish installing.</p>
      <button class="btn-primary" on:click={restart}>Restart to Apply</button>
    {:else if $updateState.status === 'error'}
      <p class="status-error">⚠ {$updateState.error}</p>
      <button class="btn-sm" on:click={checkNow}>Try Again</button>
    {/if}

    {#if $updateState.lastCheckedAt}
      <p class="last-checked">Last checked {new Date($updateState.lastCheckedAt).toLocaleString()}</p>
    {/if}

    <div class="platform-note">
      Omnisystem Workspace installers are currently Windows-only — native
      audio/PTY/GPU dependencies are only built and verified on Windows.
    </div>
  </div>
</div>

<style>
  .update-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #1a1a2e;
    color: #e0e0e0;
    font-family: 'Inter', system-ui, sans-serif;
    font-size: 13px;
  }

  .panel-header {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid #2a2a4a;
    flex-shrink: 0;
  }

  .panel-header h2 {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
    color: #c9b8ff;
  }

  .content {
    padding: 1.25rem;
    overflow-y: auto;
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
  }

  .version-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: #0e0e22;
    border: 1px solid #2a2a4a;
    border-radius: 6px;
    padding: 0.6rem 0.85rem;
    font-size: 12px;
  }
  .lbl { color: #888; }
  .val { color: #e0e0e0; font-weight: 600; }

  .hint { margin: 0; color: #aaa; font-size: 13px; }
  .last-checked { margin: 0; color: #666; font-size: 11px; }

  .status-ok { color: #7dff7d; margin: 0; }
  .status-available { color: #c9b8ff; margin: 0; }
  .status-error { color: #ff8888; margin: 0; }

  .btn-primary {
    background: #5a4abf;
    border: none;
    color: #fff;
    padding: 0.45rem 1rem;
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
    align-self: flex-start;
  }
  .btn-primary:hover { background: #7c5cbf; }

  .btn-sm {
    background: #2a2a4a;
    border: 1px solid #3a3a6a;
    color: #ccc;
    padding: 0.35rem 0.75rem;
    border-radius: 5px;
    cursor: pointer;
    font-size: 12px;
    align-self: flex-start;
  }
  .btn-sm:hover { background: #3a3a6a; color: #fff; }

  .available-card, .download-card {
    background: #0e0e22;
    border: 1px solid #2a2a4a;
    border-radius: 8px;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.7rem;
  }

  .changelog h4 { margin: 0 0 0.4rem; font-size: 12px; color: #c9b8ff; }
  .changelog pre {
    margin: 0;
    white-space: pre-wrap;
    font-family: 'Fira Code', monospace;
    font-size: 12px;
    color: #ccc;
    max-height: 200px;
    overflow-y: auto;
  }

  .progress-track {
    height: 8px;
    background: #2a2a4a;
    border-radius: 4px;
    overflow: hidden;
  }
  .progress-fill {
    height: 100%;
    background: #7c5cbf;
    transition: width 0.2s ease;
  }
  .progress-label { margin: 0; font-size: 11px; color: #888; }

  .platform-note {
    margin-top: auto;
    padding-top: 0.85rem;
    border-top: 1px solid #2a2a4a;
    font-size: 11px;
    color: #666;
    line-height: 1.5;
  }
</style>
