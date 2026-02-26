import { isTauriRuntime } from './runtime';

interface ProjectEventHandlers {
  onIssuesChanged?: () => void;
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

  if (handlers.onIssuesChanged) {
    const unlisten = await listen('ship://issues-changed', () => {
      handlers.onIssuesChanged?.();
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
