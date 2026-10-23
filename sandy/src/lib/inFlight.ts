/**
 * Deduplicates concurrent calls to one async operation: while a call is in
 * flight, every other caller awaits the same promise instead of spawning a
 * duplicate. React StrictMode's double-invoked effects and overlapping
 * window-focus handlers each fire two identical backend commands; this makes
 * them one CLI run.
 */
export function shareInFlight<T>(operation: () => Promise<T>): () => Promise<T> {
  let inFlight: Promise<T> | null = null;
  return () => {
    if (!inFlight) {
      inFlight = operation().finally(() => {
        inFlight = null;
      });
    }
    return inFlight;
  };
}
