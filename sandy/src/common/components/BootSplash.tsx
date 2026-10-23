import { useEffect } from "react";
import logo from "../../assets/logo.png";

/**
 * Boot screen: the logo, a spinner, and a label.
 *
 * In the main window it is the (hidden) pre-reveal content while the first
 * setup check runs. In the dedicated splash window (`transparent`) it is the
 * only content — a small, frameless, transparent window shown while the main
 * window loads. Neither variant reveals anything: the main window is revealed
 * by App once loading settles, and the splash is closed by it afterwards.
 */
export default function BootSplash({ transparent = false }: { transparent?: boolean }) {
  useEffect(() => {
    if (!transparent) return;
    // The app stylesheet paints a page background; the splash window must be
    // truly transparent, so clear it on both root elements.
    document.documentElement.style.background = "transparent";
    document.body.style.background = "transparent";
  }, [transparent]);

  return (
    <div className={transparent ? "loading loading-splash" : "loading"} role="status">
      <div className={transparent ? "loading-card" : undefined}>
        <img src={logo} alt="" />
        <p className="loading-label">
          <span className="spinner" /> Getting Sandy ready…
        </p>
      </div>
    </div>
  );
}
