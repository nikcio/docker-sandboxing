import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import logo from "../../assets/logo.png";

/**
 * Boot screen shown while the first status check runs. The window starts
 * hidden, so this reveals it once the splash has actually painted — the user
 * never sees a blank frame or the app shell pop in mid-load.
 */
export default function BootSplash() {
  useEffect(() => {
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
  }, []);

  return (
    <div className="loading" role="status">
      <img src={logo} alt="" />
      <p className="loading-label">
        <span className="spinner" /> Getting Sandy ready…
      </p>
    </div>
  );
}
