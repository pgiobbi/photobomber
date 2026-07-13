# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

This is a guest photo-upload web app. The frontend is themed for TOMORROWLAND 2026 and its
"Consciencia" theme (Boom, Belgium, July 2026): the owners wear t-shirts printed with a QR
code pointing at this site, and whoever scans it is pushed to take a picture WITH the wearers
(or photobomb them) and upload it privately to their phones. The backend is generic and still
carries older festival concepts (locations/stages, leaderboard, upvotes) that the current
frontend no longer uses but has NOT removed.

Three pieces:
- `backend/` - Rust (actix-web) HTTP API, listens on `0.0.0.0:3002`. Generic; unaware of the
  frontend theme.
- `frontend/` - static vanilla JS/HTML/CSS, no build step or framework. `index.html` is the
  guest camera + multi-select upload flow (`main.js`, `compress.js`); `admin-only.html` is the
  standalone admin UI (older festival-era). `leaderboard.js` exists but is no longer wired in.
- `nginx/` - reverse proxy: serves the static frontend and proxies `/api/` to the backend.

### Frontend theme vs backend

The Consciencia frontend uploads everything with `isPublic=false` (private to the shirt
wearers), uploads each selected file as its own request (multi-select supported), and
compresses HARD on-device before upload (`compress.js`: max 1600px, WebP starting at q0.75
stepping down to q0.5, ~350KB target; pass-through only for already-tiny files). The venue
network is slow and saturated, so keep page assets tiny too: everything under
`frontend/assets/consciencia/` is deliberately giga-compressed (~36KB total) and new assets
must follow suit. Theme tokens live in `styles.css` (:root): desert gold `#D9A648`,
terracotta `#C4652F`, bone `#EFE3C8` on midnight `#10181F`; display font Marcellus, body
Jost. The frontend does NOT use the location/stage UI, the leaderboard gallery, or upvotes.
Those backend endpoints still exist and work; they are simply unused by `index.html`. If you
reskin or repurpose again, the backend contract below is what matters.

Official theme assets came from https://consciencia.tomorrowland.com (Webflow CDN
`cdn.prod.website-files.com`). Gotcha: some of the CDN's "transparent" PNG/AVIF variants have
an opaque green key plate (`#47714D`) baked in - verify alpha before using them, and prefer
`cwebp -resize` directly on the source (ffmpeg conversions can silently flatten alpha).

## Commands

Backend (run from `backend/`):
- `cargo build` / `cargo run` - build/run locally. Requires env vars (see below); set
  `CONFIG_DIR` (must contain `stages.json`; the SQLite db is created here) and `UPLOAD_DIR`.
- `cargo build --locked --release` - release build (what the Dockerfile runs).
- `docker compose up --build` (from `backend/`) - build + run the backend container. Mounts
  `./uploads` and `./config`, exposes `127.0.0.1:3002`.

Frontend / full stack:
- `docker compose -f compose.dev.yaml up` (from repo root) - runs nginx serving `frontend/`
  with `nginx/nginx.dev.conf`. The backend is expected at `host.docker.internal:3002`.
- The frontend is static; just edit and reload. No bundler. `main.js` lazy-imports
  `leaderboard.js` via dynamic `import()`.

There are no automated tests in this repo.

## Backend architecture

`main.rs` builds a single `AppState` (`types/state.rs`) shared across handlers via
`web::Data`. It holds: `upload_dir`, `max_file_size`, a `RwLock<LocationState>` (current
stage, mutated in-memory only - NOT persisted), `CredentialState` (admin user/pass from
env), the parsed `stages` list, the SQLite `db_pool`, and a `moka` `Cache<String, i8>`.

Source layout under `src/api/`:
- `routes/` - handlers, one module per resource (`auth`, `images`, `location`, `params`,
  `stages`). Endpoints use empty `#[get("")]`/`#[post("")]` paths because `main.rs` mounts
  each under its own `web::scope(...)`.
- `extractors/` - `AuthenticatedUser` (request extractor) and `auth_middleware.rs` (the
  `AuthMiddlewareFactory` actix middleware).
- `libraries/` - `auth.rs` (JWT) and `db.rs` (SQLite init + helpers).
- `constants.rs` - `ADMIN_ID = 1`, `PUBLIC_ID = 0`, allowed extensions, `UPVOTES_PER_UPLOAD = 3`,
  default 6MB max file size (`DEFAULT_MAX_FILE_SIZE`), `APP_NAME` (JWT issuer).

### Route scopes and auth

Three top-level scopes, each with deliberately path-scoped cookies to avoid collisions:
- `/api/auth` - no auth. `login` (sets cookies), `logout`.
- `/api/public` - requires auth, wrapped in `AuthMiddlewareFactory`. Cookies are `Path=/api/public`.
- `/api/admin` - requires auth AND `user.is_admin()`. Cookies are `Path=/api/admin`.

Auth is cookie-based JWT (HS256, `jsonwebtoken`), NOT bearer headers. `login` accepts an
optional username/password body: matching admin creds -> `user_id = ADMIN_ID`, anything else
(including empty `{}`) -> anonymous `PUBLIC_ID`. The public frontend calls `login` with `{}`
on load to obtain a public session. The middleware validates the access token, transparently
refreshes it from the refresh token when >80% through its lifetime
(`should_refresh_token`), and re-sets cookies on the response (admin vs public path inferred
from `req.path()`). Admin-only endpoints additionally check `user.is_admin()` inside the
handler (e.g. `post_location`).

### Image upload + upvote flow

`POST /api/public/images/upload` (multipart, `images` + `isPublic`):
- Gated by the `upload_allowed` DB param (`get_upload_allowed`); denied with 403 if not `"1"`.
- Validates content-type is `image/*` and size <= `max_file_size`.
- Filenames are content-hashed with `XxHash64` (`<hash>.<ext>`), giving dedup: re-uploading an
  identical image is detected via existing file path and does NOT issue a new upvote token.
- File is persisted then a row is inserted into `images`; if the DB insert fails the file is
  removed (no orphans).
- On a fully-new upload, an `upvote_token` cookie is issued and the `moka` cache maps
  `upvote.<token>` -> `UPVOTES_PER_UPLOAD` remaining upvotes.

`POST /api/public/images/{id}/upvote` decrements the cached count for the caller's
`upvote_token`; karma is only incremented for `is_public` rows; the cookie is removed when the
count hits zero. Upvote budget lives only in the in-memory cache, so it resets on restart.

Frontend compresses images client-side for bandwidth (`compress.js`): only files already
<=~350KB and <=1600px in a web format are uploaded untouched, everything else is scaled to
max 1600px and re-encoded to WebP, stepping quality from 0.75 down to 0.5 until under the
~350KB target. The backend enforces a hard 6MB cap (`DEFAULT_MAX_FILE_SIZE`), and nginx caps
the proxied body at 12M (`nginx/nginx.dev.conf`) - raise both together if you need larger
uploads. Each selected photo is sent as its own request.

### Persistence

SQLite at `$CONFIG_DIR/photobomb.sqlite`, created/initialized on startup by
`connect_or_initialize_db` (checks for table existence, then `CREATE TABLE`). There are no
migration files - schema changes go in `db.rs`. Two tables:
- `images` (id, filename, is_public, karma, created_at).
- `params` (key/value/description/updated_at) - runtime feature flags; `upload_allowed`
  (default `'0'`) gates uploads. Admins read/write these via `/api/admin/params`.

Uploaded files live in `$UPLOAD_DIR` (mounted volume `uploads/`); `GET /api/public/images/{filename}`
sanitizes the name and serves the bytes. The image count (`/api/public/images/count`) is derived
by scanning the upload directory, not the DB.

### Locations / stages

`config/stages.json` (`{ "stages": [{ id, name }] }`) is the source of truth for valid stages,
loaded once at startup into `AppState.stages`. `POST /api/admin/location` validates a selected
Tomorrowland stage against this list before storing it in the in-memory `LocationState`
(a `Custom { name }` variant is also allowed). Current location is read by the public UI to
show "where is the photobomber now". Note this state is in-memory only and is lost on restart.

## Environment variables

Required at runtime (see `backend/compose.yaml` for defaults/examples):
- `CONFIG_DIR` - dir containing `stages.json` + the SQLite db (no default; `main.rs` panics if unset).
- `UPLOAD_DIR` - upload target (default `./uploads`).
- `PHOTOBOMBER_ADMIN_USERNAME` / `PHOTOBOMBER_ADMIN_PASSWORD` - admin login (read at startup;
  `CredentialState::from_env` unwraps, so both must be set).
- `FPX_JWT_SECRET` - HS256 signing secret (falls back to a hardcoded dev default - set in prod).
- `FPX_ACCESS_TOKEN_DURATION_MINUTES` (default 15) / `FPX_REFRESH_TOKEN_DURATION_MINUTES` (default 10080).
- `PHOTOBOMBER_CORS_ALLOW_ALL` - `"true"` disables the origin allowlist. The allowlist (in
  `main.rs`) hardcodes localhost plus the production domains; add new domains there.
- `MAX_FILE_SIZE` (bytes, default 6MB), `RUST_LOG`.

## Conventions / gotchas

- Rust edition 2024, pinned toolchain `1.86.0` (Dockerfile builds for `aarch64`/arm64 only).
- JSON over the wire is camelCase (serde `rename_all = "camelCase"` on request/response types).
- Handlers generally return `Ok(HttpResponse::...)` even for client errors (400/403) rather
  than propagating `Err`, so the middleware cookie-refresh path still runs.
- `is_admin` credential check sleeps 500ms deliberately (timing-attack mitigation on login).
- ASCII-only punctuation in all files (see user global preferences): no emdashes, smart quotes,
  ellipsis, or arrow glyphs.
