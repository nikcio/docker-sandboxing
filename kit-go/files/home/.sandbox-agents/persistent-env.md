# Persistent shell environment

Shell state lives in `/etc/sandbox-persistent.sh`, sourced before every bash command and in
login/interactive shells (`BASH_ENV`, `/etc/profile.d`, bashrc). Environment variables stored here
persist across all bash invocations.

## Persisting tool setups

Append core init scripts only:

```bash
echo 'export NVM_DIR="$HOME/.nvm"' >> /etc/sandbox-persistent.sh
echo '[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"' >> /etc/sandbox-persistent.sh
```

Same shape for sdkman: `export SDKMAN_DIR="$HOME/.sdkman"` plus source
`$SDKMAN_DIR/bin/sdkman-init.sh`.

## Shell completions break the bash tool

Completion scripts (`$NVM_DIR/bash_completion`, `$SDKMAN_DIR/etc/bash_completion.sh`) must stay out
of `/etc/sandbox-persistent.sh`: it is sourced before every command execution, and completions
rely on variables (`COMP_WORDS`, `COMPREPLY`) that only exist during tab-completion. Symptom:
every bash command — even `echo` — silently returns no output.

Recovery: remove the completion line from `/etc/sandbox-persistent.sh`, exit and restart the
session, verify with `echo test`.

## Tools missing from PATH

Shell snapshots may cache environment state from before a tool was installed. Run the tool through
a fresh login shell so the persistent file is re-sourced:

```bash
bash -l -c "node --version"
```
