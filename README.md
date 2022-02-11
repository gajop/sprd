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
TODO:
- Use the new sprd-backend API to speed up metadata queries
- Make it possible to specify the server URLs
- Add description. What does this program do?
- Fix/finish the implementation of rapid downloads
- Make it possible to pipe output to files, right now it's only interactive.
- Support parallel downloads (optionally)
 -->