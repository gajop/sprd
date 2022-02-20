# Install

This project is still under development and there are no official releases.
To install it, you have to clone the repository, compile (setup Rust first if you haven't) and install locally.

```
cargo install --path .
```

# Use

![sprd example](/docs/example.gif?raw=true "sprd example")


```
❯ sprd --help
sprd 0.1.0
Gajo Petrovic <gajopetrovic@gmail.com>
Rapid client

USAGE:
    sprd [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -h, --help
            Print help information

    -r, --root-folder <ROOT_FOLDER>


    -V, --version
            Print version information

SUBCOMMANDS:
    check-sdp
            Check if SDP is fully downloaded
    download
            Download the specified rapid tag
    download-registry
            Download the registry metadata
    download-repo
            Download the repository metadata
    download-sdp
            Download SDP
    fix
            Fix
    help
            Print this message or the help of the given subcommand(s)
    validate
            Validate by fullname
```

## Examples

Download latest Beyond All Reason:

```
cargo run -- --root-folder ~/projects/spring-dir download byar:test
```


Update metadata:
```
cargo run -- --root-folder ~/projects/spring-dir download-registry
```
```
cargo run -- --root-folder ~/projects/spring-dir download-repo
```


# Development

Install Rust and then build as normal:

```
cargo build
```

### Running tests

To run tests you will need to also have pr-downloader installed - it's used to verify that sprd and pr-downloader output is the same.

If you are on Linux, you can get it as so:
```sh
wget https://github.com/gajop/spring-launcher/blob/master/bin/pr-downloader\?raw\=true -O pr-downloader
chmod +x pr-downloader
```

Tests can only be run in single-threaded mode (the rapid filesystem isn't multi-thread safe yet).
```sh
cargo test -- --test-threads=1
```


# :construction: WIP

This project is under construction and not ready for use.

Below you can see my very rough roadmap:

### :construction: In progress:
- New output types: Interactive, Json, Piped (like Interactive but pipable to files), Silent and Auto

### :bookmark: Dev release:
- Improve command naming: download-repo is especially deceptive
- Avoid panic in library code, use explitic errors instead of `<dyn Error>`
- Pool downloader autofix files
- Check validity of other metadata files too: save size & gz validity
- Auto-retry
- Statically link it

### :memo: TODO:

Pool downloader:
- pool downloader tests without internet connectivity (save downloads to files)
- Back to fully async IO (file): use tokio::fs, figure out what was causing the partial write issue despite write_all

Commands:
- Refactor commands out of lib.rs, they're only sensible to have for the binary. We don't want library authors to use the command API

Downloader:
- Option to not commit metadata downloads until pool finishes
- Option to download in parallel (?)
- Make it possible to specify the server URLs


# License

`sprd` is dual licensed under either:

- MIT license ([LICENSE-MIT](docs/LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0, ([LICENSE-APACHE](docs/LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
at your option.

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
