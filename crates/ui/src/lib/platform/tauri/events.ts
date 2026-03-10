import { isTauriRuntime } from './runtime';

interface ProjectEventHandlers {
  onSpecsChanged?: () => void;
  onAdrsChanged?: () => void;
  onFeaturesChanged?: () => void;
  onReleasesChanged?: () => void;
  onNotesChanged?: () => void;
  onLogChanged?: () => void;
  onConfigChanged?: () => void;
  onEventsChanged?: () => void;
}

type UnlistenFn = () => void;

export async function subscribeProjectEvents(
  handlers: ProjectEventHandlers
): Promise<UnlistenFn> {
  if (!isTauriRuntime()) {
    return () => {};
  }

  const { listen } = await import('@tauri-apps/api/event');
  const unlistenFns: UnlistenFn[] = [];

  if (handlers.onSpecsChanged) {
    const unlisten = await listen('ship://specs-changed', () => {
      handlers.onSpecsChanged?.();
    });
    unlistenFns.push(unlisten);
  }

  if (handlers.onAdrsChanged) {
    const unlisten = await listen('ship://adrs-changed', () => {
      handlers.onAdrsChanged?.();
    });
    unlistenFns.push(unlisten);
  }

  if (handlers.onFeaturesChanged) {
    const unlisten = await listen('ship://features-changed', () => {
      handlers.onFeaturesChanged?.();
    });
    unlistenFns.push(unlisten);
  }

  if (handlers.onReleasesChanged) {
    const unlisten = await listen('ship://releases-changed', () => {
      handlers.onReleasesChanged?.();
    });
    unlistenFns.push(unlisten);
  }

  if (handlers.onNotesChanged) {
    const unlisten = await listen('ship://notes-changed', () => {
      handlers.onNotesChanged?.();
    });
    unlistenFns.push(unlisten);
  }

  if (handlers.onLogChanged) {
    const unlisten = await listen('ship://log-changed', () => {
      handlers.onLogChanged?.();
    });
    unlistenFns.push(unlisten);
  }

  if (handlers.onConfigChanged) {
    const unlisten = await listen('ship://config-changed', () => {
      handlers.onConfigChanged?.();
    });
    unlistenFns.push(unlisten);
  }

  if (handlers.onEventsChanged) {
    const unlisten = await listen('ship://events-changed', () => {
      handlers.onEventsChanged?.();
    });
    unlistenFns.push(unlisten);
  }

  return () => {
    for (const unlisten of unlistenFns) {
      unlisten();
    }
  };
}
