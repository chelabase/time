# chelabase-time

Timestamps, dates and duration parsing for [Chelabase](https://github.com/chelabase/chelabase)
and its Neon fork: newtypes over [jiff](https://github.com/BurntSushi/jiff) that write and read
the same JSON, Postgres and SQL forms chrono did, and a `parse_duration` with humantime's
grammar. The crate docs (`src/lib.rs`) are the contract.

## Use

It isn't published to crates.io. Depend on it by git `rev`, with the tag in a comment:

```toml
chelabase-time = { git = "https://github.com/chelabase/time", rev = "<commit sha>" } # v0.1.0
```

Tags are cut only on `main` (rebase-merge), so a pinned rev stays reachable.

Features (none are on by default):

- `sqlx`: Postgres codecs for `Timestamp` and `Date` (sqlx and jiff-sqlx);
- `openapi`: utoipa.

## Develop

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test --workspace --no-default-features
cargo deny check && bash scripts/deny-bans.sh
```

The live Postgres tests are ignored by default. Run them with a Postgres you can create a
schema in (each test drops its own schema afterwards):

```bash
CHELABASE_METADATA_DATABASE_URL=postgres://user:password@127.0.0.1:5432/db \
  cargo test --all-features --test pg_live -- --include-ignored
```

`oracle/` is an unpublished workspace member that compares this crate with chrono 0.4.45 and
humantime 2.2.0. It is the only place `deny.toml` lets them in.

## License

Apache-2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
