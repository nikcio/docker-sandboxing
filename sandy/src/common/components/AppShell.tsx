import type { IconName } from "./Icon";
import Icon from "./Icon";
import logo from "../../assets/logo.png";
export type View = "home" | "sandboxes" | "templates" | "settings" | "create";

const navigation: {
  id: Exclude<View, "create">;
  icon: IconName;
  label: string;
}[] = [
  { id: "home", icon: "home", label: "Home" },
  { id: "sandboxes", icon: "box", label: "Sandboxes" },
  { id: "templates", icon: "template", label: "Templates" },
  { id: "settings", icon: "settings", label: "Settings" },
];

export function Topbar() {
  return (
    <header className="topbar">
      <div className="brand">
        <img src={logo} alt="" />
        <span>Sandy</span>
      </div>
    </header>
  );
}

export function Sidebar({
  view,
  busy,
  checkingSetup,
  count,
  onNavigate,
  onViewGuide,
}: {
  view: View;
  busy: boolean;
  checkingSetup: boolean;
  count: number | null;
  onNavigate: (view: Exclude<View, "create">) => void;
  /** Replaces the guide link while the guide view is not built yet. */
  onViewGuide: () => void;
}) {
  return (
    <aside className="sidebar">
      <nav className="nav" aria-label="Main navigation">
        {navigation.map((item) => (
          <button
            key={item.id}
            className={`nav-item ${
              view === item.id || (view === "create" && item.id === "home") ? "active" : ""
            }`}
            aria-current={
              view === item.id || (view === "create" && item.id === "home") ? "page" : undefined
            }
            disabled={busy || checkingSetup}
            onClick={() => onNavigate(item.id)}
          >
            <Icon name={item.icon} />
            <span>{item.label}</span>
            {item.id === "sandboxes" && count !== null && <span className="nav-count">{count}</span>}
          </button>
        ))}
      </nav>
      <div className="help-card">
        <div className="help-title">
          <Icon name="bulb" size={19} /> Need help?
        </div>
        <p>Check out our guide on how to use sandboxes.</p>
        <button type="button" className="link-button" onClick={onViewGuide}>
          View guide <Icon name="arrow" size={15} />
        </button>
      </div>
    </aside>
  );
}
