# cai-web

Web dashboard interface for CAI.

## Overview

`cai-web` provides a local web server with real-time updates for browsing CAI data.

## Features

- **Dashboard**: Overview and statistics
- **Query Interface**: Execute SQL queries
- **Entry Browser**: Browse all entries
- **Real-time Updates**: WebSocket for live data
- **REST API**: Programmatic access

## Usage

### Standalone

```bash
cai web --port 3000
```

### In Code

```rust
use cai_storage::MemoryStorage;
use cai_web::{run, Config};

let storage = std::sync::Arc::new(MemoryStorage::new());
let config = Config {
    port: 3000,
    host: "127.0.0.1".to_string(),
};
run(storage, config).await?;
```

## API Endpoints

- `GET /` - Web dashboard
- `GET /api/query` - Execute query
- `GET /api/entries` - List entries
- `GET /api/entries/:id` - Get entry
- `WS /ws` - WebSocket for updates

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
cai-web = { path = "../cai-web" }
```

## Design Decisions

- **Axum**: Async web framework
- **WebSocket**: Real-time updates
- **HTMX**: Interactive frontend (optional)

## Testing

```bash
cargo test -p cai-web
```

## License

MIT OR Apache-2.0
