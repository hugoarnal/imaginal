# imaginal

[Multi-platform](#supported-platforms) currently listening display

> Currently in development, wanting to expand to something more complex than terminal-based display 👀

## Supported platforms

- LastFM (recommended)
- Spotify

### Obtain necessary keys

**LastFM**:

Create an [API account here](https://www.last.fm/api/account/create) then replace the LASTFM_* keys in your `.env`.

**Spotify**:

Create an [app here](https://developer.spotify.com/dashboard/create):
- Add `http://127.0.0.1:9761/callback` as the redirect URI

Then replace the SPOTIFY_* keys in your `.env`.

## Build

#### Prerequisites
- Rust (w/ Cargo)
- Follow the [platform guide](#obtain-necessary-keys)

Once cloned, rename the `.env.example` into `.env` and replace all useful variables.

Build the binary:
```sh
cargo build --release
```
