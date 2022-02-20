 Install

## Project

`cargo build`

## Examples

`cargo run -- --root-folder ~/projects/spring-dir download-registry`

`cargo run -- --root-folder ~/projects/spring-dir download-repo`

Download latest BYAR test

`cargo run -- --root-folder ~/projects/spring-dir download byar:test`



`cargo run -- --root-folder ~/projects/spring-dir download-sdp some-sdp-name`

# License

`sprd` is dual licensed under either:

- MIT license ([LICENSE-MIT](docs/LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0, ([LICENSE-APACHE](docs/LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
at your option.

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.


# WIP :construction:

This project is under construction and not ready for use.

Below you can see my very rough roadmap:

WIP :construction:
Output:
- New output types: Interactive, Json, Piped (like Interactive but pipable to files), Silent and Auto

:bookmark: Dev preview:
- Improve command naming: download-repo is especially deceptive

:bookmark: Dev release:
- Statically link it
- Pool downloader autofix files
- Check validity of other metadata files too: save size & gz validity
- Auto-retry
- Add description. What does this program do?
- Add --help to README.md
- Add contribution guidelines

:memo: TODO:
Pool downloader:
- pool downloader tests without internet connectivity (save downloads to files)
- Back to fully async IO (file): use tokio::fs, figure out what was causing the partial write issue despite write_all

Commands:
- Refactor commands out of lib.rs, they're only sensible to have for the binary. We don't want library authors to use the command API

Downloader:
- Option to not commit metadata downloads until pool finishes
- Option to download in parallel (?)
- Make it possible to specify the server URLs
