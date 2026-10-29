# mcr (v3)

Network rules so the in-sandbox Docker engine can pull base images and
build containers. v3 port of [`../mcr`](../mcr).

Declaration-only: no image content, no build args.

## Runtime egress

`mcr.microsoft.com` + wildcard, `*.data.mcr.microsoft.com`.
