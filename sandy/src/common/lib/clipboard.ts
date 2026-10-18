/**
 * Copies text to the clipboard, with a fallback for native WebViews that
 * expose the system clipboard only through the editing API.
 */
export async function copyText(text: string) {
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text);
    return;
  }
  const previous = document.activeElement;
  const field = document.createElement("textarea");
  field.value = text;
  field.readOnly = true;
  field.style.position = "fixed";
  field.style.opacity = "0";
  const modal = document.querySelector("dialog[open]");
  (modal ?? document.body).append(field);
  field.select();
  try {
    if (!document.execCommand("copy"))
      throw new Error("Clipboard access is unavailable. Select and copy the text below.");
  } finally {
    field.remove();
    if (previous instanceof HTMLElement) previous.focus();
  }
}
