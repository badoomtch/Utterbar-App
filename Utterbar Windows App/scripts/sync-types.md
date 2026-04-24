# Type sync

`ts-rs` emits TypeScript definitions from the Rust `domain/` structs into `src/generated/` as a side-effect of running tests.

```bash
cd src-tauri
cargo test --lib
```

Commit the resulting `src/generated/*.ts` files. They are consumed by the frontend at the seams where Rust types cross into TypeScript (events, commands, domain payloads).
