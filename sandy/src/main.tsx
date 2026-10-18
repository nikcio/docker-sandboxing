import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import BootSplash from "./common/components/BootSplash";
import "./styles/styles.css";
import "./styles/templates.css";
import "./styles/dashboard.css";

// The dedicated splash window (opened by the Rust setup hook with the
// `?boot=splash` query) renders only the boot splash: a transparent,
// frameless window with the logo, a spinner, and a label.
const isSplashWindow = new URLSearchParams(window.location.search).has("boot");

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    {isSplashWindow ? <BootSplash transparent /> : <App />}
  </React.StrictMode>,
);
