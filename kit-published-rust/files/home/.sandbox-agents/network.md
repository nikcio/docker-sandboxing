# Network access

Outbound network is deny-by-default behind a proxy firewall. Blocked HTTP/HTTPS requests return
**HTTP 403** — read the body, it names the case:

- `Blocked by local rule for <host>` — a local deny rule, which allow rules cannot override. Ask
  the user to remove it: `sbx policy rm network --resource <host>` (add `--sandbox <name>` for a
  sandbox-scoped deny).
- `Blocked by org policy` — organisation-enforced; tell the user to contact IT.
- `Blocked by network policy: domain <host>` with `detail: no matching allow rule` — default deny.
  Ask the user to run on the host:
  ```bash
  sbx policy allow network <domain>[,<domain>…]   # allow specific domains
  sbx policy allow network "**"                   # allow all traffic not on the denylist
  ```

Inspect recent blocks with `sbx policy log` and active rules with `sbx policy ls`.

## Connection errors (not 403s)

If requests fail with connection errors, the proxy may have misdetected the host's IP stack. Ask
the user to set `DOCKER_SANDBOXES_IP_STACK` (`ipv4only` / `ipv6only` / `dual-stack`) before
starting sandboxd. With `dual-stack`, a protocol without real upstream connectivity shows up as
very slow requests (happy-eyeballs fallback timeout) rather than failures — pin to whichever
single protocol works.

## Publishing ports to the host

Sandbox services are not directly reachable from the host. Ask the user to run on their host:

```bash
sbx ports <sandbox-name> --publish [[HOST_IP:]HOST_PORT:]SANDBOX_PORT[/PROTOCOL]  # e.g. 8080:8080/tcp
sbx ports <sandbox-name>                                                          # list
sbx ports <sandbox-name> --unpublish 8080:8080/tcp
```

Services must listen on `eth0` — bind `0.0.0.0` (IPv4) or `::` (IPv6), not just `127.0.0.1`.

## Reaching host services

The sandbox has its own `localhost`. To reach a service bound to the host's localhost, use
`host.docker.internal` (e.g. `curl http://host.docker.internal:3000`); the target port must be
allowed in the network policy (`localhost:<port>`). Services on other host addresses (e.g. a LAN
IP) are reachable directly if the policy allows them.

Docker containers: published ports are reachable on "localhost" (already in no_proxy). For direct
access to container IPs, add the container's network to the no_proxy configuration.

## .NET Aspire: IPv6 loopback (`[::1]`) and the proxy

Aspire's DCP addresses services as `http://[::1]:<port>` and matches `NO_PROXY` against that exact
bracketed literal, but the sandbox's `NO_PROXY` bypasses loopback via the unbracketed `::1` — so
Aspire's calls go through the proxy (which cannot reach the sandbox's loopback) and fail, usually
with a 502. For Aspire workloads only, append to `/etc/sandbox-persistent.sh`:

```bash
if [ -z "${SBX_ASPIRE_NOPROXY_DONE:-}" ]; then
  export NO_PROXY="${NO_PROXY:+$NO_PROXY,}[::1]"
  export no_proxy="$NO_PROXY"
  export SBX_ASPIRE_NOPROXY_DONE=1
fi
```

The bracketed literal stays out of the default `NO_PROXY` because some clients (e.g. the Azure
DevOps MCP's `typed-rest-client`) parse `NO_PROXY` entries as regular expressions, where `[::1]`
matches every host and would disable the proxy, breaking credential injection.
