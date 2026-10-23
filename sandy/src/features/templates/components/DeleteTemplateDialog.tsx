import { useEffect, useId, useRef, useState } from "react";
import type { SavedTemplate } from "../../../lib/templates";
import { deleteTemplate } from "../../../lib/templates";
import Icon from "../../../common/components/Icon";
import { ErrorBanner } from "../../../common/components/ErrorBanner";
import { useMountedRef } from "../../../common/hooks/useMountedRef";

/** Asks for confirmation, then deletes the saved template file and record. */
export function DeleteTemplateDialog({
  template,
  onDeleted,
  onClose,
}: {
  template: SavedTemplate;
  onDeleted: () => void;
  onClose: () => void;
}) {
  const id = useId();
  const dialog = useRef<HTMLDialogElement>(null);
  const mounted = useMountedRef();
  const actionLock = useRef(false);
  const [busy, setBusy] = useState(false);
  const [deleteError, setDeleteError] = useState<string | null>(null);

  useEffect(() => {
    if (!dialog.current?.open) {
      dialog.current?.showModal();
      dialog.current?.querySelector<HTMLButtonElement>("button")?.focus();
    }
  }, []);

  const close = () => {
    if (actionLock.current) return;
    dialog.current?.close();
  };

  const remove = async () => {
    if (actionLock.current) return;
    actionLock.current = true;
    setBusy(true);
    setDeleteError(null);
    try {
      await deleteTemplate(template.id);
      if (!mounted.current) return;
      onDeleted();
      dialog.current?.close();
    } catch (failure) {
      if (mounted.current) {
        setDeleteError(String(failure));
        actionLock.current = false;
        setBusy(false);
      }
    }
  };

  return (
    <dialog
      ref={dialog}
      className="catalog-launch-dialog"
      aria-labelledby={`${id}-delete-title`}
      aria-describedby={`${id}-delete-description`}
      onCancel={(event) => {
        if (actionLock.current) event.preventDefault();
      }}
      onClose={() => {
        setDeleteError(null);
        onClose();
      }}
    >
      <div aria-busy={busy}>
        <div className="catalog-section-heading">
          <h2 id={`${id}-delete-title`}>Delete {template.name}?</h2>
          <button
            type="button"
            className="icon-button"
            aria-label="Close delete dialog"
            disabled={busy}
            onClick={close}
          >
            <Icon name="close" size={20} />
          </button>
        </div>
        <p className="catalog-muted" id={`${id}-delete-description`}>
          This permanently removes the saved template file{" "}
          <code>{template.path}</code>. Sandboxes already created from it keep working.
        </p>
        {deleteError && (
          <ErrorBanner title="Couldn’t delete this template.">
            <p>{deleteError}</p>
            <span>Try deleting it again.</span>
          </ErrorBanner>
        )}
        <div className="catalog-actions">
          <button type="button" className="ghost" onClick={close} disabled={busy}>
            Cancel
          </button>
          <button type="button" className="danger" onClick={remove} disabled={busy}>
            {busy ? (
              <>
                <span className="spinner" /> Deleting…
              </>
            ) : deleteError ? (
              "Retry delete"
            ) : (
              <>
                <Icon name="trash" size={16} /> Delete template
              </>
            )}
          </button>
        </div>
        {busy && (
          <p className="catalog-status" role="status">
            Removing the saved template file…
          </p>
        )}
      </div>
    </dialog>
  );
}
