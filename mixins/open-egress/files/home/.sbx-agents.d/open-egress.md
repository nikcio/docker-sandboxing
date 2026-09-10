## Open egress

The sandbox composes the `open-egress` mixin, whose `**` rule allows all
outbound domains — the deny-by-default baseline does not apply. A blocked
request means a local deny rule or org policy: check `sbx policy log` and
`sbx policy ls`.
