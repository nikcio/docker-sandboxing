import Icon from "./Icon";

export type ToastNotice = { kind: "success" | "error"; text: string };

export function Toast({ notice, onDismiss }: { notice: ToastNotice; onDismiss: () => void }) {
  return (
    <div className={`dashboard-toast ${notice.kind}`} role={notice.kind === "error" ? "alert" : "status"}>
      <Icon name={notice.kind === "success" ? "check" : "help"} size={20} />
      <span>{notice.text}</span>
      <button className="icon-button" aria-label="Dismiss notification" onClick={onDismiss}>
        <Icon name="close" size={17} />
      </button>
    </div>
  );
}
