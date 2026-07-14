import { useEffect, useRef } from 'react';

/**
 * Calls `callback` every `intervalMs` while `enabled` is true.
 *
 * The latest `callback` is always invoked without re-creating the interval
 * when the reference changes, so consumers can pass inline closures safely.
 *
 * @param callback - Function to invoke on each tick.
 * @param intervalMs - Polling interval in milliseconds.
 * @param enabled - When false, the interval is not started.
 */
export function usePolling(
  callback: () => void,
  intervalMs: number,
  enabled: boolean = true,
): void {
  const callbackRef = useRef(callback);
  callbackRef.current = callback;

  useEffect(() => {
    if (!enabled) return;
    const id = setInterval(() => callbackRef.current(), intervalMs);
    return () => clearInterval(id);
  }, [enabled, intervalMs]);
}
