# docker-hub (v3)

Network rules so the in-sandbox Docker engine can pull base images and
build containers. v3 port of [`../docker-hub`](../docker-hub).

Declaration-only: no image content, no build args.

## Runtime egress

`docker.io` + wildcard, `*.docker.com`, `production.cloudflare.docker.com`,
`production.cloudfront.docker.com`.
