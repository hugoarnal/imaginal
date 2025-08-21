# imaginal Setup

## Platforms
### Creating an API Key

#### LastFM

Create an [API account here](https://www.last.fm/api/account/create) then replace the `LASTFM_*` keys in your `.env`.

#### Spotify

> [!NOTE]
> If you're using a different port than `9761`, change it in the `SPOTIFY_PORT` environment variable.
> You will need to replace URIs with the `9761` port with the one specified in `SPOTIFY_PORT`.

Create an [app here](https://developer.spotify.com/dashboard/create):
- Add `http://127.0.0.1:9761/callback` as the redirect URI

Then replace the `SPOTIFY_*` keys in your `.env`.
