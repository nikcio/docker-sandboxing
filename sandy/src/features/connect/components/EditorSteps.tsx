import type { ConnectionInfo } from "../../../lib/sandbox";
import type { Method } from "./ConnectionMethods";

export function EditorSteps({
  method,
  info,
  sandboxName,
}: {
  method: Exclude<Method, "terminal">;
  info: ConnectionInfo | null;
  sandboxName: string;
}) {
  if (method === "opencode") {
    return (
      <ol className="connection-steps">
        <li>
          Open <strong>OpenCode Desktop</strong> on your computer.
        </li>
        <li>
          Go to <strong>Settings → Server → Add SSH Server</strong>.
        </li>
        <li>
          Enter host <code>{info?.host ?? `${sandboxName}.sbx`}</code> and SSH user{" "}
          <code>{info?.user ?? "_default_user_"}</code>.
          {info?.port != null ? (
            <>
              {" "}
              Set the port to <code>{info.port}</code>.
            </>
          ) : (
            <> Leave the port unset to use your SSH configuration.</>
          )}
        </li>
        <li>Save the server and select it to connect to the sandbox.</li>
      </ol>
    );
  }
  return (
    <ol className="connection-steps">
      <li>
        Install the <strong>Remote – SSH</strong> extension in VS Code.
      </li>
      <li>
        Click the <strong>&gt;&lt; remote connection button</strong> in the bottom-left corner.
      </li>
      <li>
        Choose <strong>Connect to Host…</strong> and enter{" "}
        <code>{info ? `${info.user}@${info.host}` : `_default_user_@${sandboxName}.sbx`}</code>.
        {info?.port != null && (
          <>
            {" "}
            Use the SSH command below with <strong>Add New SSH Host…</strong> to include port{" "}
            <code>{info.port}</code>.
          </>
        )}
      </li>
      <li>
        Connect, then use <strong>Open Folder</strong> to choose your project inside the sandbox.
      </li>
    </ol>
  );
}
