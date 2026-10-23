# Xzuff App Hub service

SQLite-backed catalog and package upload/download API for OpenXzuff UI LAP/LUAP packages.

## Run locally

```sh
cargo run --manifest-path services/app-hub/Cargo.toml
```

Defaults: API at `http://127.0.0.1:8787`, SQLite at `data/xzuff-app-hub.sqlite3`, packages at `data/packages/`. Configure `XZUFF_HUB_BIND`, `XZUFF_HUB_DB`, and `XZUFF_HUB_STORAGE` to change them. Uploads are disabled until `XZUFF_HUB_UPLOAD_TOKEN` is set; use a long random token and keep it in the host secret store.

## Upload a package

```sh
export XZUFF_HUB_UPLOAD_TOKEN='replace-with-a-long-random-token'
curl -f -X POST http://127.0.0.1:8787/api/apps \
  -H 'Authorization: Bearer $XZUFF_HUB_UPLOAD_TOKEN' \
  -F 'title=My Luau App' \
  -F 'version=1.0.0' \
  -F 'publisher=Your Studio' \
  -F 'category=Utilities' \
  -F 'description=An app for OpenXzuff UI' \
  -F 'package=@./MyApp.lap'
```

The package field accepts a ZIP-based `.lap` or `.luap` file up to 64 MiB. The API stores package bytes on disk and metadata in SQLite, and returns a generated app ID and SHA-256 digest.

## Browse and download

- `GET /health`
- `GET /api/apps`
- `GET /api/apps?q=calculator&limit=20`
- `GET /api/apps/{id}/download`

## Security and current limits

**This is a development service, not a public production marketplace.** Uploads require the configured bearer token. The service still has no user accounts, per-user authorization, moderation, malware scanning, signature verification, or rate limiting. By default it binds only to localhost. Do not expose it directly to the internet; place it behind authenticated HTTPS and add review/moderation before public uploads. Upload checks file extension, size, and ZIP signature, but does not fully validate LAP/LUAP manifests. The native installer must validate the package again before installing it.

The Lua Hub screen uses host callbacks to browse the API and ask the native C++ installer to install downloaded files. The bare-metal Gravity Kernel currently has no network stack, filesystem service, or integrated Lua VM, so the service is presently a host-side backend.
