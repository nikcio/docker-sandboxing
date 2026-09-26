# syntax=docker/dockerfile:1
FROM scratch
# OPENCODE_CONFIG points at the combined provider config the opencode
# mixin's install hook builds from ~/.config/opencode/mixins.d/ fragments.
ENV OPENCODE_CONFIG=/home/agent/.config/opencode/sandbox-mixins.jsonc
COPY --chown=1000:1000 files/home/.config/opencode/opencode.jsonc /usr/share/sandbox/opencode/opencode.jsonc
