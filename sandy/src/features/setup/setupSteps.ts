import type { SetupStatus } from "../../lib/sandbox";
import { installSbx, login, enableImagePaste, setupSsh } from "../../lib/sandbox";
import type { IconName } from "../../common/components/Icon";

export interface SetupStep {
  id: string;
  label: string;
  title: string;
  description: string;
  command: string;
  action: string;
  icon: IconName;
  isDone: (status: SetupStatus | null) => boolean;
  run: () => Promise<string | void>;
}

export const SETUP_STEPS: SetupStep[] = [
  {
    id: "install",
    label: "Install CLI",
    title: "A small tool. A world of possibilities.",
    description:
      "Sandy uses the Docker Sandboxes CLI (sbx) to work with your local sandboxes. We’ll install it with your system’s package manager. You may be asked for administrator permission.",
    command: "Install sbx using Homebrew, WinGet, or apt for your platform",
    action: "Install CLI",
    icon: "terminal",
    isDone: (status) => !!status?.sbx_installed,
    run: installSbx,
  },
  {
    id: "login",
    label: "Docker account",
    title: "Let’s get you signed in",
    description:
      "Connect your Docker account to use Sandboxes. We’ll open your browser so you can sign in, then check the result here. Already signed in? Sandy will pick that up automatically.",
    command: "sbx login",
    action: "Sign in to Docker",
    icon: "user",
    isDone: (status) => !!status?.signed_in,
    run: login,
  },
  {
    id: "imagepaste",
    label: "Image pasting",
    title: "Give your agent the full picture",
    description:
      "Paste screenshots and images from your clipboard directly into agent chats. This enables clipboard image pasting for your local sandboxes.",
    command: "sbx settings set clipboard.imagePaste true",
    action: "Enable image pasting",
    icon: "clipboard",
    isDone: (status) => !!status?.image_paste_on,
    run: enableImagePaste,
  },
  {
    id: "ssh",
    label: "SSH access",
    title: "Your favorite tools, connected",
    description:
      "Configure your SSH client to reach sandboxes at <name>.sbx. This lets you connect from your editor or terminal using the tools you already love.",
    command: "sbx setup ssh",
    action: "Set up SSH",
    icon: "key",
    isDone: (status) => !!status?.ssh_set_up,
    run: setupSsh,
  },
];

export function isSetupComplete(status: SetupStatus | null) {
  return SETUP_STEPS.every((step) => step.isDone(status));
}
