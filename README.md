# imaginal

[Multi-platform](#supported-platforms) currently listening display

> Currently in development, wanting to expand to something more complex than terminal-based display 👀

## Supported platforms

- LastFM (recommended)
- Spotify

## Build

#### Prerequisites
- Rust (w/ Cargo)

Once cloned, rename the `.env.example` into `.env`.

Then follow the [SETUP.md](./SETUP.md) file for instructions on how to setup.

Build the binary:
```sh
cargo build --release
```

## References

- Spotify Currently Playing: https://developer.spotify.com/documentation/web-api/reference/get-the-users-currently-playing-track

- LastFM Currently Playing: https://www.last.fm/api/show/user.getRecentTracks
