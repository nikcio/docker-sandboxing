import { useEffect, useRef } from "react";

/**
 * A ref that becomes `false` after the first unmount, for guarding async
 * state updates against a component that has gone away.
 */
export function useMountedRef(): { current: boolean } {
  const mounted = useRef(true);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);
  return mounted;
}
