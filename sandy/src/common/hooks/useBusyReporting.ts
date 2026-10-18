import { useEffect, useRef } from "react";

/**
 * Reports the component's busy state to a parent callback for the lifetime of
 * the component, and always reports `false` again on unmount.
 */
export function useBusyReporting(busy: boolean, onBusyChange?: (busy: boolean) => void) {
  const callback = useRef(onBusyChange);
  callback.current = onBusyChange;

  useEffect(() => {
    callback.current?.(busy);
  }, [busy]);

  useEffect(() => {
    return () => {
      callback.current?.(false);
    };
  }, []);
}
