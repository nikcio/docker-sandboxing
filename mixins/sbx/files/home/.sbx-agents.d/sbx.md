## sbx CLI (Docker Sandboxes)

- The `sbx` CLI (docker-sbx package) is installed via Docker's apt repo
  (download.docker.com — owned by this mixin)
- What works inside the sandbox: kit authoring — `sbx kit validate`,
  `sbx kit inspect`, `sbx kit pack <dir> -o <file.zip>` on this repo's
  `kit/`, `kit-published/`, and `mixins/<area>/`
- What does not: sandbox lifecycle commands (`sbx run/create`,
  `sbx template load`) — they need KVM and a signed-in Docker account
  (`sbx login` opens a browser); both are unavailable inside the sandbox
  VM, so expect them to fail
- Update the CLI with `sudo apt-get update && sudo apt-get install --only-upgrade docker-sbx`
