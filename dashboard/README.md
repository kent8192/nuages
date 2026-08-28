# Reinhardt Cloud

A Reinhardt project.

## Quick links

- Full usage guide → [`../docs/tools/dashboard.md`](../docs/tools/dashboard.md)
- Deployment guide for Platform Operators → [`../docs/tools/dashboard.md#deployment-of-the-dashboard-itself-for-platform-operators`](../docs/tools/dashboard.md#deployment-of-the-dashboard-itself-for-platform-operators)
- Source of truth for Dashboard usage and configuration is the guide above; this README is a contributor-oriented summary.
- Deployment flow & component responsibilities → [`../docs/architecture/deployment-flow.md`](../docs/architecture/deployment-flow.md)

## Getting Started

### Using cargo-make (Recommended)

Install cargo-make:
```bash
cargo install cargo-make
```

Run the development server:
```bash
cargo make runserver
```

### Using manage command

```bash
# Run the development server
cargo run --bin manage runserver

# Run migrations
cargo run --bin manage migrate
```

### Using reinhardt-admin

Install [reinhardt-web](https://github.com/kent8192/reinhardt-web) CLI tools:
```bash
cargo install reinhardt-admin
```

```bash
# Create a new app
reinhardt-admin startapp myapp
```

## Common Tasks

### Development

```bash
cargo make dev              # Run checks + build + start server
cargo make dev-watch        # Development with auto-reload (requires bacon)
cargo make runserver-watch  # Start server with auto-reload (requires bacon)
```

### Component styles

The v0.4.0-alpha.11 Dashboard follows the Reinhardt Pages Project Template:
each app owns its component stylesheet at
`src/apps/<app>/client/style.rs` and exports it from `client.rs` with
`pub mod style;`. Cross-app primitives belong in
`src/shared/client/style.rs`; app-specific layout and state rules remain in
the owning app stylesheet.

Styles use a crate-unique `#[style_def]` collection built with `style!`.
Components consume the generated `ClassToken` accessors with
`class: STYLES.rule()` and compose state with `+`, which yields a typed
`ClassList`. Imperative DOM and raw HTML fragments must interpolate those
generated accessors rather than construct class values directly. Do not add
UnoCSS, Tailwind, a Node CSS pipeline, or utility-class strings.

`index.html` links the generated
`__reinhardt__/components.css` asset once through `static_url`. Its small
plain-CSS document reset and loading rule are the only styling exception:
they render before generated component hashes are available. All rendered Pages
UI uses generated style tokens.

Extract styles after changing a `style!` definition with:

```bash
cargo run --locked --bin manage -- \
  collectstatic --no-input --package reinhardt-cloud-dashboard --all-features
```

`manifest.json` records the logical `__reinhardt__/components.css` asset and
resolves it to its content-hashed CSS file below `STATIC_ROOT`. The
workspace-root `cargo make runserver` preflight performs static collection and
the Pages server serves the linked asset; the dashboard-local `cargo make
runserver` skips that preflight. Generated static output is a build artifact
and is not committed.

In `page!` forms, place controls inside their visible `label` and style the
label text with a nested `rc-label` span. This preserves native label behavior
without raw-identifier HTML attributes. Import Reinhardt attribute macros and
DI parameter types at module scope so annotations and handler signatures stay
consistent across Dashboard apps.

### Database

```bash
cd dashboard && cargo make makemigrations   # Generate migrations from model changes
cd dashboard && cargo run --bin manage migrate   # Apply checked-in migrations
```

The v0.4.0-alpha.11 migration baseline is a breaking reset with six generated
app initial migrations (`auth`, `clusters`, `default`, `deployments`,
`github`, and `organizations`) plus any generated follow-up migrations (for
example, the GitHub import-lease column). It supports only an empty
PostgreSQL database. Existing migration histories, in-place data migration,
and `fake-initial` compatibility are not provided.

`cd dashboard && cargo make makemigrations` is the authoritative way to
regenerate migrations. Migration files are generated source and must not be
hand-edited.

Cluster creation uses a generated ModelForm that accepts only `name` and
`api_url`; the owning organization, active state, and agent token state are
set by the server.

### Client routes and data

The v0.4.0-alpha.11 client uses one reinhardt-pages `ClientRouter` tree. The
`#[layout]` Dashboard shell renders its child routes through `Outlet`:
`/login` and `/register` are public, while `/`, `/account`, `/clusters`,
`/deployments`, and `/github` are authenticated children.

Direct `page!({ ... })` bodies automatically capture cloneable local values in
alpha.11; use an explicit closure form only when a reusable page factory is
needed. Generated ClientForm submissions have the same typed method on native
and WASM, so auth forms share one action path.

Client reads use Query Client V2 generated server-function query descriptors
with `use_query`. The Launcher or SSR runtime owns the QueryClient; pages do
not install a separate client cache provider. Mutation success invalidates the
affected query keys so dependent views refetch.

The notification WebSocket is mounted at `/ws/notifications`. Configure its
separate `[ws_origin]` allow-list alongside `[cors]`; unlisted browser origins
are rejected during the handshake.

Deployment log selection is canonically represented by
`/deployments?logs=<i64>` and is extracted at the component boundary as
`Query(logs): Query<Option<i64>>`. An omitted `logs` parameter produces no
selection, while a malformed value is rejected by the typed extractor.
Deployment IDs are `i64`; no UUID compatibility adapter is used.

GitHub repository imports use the repository `selected` flag plus the dedicated
`import_claimed_at` timestamp as a bounded 30-minute lease. Repository
synchronization does not renew an active lease. An interrupted import is
reclaimed only when no project row exists, and the conditional timestamp check
prevents one import from clearing another import's lease.

### OAuth account linking

Normal GitHub OAuth sign-in remains independent of account linking. Starting a
link from `/account` creates a signed, short-lived intent bound to the
initiating valid session. The callback links an identity only when its current
`sessionid` still validates and matches that intent's user and session binding;
logout, session rotation, or a session swap invalidates the link flow.
Membership removal is authoritative; reauthentication never recreates a
revoked Personal Organization membership.

### Testing

```bash
cargo make test             # Run all tests (native nextest + WASM browser E2E)
cargo make test-unit        # Run unit tests only
cargo make test-integration # Run integration tests only
cargo make test-watch       # Run tests with auto-reload (requires bacon)
cargo test --test unit test_component_style_contract --all-features
```

### Project Management

```bash
cargo make check            # Check project for common issues
cargo make showurls         # Display all registered URL patterns
cargo make shell            # Run an interactive Rust shell (REPL)
cargo make collectstatic    # Collect static files into STATIC_ROOT
```

### Code Quality

```bash
cargo make fmt-check        # Check code formatting
cargo make fmt-fix          # Fix code formatting
cargo make clippy-check     # Check linting rules
cargo make clippy-fix       # Fix linting issues
cargo make quality          # Run all checks (format + lint)
cargo make quality-fix      # Fix all issues automatically
```

For a style change, run the component source contract above, `cargo make
fmt-check`, `cargo make clippy-check`, and native plus WASM dashboard checks.
After extraction, verify the `manifest.json` mapping for the logical
`__reinhardt__/components.css` asset and that its resolved hashed CSS file is
non-empty.

### Build

```bash
cargo make build            # Build in debug mode
cargo make build-release    # Build in release mode
cargo make ci               # Run CI pipeline (format, lint, build, test)
```

### Help

```bash
cargo make help             # Show all available tasks
```

## Generated with

This project was created using `reinhardt-admin startproject`.
