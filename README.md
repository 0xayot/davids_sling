# David's Sling (WIP)

A small and simple trading bot.

1. Execute `cargo run` to start the server

1. Visit [localhost:8000](http://localhost:8000) in browser

Run server with auto-reloading:

```bash
cargo install systemfd cargo-watch
systemfd --no-pid -s http::8000 -- cargo watch -x run
```
