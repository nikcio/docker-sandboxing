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

export function Topbar({
  busy,
  checkingSetup,
  onAccount,
}: {
  busy: boolean;
  checkingSetup: boolean;
  onAccount: () => void;
}) {
  return (
    <header className="topbar">
      <div className="brand">
        <img src={logo} alt="" />
        <span>Sandy</span>
      </div>
      <div className="topbar-right">
        <a
          className="icon-button"
          href="https://docs.docker.com/ai/sandboxes/"
          target="_blank"
          rel="noreferrer"
          aria-label="Docker Sandboxes help"
        >
          <Icon name="help" />
        </a>
        <button
          className="account-button"
          disabled={busy || checkingSetup}
          onClick={onAccount}
          aria-label="Account settings"
        >
          <span className="avatar-chip">
            <Icon name="user" size={18} />
          </span>
          <Icon name="chevron" size={16} />
        </button>
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
}: {
  view: View;
  busy: boolean;
  checkingSetup: boolean;
  count: number | null;
  onNavigate: (view: Exclude<View, "create">) => void;
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
        <a href="https://docs.docker.com/ai/sandboxes/" target="_blank" rel="noreferrer">
          View guide <Icon name="arrow" size={15} />
        </a>
      </div>
    </aside>
  );
}
