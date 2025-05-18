---
applyTo: "**/*.rs"
---
# Embedded Rust coding standards

- Use `serde` and `postcard` for serialization/deserialization of data structures
- Use `embassy` for async I/O and concurrency in this embedded application.
- Do not use `std` library features, as this is an embedded application
- `esp-alloc` is available for heap allocation, but use it sparingly
