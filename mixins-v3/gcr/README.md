# gcr (v3)

Network rules so the in-sandbox Docker engine can pull base images and
build containers. v3 port of [`../gcr`](../gcr).

Declaration-only: no image content, no build args.

## Runtime egress

`gcr.io:443` + wildcard.
