/**
 * Update-check state machine backing $lib/panels/UpdatePanel.svelte.
 *
 * Kept separate from the panel component so the state transitions (idle →
 * checking → available/up-to-date/error → downloading → ready-to-restart)
 * are unit-testable without mounting Svelte or touching Tauri's IPC bridge —
 * see updater.test.ts, which drives this store against a fake `check()`.
 *
 * The real `check()` / `Update.downloadAndInstall()` calls come from
 * `@tauri-apps/plugin-updater` and only work inside the Tauri webview; the
 * store takes them as injectable functions (default: the real plugin) so
 * tests can swap in a fake without mocking module resolution.
 */
import { writable, get } from 'svelte/store';

export type UpdateStatus =
  | 'idle'
  | 'checking'
  | 'up-to-date'
  | 'available'
  | 'downloading'
  | 'ready-to-restart'
  | 'error';

export interface UpdateState {
  status: UpdateStatus;
  currentVersion: string | null;
  availableVersion: string | null;
  releaseNotes: string | null;
  /** 0-100, only meaningful while status === 'downloading'. */
  downloadProgress: number;
  downloadedBytes: number;
  totalBytes: number | null;
  error: string | null;
  lastCheckedAt: number | null;
}

const initialState: UpdateState = {
  status: 'idle',
  currentVersion: null,
  availableVersion: null,
  releaseNotes: null,
  downloadProgress: 0,
  downloadedBytes: 0,
  totalBytes: null,
  error: null,
  lastCheckedAt: null,
};

export const updateState = writable<UpdateState>({ ...initialState });

function setState(patch: Partial<UpdateState>) {
  updateState.update((s) => ({ ...s, ...patch }));
}

export function resetUpdateState() {
  updateState.set({ ...initialState });
}

/** Minimal shape of the object `@tauri-apps/plugin-updater`'s `check()` resolves to. */
export interface UpdateLike {
  version: string;
  currentVersion: string;
  body?: string | null;
  downloadAndInstall(
    onEvent?: (event: { event: string; data?: { contentLength?: number; chunkLength?: number } }) => void
  ): Promise<void>;
}

export type CheckFn = () => Promise<UpdateLike | null>;

/**
 * Run an update check using the given `checkFn` (defaults to the real
 * `@tauri-apps/plugin-updater` `check()`, lazily imported so this module —
 * and anything that imports it in a test — never touches Tauri's IPC bridge
 * unless a check is actually triggered from inside the webview).
 */
export async function runUpdateCheck(checkFn?: CheckFn): Promise<void> {
  setState({ status: 'checking', error: null });
  try {
    const check = checkFn ?? (await defaultCheckFn());
    const update = await check();
    if (update) {
      setState({
        status: 'available',
        currentVersion: update.currentVersion,
        availableVersion: update.version,
        releaseNotes: update.body ?? null,
        lastCheckedAt: Date.now(),
      });
      pendingUpdate = update;
    } else {
      setState({ status: 'up-to-date', lastCheckedAt: Date.now(), error: null });
      pendingUpdate = null;
    }
  } catch (e) {
    setState({ status: 'error', error: e instanceof Error ? e.message : String(e) });
    pendingUpdate = null;
  }
}

let pendingUpdate: UpdateLike | null = null;

async function defaultCheckFn(): Promise<CheckFn> {
  const { check } = await import('@tauri-apps/plugin-updater');
  return check;
}

/**
 * Download and install the update found by the most recent `runUpdateCheck()`.
 * Reports progress into `updateState.downloadProgress` as chunks arrive.
 */
export async function downloadAndInstallUpdate(): Promise<void> {
  const current = get(updateState);
  if (current.status !== 'available' || !pendingUpdate) {
    setState({ status: 'error', error: 'No update available to install.' });
    return;
  }

  setState({ status: 'downloading', downloadProgress: 0, downloadedBytes: 0, totalBytes: null, error: null });
  try {
    let downloaded = 0;
    let total: number | null = null;
    await pendingUpdate.downloadAndInstall((event) => {
      if (event.event === 'Started') {
        total = event.data?.contentLength ?? null;
        setState({ totalBytes: total });
      } else if (event.event === 'Progress') {
        downloaded += event.data?.chunkLength ?? 0;
        const progress = total ? Math.min(100, Math.round((downloaded / total) * 100)) : 0;
        setState({ downloadedBytes: downloaded, downloadProgress: progress });
      } else if (event.event === 'Finished') {
        setState({ downloadProgress: 100 });
      }
    });
    setState({ status: 'ready-to-restart' });
  } catch (e) {
    setState({ status: 'error', error: e instanceof Error ? e.message : String(e) });
  }
}

/** Restart the app to apply a downloaded update ("Restart to apply" button). */
export async function restartToApplyUpdate(relaunchFn?: () => Promise<void>): Promise<void> {
  const relaunch = relaunchFn ?? (await defaultRelaunchFn());
  await relaunch();
}

async function defaultRelaunchFn(): Promise<() => Promise<void>> {
  const { relaunch } = await import('@tauri-apps/plugin-process');
  return relaunch;
}
