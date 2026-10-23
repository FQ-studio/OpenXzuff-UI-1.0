# Xzuff App Hub — Appwrite publisher configuration

This document prepares the configuration contract for the Appwrite-backed publisher. It does not contain credentials.

## 1. Create local environment file

Copy `.env.example` to `.env` and fill in the values locally. Keep `.env` out of Git.

- `APPWRITE_ENDPOINT`: Appwrite API endpoint, normally `https://cloud.appwrite.io/v1`.
- `APPWRITE_PROJECT_ID`: your Appwrite Project ID.
- `APPWRITE_API_KEY`: server-side API key. Keep this secret; never place it in Lua, a .lap/.luap package, client-side UI, or a public repository.
- `APPWRITE_BUCKET_ID`: ID of the Storage bucket that will hold published packages.
- `XZUFF_HUB_UPLOAD_TOKEN`: separate bearer token used by the current Xzuff App Hub upload API. It is not the Appwrite API key.

## 2. Appwrite permissions

Create a Storage bucket dedicated to Xzuff App Hub packages. Give the server API key only the scopes needed for file storage. If packages should be downloadable by everyone, configure public read access intentionally; never grant public write access. Otherwise, add a download authorization flow.

## 3. Important current implementation status

The existing `services/app-hub` Rust service currently stores package bytes on local disk and catalog metadata in SQLite. The Appwrite values in `.env.example` are configuration placeholders and are **not yet wired into the upload/download handlers**. Do not expect changing these values alone to send files to Appwrite. The Appwrite Storage adapter must be implemented and tested before enabling Appwrite publishing.

## 4. Local development

The service currently reads process environment variables. Export values from your local `.env` using your development environment, or add a dotenv loader as part of the next implementation step. Do not paste secrets into chat or commit them.

## 5. Before public launch

The current upload bearer token is a shared publisher credential, not a user-account system. Before public hosting, use HTTPS, protect the publisher route, add package manifest validation, malware scanning/moderation, rate limiting, and auditable publisher identities.
