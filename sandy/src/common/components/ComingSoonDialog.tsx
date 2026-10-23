import { useEffect, useId, useRef } from "react";
import Icon from "./Icon";

export function ComingSoonDialog({
  feature,
  onClose,
}: {
  feature: string;
  onClose: () => void;
}) {
  const id = useId();
  const dialog = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    if (!dialog.current?.open) {
      dialog.current?.showModal();
      dialog.current?.querySelector<HTMLButtonElement>("button")?.focus();
    }
  }, []);

  return (
    <dialog
      ref={dialog}
      className="feature-dialog"
      aria-labelledby={`${id}-title`}
      aria-describedby={`${id}-description`}
      onClose={onClose}
    >
      <button
        type="button"
        className="icon-button dialog-close"
        aria-label="Close dialog"
        onClick={() => dialog.current?.close()}
      >
        <Icon name="close" size={20} />
      </button>
      <span className="eyebrow">COMING SOON</span>
      <h2 id={`${id}-title`}>{feature} is on its way</h2>
      <p id={`${id}-description`}>
        This part of Sandy is still being built. Check back in a future update.
      </p>
      <button type="button" className="primary" onClick={() => dialog.current?.close()}>
        Got it
      </button>
    </dialog>
  );
}
