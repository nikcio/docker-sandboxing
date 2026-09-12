# Changelog

## [1.1.2](https://github.com/nikcio/docker-sandboxing/compare/v1.1.1...v1.1.2) (2026-09-12)


### Bug Fixes

* **mixins:** allow CA revocation endpoints on port 80 too ([#71](https://github.com/nikcio/docker-sandboxing/issues/71)) ([52d6853](https://github.com/nikcio/docker-sandboxing/commit/52d685305c2f8b0ead34970b28e48102b13e1981))

## [1.1.1](https://github.com/nikcio/docker-sandboxing/compare/v1.1.0...v1.1.1) (2026-09-12)


### Bug Fixes

* **mixins:** allow CA certificate revocation endpoints in the base mixin ([#69](https://github.com/nikcio/docker-sandboxing/issues/69)) ([5eda5c2](https://github.com/nikcio/docker-sandboxing/commit/5eda5c249f31253c4aab1adbf1e0c7c5d9579167))

## [1.1.0](https://github.com/nikcio/docker-sandboxing/compare/v1.0.0...v1.1.0) (2026-09-10)


### Features

* **mixins:** add open-egress mixin (allow all outbound domains) ([#65](https://github.com/nikcio/docker-sandboxing/issues/65)) ([ece0f6c](https://github.com/nikcio/docker-sandboxing/commit/ece0f6c239e9b5162a10664460e1875b865e6138))
* **mixins:** add openapi-codegen mixin (Nikcio.OpenApiCodeGen tool + docs egress) ([#48](https://github.com/nikcio/docker-sandboxing/issues/48)) ([5361bc5](https://github.com/nikcio/docker-sandboxing/commit/5361bc5081630f246962fcc7fc634b6be30d4e9b))
* **mixins:** check-and-install toolchains so any mixin composes on any template ([#57](https://github.com/nikcio/docker-sandboxing/issues/57)) ([36d1a3e](https://github.com/nikcio/docker-sandboxing/commit/36d1a3e99780f9a0889bd17a737ff44d81d859bf))
* **mixins:** update opencode to the latest npm release at sandbox creation ([#67](https://github.com/nikcio/docker-sandboxing/issues/67)) ([b805925](https://github.com/nikcio/docker-sandboxing/commit/b80592564dc5c2a779b4431db358753a13433ba7))
* **templates:** add node-only template, kits, and example ([#56](https://github.com/nikcio/docker-sandboxing/issues/56)) ([d89e83c](https://github.com/nikcio/docker-sandboxing/commit/d89e83c1b47bff666ba845ff3df3fcbe74231848))


### Bug Fixes

* **mixins:** allow the hosts the browser and playwright installs download from ([#54](https://github.com/nikcio/docker-sandboxing/issues/54)) ([7615f6c](https://github.com/nikcio/docker-sandboxing/commit/7615f6c11b6ba2307d045a760e0e9d90e4080e10))
* **mixins:** close the env-guard's symlink and .env.* blind spots ([#63](https://github.com/nikcio/docker-sandboxing/issues/63)) ([c1eab2b](https://github.com/nikcio/docker-sandboxing/commit/c1eab2b3334882db0c040134ea4164d548d12820))
* **mixins:** harden the AGENTS.md rebuild against kits-section injection ([#60](https://github.com/nikcio/docker-sandboxing/issues/60)) ([9747568](https://github.com/nikcio/docker-sandboxing/commit/9747568c685f28b71f17b3da7d70845d6111c99c))
* **mixins:** interpolate MCP gateway config through jq, not a heredoc ([#61](https://github.com/nikcio/docker-sandboxing/issues/61)) ([b165a63](https://github.com/nikcio/docker-sandboxing/commit/b165a63887c44741a3e74a86f1f9e2fed1e73add))
* **mixins:** require --no-sandbox for Chrome headless in the agent note ([#68](https://github.com/nikcio/docker-sandboxing/issues/68)) ([1ea7459](https://github.com/nikcio/docker-sandboxing/commit/1ea745915e72a8ea3b8e12532416a68047907b7e))
* **scripts:** stop eval-ing constructed variable names in new-sandbox.sh ([#62](https://github.com/nikcio/docker-sandboxing/issues/62)) ([08e959d](https://github.com/nikcio/docker-sandboxing/commit/08e959dfca003757d44d0c144bf0c95fafc1616e))

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
