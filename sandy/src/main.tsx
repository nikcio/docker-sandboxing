import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App";
import BootSplash from "./common/components/BootSplash";
import "./styles/styles.css";
import "./styles/templates.css";
import "./styles/dashboard.css";

// The dedicated splash window (created by the Rust setup hook with the label
// `splash`) renders only the boot splash: a transparent, frameless window
// with the logo, a spinner, and a label.
const isSplashWindow = getCurrentWindow().label === "splash";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    {isSplashWindow ? <BootSplash transparent /> : <App />}
  </React.StrictMode>,
);
