# Changelog

## [1.0.0](https://github.com/nikcio/docker-sandboxing/compare/v0.6.0...v1.0.0) (2026-09-06)


### ⚠ BREAKING CHANGES

* **mixins:** kit sandboxes must now compose mixins/base, mixins/opencode-config, and mixins/opencode-entrypoint (previously mixins/base alone). Environment files listing only mixins/base get the entrypoint error on start; add the two new kit lines and recreate.

### Features

* add Go template, kits, and mixin ([#21](https://github.com/nikcio/docker-sandboxing/issues/21)) ([1724b70](https://github.com/nikcio/docker-sandboxing/commit/1724b7079f6fb877e858c130f6b3720a58db6771))
* add python + uv stack and rename node-dotnet dirs to match ([#11](https://github.com/nikcio/docker-sandboxing/issues/11)) ([b5b613c](https://github.com/nikcio/docker-sandboxing/commit/b5b613c90a9c7f030691d8e93d66920d1f210a5c))
* add Rust template, kits, and mixin ([#22](https://github.com/nikcio/docker-sandboxing/issues/22)) ([44c17f0](https://github.com/nikcio/docker-sandboxing/commit/44c17f0c39b8495d1b0d69b1988ecdd99e8fa9ac))
* **kit:** upgrade opencode to the newest release on sandbox start ([#35](https://github.com/nikcio/docker-sandboxing/issues/35)) ([6b64e10](https://github.com/nikcio/docker-sandboxing/commit/6b64e10e860e421392d61e863803f952d959916e))
* **mixins:** add browser, playwright, and sbx mixins ([#2](https://github.com/nikcio/docker-sandboxing/issues/2)) ([969830e](https://github.com/nikcio/docker-sandboxing/commit/969830ea34c109abbfa417cfce1198ce53823dba))
* **mixins:** add GitHub Copilot provider with mergeable provider configs ([#39](https://github.com/nikcio/docker-sandboxing/issues/39)) ([43ca303](https://github.com/nikcio/docker-sandboxing/commit/43ca303824791b6bb06c8381d92dd1b428599003))
* **mixins:** add omnium mixin (API egress + proxy-managed bearer token) ([#30](https://github.com/nikcio/docker-sandboxing/issues/30)) ([dc45350](https://github.com/nikcio/docker-sandboxing/commit/dc453501a58fdf21ce55d681e68a25d9edbfa6ca))
* **mixins:** add openapi-ts mixin (openapi-ts.dev docs egress) ([#27](https://github.com/nikcio/docker-sandboxing/issues/27)) ([06c8e86](https://github.com/nikcio/docker-sandboxing/commit/06c8e866cfdf70d743d5d7f62cd5bab6fdeb07c8))
* **mixins:** add shared base mixin owning the kit entrypoint runtime and config ([#36](https://github.com/nikcio/docker-sandboxing/issues/36)) ([75d14ea](https://github.com/nikcio/docker-sandboxing/commit/75d14ea9af119eb20264112d3badccb6f310e48b))
* **mixins:** add uniform mixin (docs egress + proxy-managed API key) ([#29](https://github.com/nikcio/docker-sandboxing/issues/29)) ([6204de6](https://github.com/nikcio/docker-sandboxing/commit/6204de6a931edcb09b13cfbd61ff02bd7e044390))
* **mixins:** allow python.org for agent docs lookups ([#18](https://github.com/nikcio/docker-sandboxing/issues/18)) ([a599120](https://github.com/nikcio/docker-sandboxing/commit/a599120c14d4cdaf5ddbe78704993dcab99a4513))
* **mixins:** hold back pnpm installs of packages younger than 24h ([#28](https://github.com/nikcio/docker-sandboxing/issues/28)) ([49d351f](https://github.com/nikcio/docker-sandboxing/commit/49d351fa86598fda8b07217046a30955cfa938be))
* **mixins:** make the default model come from the project opencode config ([#41](https://github.com/nikcio/docker-sandboxing/issues/41)) ([26d3428](https://github.com/nikcio/docker-sandboxing/commit/26d34284f08318addfe4afd24fed4d25477f4be5))
* **mixins:** split base mixin into pick-and-choose mixins ([#40](https://github.com/nikcio/docker-sandboxing/issues/40)) ([b8def6e](https://github.com/nikcio/docker-sandboxing/commit/b8def6e3ba468e4b870b60e723510a48fa755434))
* **scripts:** add sbx-env launcher with per-environment GitHub PAT ([#4](https://github.com/nikcio/docker-sandboxing/issues/4)) ([c18905b](https://github.com/nikcio/docker-sandboxing/commit/c18905ba47940b6c5ae9f97acf44191ea5eeed8d))
* **scripts:** drop the sbx-env launchers in favor of built-in sbx commands ([#7](https://github.com/nikcio/docker-sandboxing/issues/7)) ([595f1ab](https://github.com/nikcio/docker-sandboxing/commit/595f1ab0652f2031a4a0f6932195ee80b3333570))


### Bug Fixes

* **mixins:** allow aka.ms, dotnet-install.sh's primary link source ([#20](https://github.com/nikcio/docker-sandboxing/issues/20)) ([c7bdf23](https://github.com/nikcio/docker-sandboxing/commit/c7bdf233917fd62175cfd0de8f707332ad70d88a))
* **mixins:** allow astral.sh for in-sandbox uv updates ([#15](https://github.com/nikcio/docker-sandboxing/issues/15)) ([2f08858](https://github.com/nikcio/docker-sandboxing/commit/2f08858a1c6d132f6a7a12999232e0c3ab36c124))
* **mixins:** allow Docker Hub's CloudFront blob CDN ([#12](https://github.com/nikcio/docker-sandboxing/issues/12)) ([ebabea4](https://github.com/nikcio/docker-sandboxing/commit/ebabea41eb8a52ae6b3529d6d1ba419f0f24541e))
* **mixins:** allow iojs.org, nvm's io.js mirror ([#17](https://github.com/nikcio/docker-sandboxing/issues/17)) ([483245b](https://github.com/nikcio/docker-sandboxing/commit/483245b741d7feab314cbd5b9e66aa43a998a5ea))
* **mixins:** allow releases.astral.sh, the uv installer's redirect target ([#19](https://github.com/nikcio/docker-sandboxing/issues/19)) ([09cd2e1](https://github.com/nikcio/docker-sandboxing/commit/09cd2e1ac3757fa4022990cbd0b1d990e17a6d65))
* **mixins:** allow the .NET SDK download host on dotnet.microsoft.com ([#16](https://github.com/nikcio/docker-sandboxing/issues/16)) ([12a9bd3](https://github.com/nikcio/docker-sandboxing/commit/12a9bd31f640548dc9993c3d57bf5045f4e3f56c))


### Reverts

* **kit:** upgrade opencode to the newest release on sandbox start ([#38](https://github.com/nikcio/docker-sandboxing/issues/38)) ([ca2f7f3](https://github.com/nikcio/docker-sandboxing/commit/ca2f7f3e7d2001320fa532095971b012798f2098)), closes [#37](https://github.com/nikcio/docker-sandboxing/issues/37)

## [0.6.0](https://github.com/nikcio/docker-sandboxing/compare/v0.5.0...v0.6.0) (2026-09-05)


### Features

* add sandbox kit, template image and bootstrap scripts ([934bbc0](https://github.com/nikcio/docker-sandboxing/commit/934bbc0549d5469412b93ff083307e5217df56b6))
* **kit:** add startup hooks for apt cache and MCP gateway ([dae6914](https://github.com/nikcio/docker-sandboxing/commit/dae691467dcefc7141fe408cd93190443939b12d))
* **kit:** append mixin agent notes to AGENTS.md ([5666507](https://github.com/nikcio/docker-sandboxing/commit/566650730823014f3dd7ec37b243dd906f41cd4c))
* **kit:** make the workspace AGENTS.md authoritative from the kit ([4e1d68a](https://github.com/nikcio/docker-sandboxing/commit/4e1d68a7df3149cfe92b237d16be619ce53b12ad))
* **kit:** refuse sandboxes with .env files in the workspace ([b3e16b7](https://github.com/nikcio/docker-sandboxing/commit/b3e16b756991eac9ddef601c452d93f50641f7f6))
* **kit:** ship a permissive OpenCode config ([1c453cb](https://github.com/nikcio/docker-sandboxing/commit/1c453cb3a235c0cafb9d84ef9e4c0c9b1d020d5f))
* **kit:** split capabilities into composable mixins ([e4a0e7c](https://github.com/nikcio/docker-sandboxing/commit/e4a0e7c7cfa68fc99074cc11e8eac1039cdcb924))
* **mixins:** add GitHub CLI support ([d189a2c](https://github.com/nikcio/docker-sandboxing/commit/d189a2c843a49be54b9f0ba25b226628751fdd62))
* **mixins:** switch the Zeldoc model to zdev-2 ([27954dc](https://github.com/nikcio/docker-sandboxing/commit/27954dc4c8cbeca2912dc2fc26b9c8ce3077b9ba))
* **release:** add release-please and Docker Hub image publishing ([09e5d41](https://github.com/nikcio/docker-sandboxing/commit/09e5d415027d0e729a707d52dee9846191609eca))
* **scripts:** add the sbx-new launcher and register a profile alias ([1c6a5ce](https://github.com/nikcio/docker-sandboxing/commit/1c6a5cebab92be727a54bac83fe8df26ce51cc75))
* **scripts:** make sbx-new wizard-first ([815495c](https://github.com/nikcio/docker-sandboxing/commit/815495c2cead17037c23d9cc88428a6f5ec1609c))
* **scripts:** skip bootstrap secret registration when already stored ([aa08be1](https://github.com/nikcio/docker-sandboxing/commit/aa08be1b0a60415b6bde9e46c4ac7b9c47a95c4a))
* **template:** add the o PATH shim for relaunching opencode ([8261de6](https://github.com/nikcio/docker-sandboxing/commit/8261de6d769d232bb380913417a96c87725b3893))


### Bug Fixes

* **kit:** stop extending the built-in opencode kit ([801a3e4](https://github.com/nikcio/docker-sandboxing/commit/801a3e43930b682987ca09c6fc9cea76b4620d04))
* **mixins:** allow .NET certificate revocation endpoints ([8bcb63f](https://github.com/nikcio/docker-sandboxing/commit/8bcb63f33b375e69fb6461d7ede3cbc498ab9e40))
* **mixins:** allow-list the Zeldoc provider in the model config ([58fe298](https://github.com/nikcio/docker-sandboxing/commit/58fe2983628cbbc97db7b043bdf5669d937929df))
* **mixins:** send the Zeldoc key as a Basic auth header ([c5d3022](https://github.com/nikcio/docker-sandboxing/commit/c5d3022c01edb5091076d1ac0e1f1d679632d45f))

## Changelog

All notable changes to this project will be documented in this file.
