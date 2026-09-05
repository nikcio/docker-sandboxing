# Changelog

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
