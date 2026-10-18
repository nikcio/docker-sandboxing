import type { ReactNode } from "react";

/**
 * The repeating `error-banner` alert pattern: a bold heading, body content,
 * and an optional retry action.
 */
export function ErrorBanner({
  title,
  children,
  actionLabel,
  onAction,
  disabled,
}: {
  title: string;
  children: ReactNode;
  actionLabel?: string;
  onAction?: () => void;
  disabled?: boolean;
}) {
  return (
    <div className="error-banner" role="alert">
      <strong>{title}</strong>
      {children}
      {actionLabel && onAction && (
        <button className="ghost" disabled={disabled} onClick={onAction}>
          {actionLabel}
        </button>
      )}
    </div>
  );
}
