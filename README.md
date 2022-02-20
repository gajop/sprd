# Install

## Project

`cargo build`

## Examples

`cargo run -- --root-folder ~/projects/spring-dir download-registry`

`cargo run -- --root-folder ~/projects/spring-dir download-repo`

Download latest BYAR test

`cargo run -- --root-folder ~/projects/spring-dir download byar:test`



`cargo run -- --root-folder ~/projects/spring-dir download-sdp some-sdp-name`

<!--
WIP:
- Fix/finish the implementation of rapid downloads

TODO:
Pool downloader:
- pool downloader tests without internet connectivity (save downloads to files)
- split pool downloader to separate file
- Pool downloader autofix files
- Back to fully async IO (file): use tokio::fs, figure out what was causing the partial write issue despite write_all

Commands:
- Refactor commands out of lib.rs, they're only sensible to have for the binary. We don't want library authors to use the command API

Downloader:
- Check validity of other metadata files too: save size & gz validity
- Option to not commit metadata downloads until pool finishes
- Option to download in parallel (?)
- Make it possible to specify the server URLs

Output:
- New output types: Interactive, Json, Piped (like Interactive but pipable to files), Silent and Auto

Docs:
- Add description. What does this program do?
- Add --help to README.md

 -->