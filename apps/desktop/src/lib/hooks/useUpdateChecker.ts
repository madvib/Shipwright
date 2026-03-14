import { useEffect } from 'react';
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { ask } from '@tauri-apps/plugin-dialog';

export function useUpdateChecker() {
  useEffect(() => {
    // Only run in production Tauri context
    if (!('__TAURI_INTERNALS__' in window)) return;

    const checkForUpdate = async () => {
      try {
        const update = await check();
        if (!update?.available) return;

        const yes = await ask(
          `Ship ${update.version} is available.\n\nInstall now and restart?`,
          { title: 'Update Available', kind: 'info' },
        );

        if (yes) {
          await update.downloadAndInstall();
          await relaunch();
        }
      } catch {
        // Silently ignore update errors — don't interrupt the user
      }
    };

    // Short delay so the app renders first
    const timer = setTimeout(() => void checkForUpdate(), 3000);
    return () => clearTimeout(timer);
  }, []);
}
