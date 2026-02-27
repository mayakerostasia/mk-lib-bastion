# ⚠️ DEPRECATED - simian-bin-cfg-creator

This crate has been **merged into `simian-config`**.

## Migration Guide

The configuration generation binary has been consolidated into the `simian-config` crate as the `cfg-gen` binary.

### Old Usage
```bash
cargo build -p simian-bin-cfg-creator
cargo run -p simian-bin-cfg-creator -- surreal --uri "ws://localhost:8000" ...
```

### New Usage
```bash
# Build and run cfg-gen from simian-config
cargo run -p simian-config --bin cfg-gen -- surreal --uri "ws://localhost:8000" ...

# Or build the binary
cargo build -p simian-config --bin cfg-gen
```

## Why the Consolidation?

The configuration generation logic was tightly coupled to `simian-config`. Merging the binary into the crate eliminates:
- Redundant dependencies
- Maintenance burden across two separate crates
- Confusion about which binary to use

## Documentation

For detailed usage of `cfg-gen`, see:
- [`simian-config/README.md`](../simian-config/README.md) - Full configuration guide
- Main project [`README.md`](../README.md) - Quick start examples

## See Also

- **simian-config** - Configuration loading library with `cfg-gen` binary
- **simian-surreal-client** - SurrealDB client using generated configs
