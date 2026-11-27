# chelabase-time

Timestamps, dates and duration parsing for [Chelabase](https://github.com/chelabase/chelabase)
and its Neon fork: newtypes over [jiff](https://github.com/BurntSushi/jiff) that write and read
the same JSON, Postgres and SQL forms chrono did, and a `parse_duration` with humantime's
grammar. For the fork, thin jiff wrappers: `human` (serde and clap for `std::time::Duration`
in jiff's friendly format, still reading humantime's spellings such as `7days`) and `rfc3339`
(`SystemTime` through `jiff::Timestamp`). The crate docs (`src/lib.rs`) are the contract.

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
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo test --no-default-features
cargo deny check
```

The live Postgres tests are ignored by default. Run them with a Postgres you can create a
schema in (each test drops its own schema afterwards):

```bash
CHELABASE_METADATA_DATABASE_URL=postgres://user:password@127.0.0.1:5432/db \
  cargo test --all-features --test pg_live -- --include-ignored
```

## License

Apache-2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
