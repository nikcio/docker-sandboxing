# ghcr (v3)

Network rules so the in-sandbox Docker engine can pull base images and
build containers. v3 port of [`../ghcr`](../ghcr).

Declaration-only: no image content, no build args.

## Runtime egress

`ghcr.io:443` + wildcard.
