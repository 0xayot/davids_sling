# David's Sling (WIP)

A small and simple trading bot.

1. Execute `cargo run` to start the server

1. Visit [localhost:8000](http://localhost:8000) in browser

Setup Migrations

```
sea migrate generate "migration name"
```

Run Migrations

```
sea migrate up
```

Generate Entities

```
sea generate entity -o entity/src --lib
```

Run server with auto-reloading:

```bash
cargo install systemfd cargo-watch
systemfd --no-pid -s http::9000 -- cargo watch -x run
```

References

- [Actix Docs]("")
- [Oliver](https://oliverjumpertz.com/blog/how-to-build-a-powerful-graphql-api-with-rust/)
- [Pyk](https://pyk.sh/rust-seaorm-insert-select-update-and-delete-rows-in-postgresql-tables?source=more_series_bottom_blogs)
- [Sea Docs](https://www.sea-ql.org/sea-orm-tutorial/ch01-03-migration-api.html)

- [Actix Examples](https://github.com/actix/examples/blob/master/background-jobs/src/ephemeral_jobs.rs)
