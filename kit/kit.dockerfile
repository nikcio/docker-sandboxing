# syntax=docker/dockerfile:1

# Shell workload on the shell base image: the platform floor plus the
# launch contract (interactive bash under tini, with the persistent-shell
# environment the base wires up). Kit authoring and debugging happen
# here; compose the mixins your task needs.
#
FROM docker/sandbox-templates:shell

USER root
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        git-lfs \
        jq \
        unzip \
        zip \
    && rm -rf /var/lib/apt/lists/* \
    && git lfs install --system

USER agent
RUN git config --global --add safe.directory "*" \
    && git config --global init.defaultBranch main

# The launch contract: the shell base launches bash under tini — kept as
# this workload's launch command.
USER agent
ENTRYPOINT ["tini", "--", "bash"]
CMD []
