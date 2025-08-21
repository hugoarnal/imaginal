# imaginal

[Multi-platform](#supported-platforms) currently listening music display

> Currently in development, wanting to expand to something more complex than terminal-based display 👀

## Supported music platforms

- LastFM (recommended)
- Spotify

## Support

> [!WARNING]
>
> Imaginal is built to work only on UNIX systems for now.
>
> You could try on other platforms but I don't know if they work because they're not made with them in mind.

- [x] Linux x86_64
- [x] Linux arm64

- [ ] MacOS x86_64
- [ ] MacOS arm64

- [ ] Windows x86_64
- [ ] Windows arm64

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
