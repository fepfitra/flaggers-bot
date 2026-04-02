# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.1.3](https://github.com/fepfitra/flaggers-bot/compare/v2.1.2...v2.1.3) - 2026-04-02

### Fixed

- remove duplicate release job (release-plz handles releases now)

## [2.1.2](https://github.com/fepfitra/flaggers-bot/releases/tag/v2.1.2) - 2026-04-02

### Added

- add release-plz for automated releases with changelog
- save challenge description in README.md for --dump
- download files locally in dump command
- add dump command via CLI
- add docker-compose.yml
- add Docker support with GHCR publishing
- add daemon logs command
- better log
- add download and attach file from each challenge
- wtf
- *(update)* show better error
- add move channel between active and archive
- update readme
- add /dump command
- add join a channel button
- add create a channel button
- add contribution note
- add the program runs systemctl daemon-reload and systemctl enable --now automatically instead of printing instructions
- add daemon uninstall
- add --uninstall
- add description for misc commands
- fix update error decoding response body
- add --update and /update command
- add --restart
- more rich /version and refactor
- add version
- apply prompt loop if invalid token
- add prompt for bot token
- add config file
- add --version command
- add daemon mode

### Fixed

- remove pull_request trigger to avoid detached HEAD
- auto restart on update only if systemd service exists
- prioritize HTML file extraction over API
- download attachments from CTFd files endpoint
- CI only runs on PRs
- build runs independently on tag push (CI already ran on PR)
- workflow runs on master push and properly handles tag pushes
- auto-pull in publish.sh if behind remote
- improve publish.sh (add branch/uncommitted checks, run tests, push tag first)
- simplify created channel message
- simplify created channel response message
- make install.sh portable and robust (jq/sed fallback, LATEST_TAG validation, mktemp, trap cleanup)
- use jq instead of grep for VERSION resolution
- Dockerfile VERSION=latest resolution and remove obsolete version warning
- build latest tag on git tag push
- improve Docker update error message
- disable /update command in Docker (use docker-compose pull instead)
- simplify Dockerfile to download pre-built binary
- log success message after update restart
- wait 2s before restart to let Discord message send
- add KillMode=process to systemd service
- register commands globally on bot startup
- add StandardOutput/StandardError to systemd service for logging
- restart properly after update
- add return statements to prevent fallthrough
- run bot after doing daemon stuffa
- systemd_install from current exe path
- remove user from systemd service
- vps tails
- add silently kills any running instances
- try add delay between stop and restart to ensure proper cleanup
- try fix update from /update
- added chmod after rename to ensure the binary has execute permission
- branch stuff
- abort
- clippy
- better token prompt
- add bot tests the token before starting

### Other

- use fully qualified html_to_markdown_rs::convert
- bump version to v2.1.2
- bump version to v2.1.1
- bump version to v2.1.0
- bump version to v2.0.28
- update about to include CLI
- add CLI dump to README, update install docs
- ignore dump directories
- bump version to v2.0.27
- bump version to v2.0.26
- Add logging for command registration
- bump version to v2.0.25
- bump version to v2.0.24
- bump version to v2.0.23
- bump version to v2.0.22
- remove redundant test in publish.sh (already runs in CI)
- bump version to v2.0.21
- bump version to v2.0.20
- fix workflow issues - add build conditions, consolidate docker, add concurrency control
- remove unnecessary DISCORD_TOKEN env var
- add pull request tests and clippy check
- Revert "fix: simplify created channel response message"
- bump version to v2.0.19
- bump version to v2.0.18
- add config.json.example and update README
- update README to not require git clone for docker-compose
- bump version to v2.0.17
- bump version to v2.0.16
- add disclaimer for update command in Docker
- bump version to v2.0.15
- bump version to v2.0.14
- bump version to v2.0.13
- bump version to v2.0.12
- bump version to v2.0.11
- restructure to clean architecture
- remove macOS build, add OS check for daemon commands
- clippy
- bump version to v2.0.10
- bump version to v2.0.9
- update readme
- add -D warnings so clippy will fail on warnings and cancel the publish.
- bump version to v2.0.8
- bump version to v2.0.7
- remove unused message
- bump version to v2.0.6
- bump version to v2.0.5
- bump version to v2.0.4
- bump version to v2.0.3
- remove redundant
- bump version to v2.0.2
- bump version to v2.0.1
- add contribution and feature request note
- bump version to v2.0.0
- clippy and fmt
- pecut eyay
- add dev workflow
- [**breaking**] change version to about
- bump version to v1.5.0
- add embed discord button
- bump version to v1.4.1
- bump version to v1.4.0
- move the run blocking into a new command
- fmt
- [**breaking**] if no arguments, it prints help
- update readme for daemon uninstall and --uninstall
- bump version to v1.3.0
- use daemon:: instead of use daemon::{..}
- remove unused function
- bump version to v1.2.4
- bump version to v1.2.3
- bump version to v1.2.2
- clean
- bump version to v1.2.0
- remove token from env
- updated README with new subcommand interface
- bump version to v1.1.0
- change PID based to systemd based daemon
- bump version to v1.0.7
- try other approach
- bump version to v1.0.6
- bump version to v1.0.5
- bump version to v1.0.4
- bump version to v1.0.3
- bump version to v1.0.2
- bump version to v1.0.1
- bump version to v1.0.0
- add if binary exists and has the same version as the latest
- remove path
- fix grep
- bump version to v0.1.26
- fix grep to handle minified json
- bump version to v0.1.25
- fmt
- bump version to v0.1.24
- move clippy from github action to local publish script
- more clear and connected context
- bump version to v0.1.23
- add discord bot setup
- update the development
- bump version to v0.1.22
- use faster update cargo.lock
- bump version to v0.1.21
- fix cargo lock error
- bump version to v0.1.20
- fix cargo lock update
- add publish version
- bump version to v0.1.19
- fix not pushed
- bump version to v0.1.18
- fix minor
- bump version to v0.1.17
- use publish script for bump and trigger publish
- bump version to v0.1.16
- bump version to v0.1.15
- fix not replaced the old binary
- update installer
- update installer
- update
- remove aliases
- bump version to v0.1.13
- branch
- add push back bump version
- clippy first befure build
- fix script prompt in the background
- wtf
- cicd docs: fix build, up, and download properly
- add quick install
- init
- fix release write permission
- fix release
- remove windows build
- ubuntu only
- automate release and bump version
- modularize bot runner
- modularize cli commands
- initial commit
