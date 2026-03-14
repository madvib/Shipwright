import { useEffect, useRef, useState } from 'react';
import { getRuntimePerfCmd, type RuntimePerfSnapshot } from '@/lib/platform/tauri/commands';

export function useRuntimePerf(enabled: boolean, intervalMs = 1200) {
  const [snapshot, setSnapshot] = useState<RuntimePerfSnapshot | null>(null);
  const inFlightRef = useRef(false);

  useEffect(() => {
    if (!enabled) {
      setSnapshot(null);
      return;
    }

    let cancelled = false;
    const tick = async () => {
      if (cancelled || inFlightRef.current) return;
      inFlightRef.current = true;
      try {
        const result = await getRuntimePerfCmd();
        if (!cancelled && result.status === 'ok') {
          setSnapshot(result.data);
        }
      } finally {
        inFlightRef.current = false;
      }
    };

    void tick();
    const timer = window.setInterval(() => {
      if (document.visibilityState !== 'hidden') void tick();
    }, intervalMs);

    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, [enabled, intervalMs]);

  return snapshot;
}

