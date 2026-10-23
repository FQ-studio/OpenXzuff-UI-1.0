# Xzuff App Hub integration contract

The host-side service lives in services/app-hub and stores app metadata in SQLite and package bytes in a separate package directory. The Lua screen is runtime/lua/apps/xzuff_app_hub.lua.

## Host adapter contract

- hub_list(query) -> array of app records from GET /api/apps?q=...
- hub_download(app_id) -> downloaded local package path from GET /api/apps/{id}/download
- choose_package({".lap", ".luap"}) -> selected local path
- hub_upload(path) -> multipart POST /api/apps and return the JSON app record
- install_package(path) -> call the native C++ installer and return true only after success

The UI intentionally fails visibly if a host callback is absent. The Lua bridge module `runtime/lua/apps/xzuff_app_hub_adapter.lua` adapts native host callbacks (HTTP GET, multipart POST, file download, JSON parsing, file selection, prompts, and installer service) to the UI callback contract. Attach it with `Adapter.attach(native_platform, {base_url = "http://127.0.0.1:8787"})`. The repo does not yet include a bare-metal network stack, filesystem, Lua VM service, or host adapter that implements these callbacks, so a real Gravity Kernel desktop cannot use the network Hub until those platform services are built.

## SQL and package storage

The service uses SQLite for title, version, publisher, category, description, package type, SHA-256, size, and creation timestamp. The actual uploaded archive is stored as a file, not a SQLite BLOB. Upload API checks extension, size and ZIP signature. The C++ installer must re-validate package manifest and paths before installation.

## Run and upload

See services/app-hub/README.md. The API binds to 127.0.0.1 by default and has no authentication in this development version. Do not expose it publicly without authenticated HTTPS, authorization, moderation, malware scanning, and rate limiting.
