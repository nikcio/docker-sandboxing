import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import logo from "../../assets/logo.png";

/**
 * Boot screen shown while the first status check runs.
 *
 * In the main window it doubles as the pre-reveal screen: the window starts
 * hidden and this reveals it once the splash has painted. In the dedicated
 * splash window (`transparent`) it is the only content — a small, frameless,
 * transparent window with just the logo, a spinner, and a label — and it
 * closes itself once the main window is ready.
 */
export default function BootSplash({ transparent = false }: { transparent?: boolean }) {
  useEffect(() => {
    if (!transparent) return;
    // The app stylesheet paints a page background; the splash window must be
    // truly transparent, so clear it on both root elements.
    document.documentElement.style.background = "transparent";
    document.body.style.background = "transparent";
  }, [transparent]);

  useEffect(() => {
    if (transparent) return;

    let handle = 0;
    let cancelled = false;
    // Two painted frames guarantee the logo and spinner are on screen before
    // the window becomes visible.
    const reveal = () => {
      if (cancelled) return;
      handle = requestAnimationFrame(() => {
        if (cancelled) return;
        void getCurrentWindow().show();
      });
    };
    handle = requestAnimationFrame(reveal);
    return () => {
      cancelled = true;
      cancelAnimationFrame(handle);
    };
  }, [transparent]);

  return (
    <div className={transparent ? "loading loading-splash" : "loading"} role="status">
      <img src={logo} alt="" />
      <p className="loading-label">
        <span className="spinner" /> Getting Sandy ready…
      </p>
    </div>
  );
}
