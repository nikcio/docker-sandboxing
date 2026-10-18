import Icon from "../../../common/components/Icon";

export type Method = "terminal" | "opencode" | "vscode";

const methods: { id: Method; icon: "terminal" | "monitor" | "code"; label: string; hint: string }[] =
  [
    { id: "terminal", icon: "terminal", label: "Terminal", hint: "Native terminal" },
    { id: "opencode", icon: "monitor", label: "OpenCode Desktop", hint: "SSH connection guide" },
    { id: "vscode", icon: "code", label: "VS Code", hint: "Remote – SSH guide" },
  ];

export function ConnectionMethods({
  method,
  busy,
  onSelect,
}: {
  method: Method;
  busy: boolean;
  onSelect: (method: Method) => void;
}) {
  return (
    <div className="connection-options" role="group" aria-label="Connection options">
      {methods.map((item) => (
        <button
          key={item.id}
          className={method === item.id ? "active" : ""}
          aria-pressed={method === item.id}
          aria-controls="connection-method"
          disabled={busy}
          onClick={() => onSelect(item.id)}
        >
          <Icon name={item.icon} size={23} />
          <span>{item.label}</span>
          <small>{item.hint}</small>
        </button>
      ))}
    </div>
  );
}
