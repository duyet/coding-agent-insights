# cai-web - Claude Code Instructions

## Crate Purpose

`cai-web` provides a local web interface for CAI:

- REST API for querying entries
- Interactive HTML/JS dashboard
- WebSocket support for real-time updates
- Static file serving

## Architecture

### Module Structure

```
src/
├── lib.rs           - Public API, Config, AppState
├── server.rs        - run() function, HTTP server setup
├── handlers.rs      - HTTP request handlers
├── api.rs           - API router, types
└── dashboard.html   - Embedded HTML dashboard
```

### Request Flow

```
Client Request
    ↓
Axum Router
    ↓
Handler (handlers.rs)
    ↓
Storage/Query Engine
    ↓
Response (JSON/HTML)
```

### AppState

```rust
pub struct AppState<S: Storage + ?Sized> {
    pub storage: std::sync::Arc<S>,
}
```

## Common Tasks

### Adding a New API Endpoint

1. Add handler in `handlers.rs`:
```rust
pub async fn my_handler<S>(
    State(state): State<AppState<S>>,
) -> Result<Json<MyResponse>>
where
    S: Storage + 'static,
{
    let entries = state.storage.query(None).await?;
    // Process entries
    Ok(Json(MyResponse { /* ... */ }))
}
```

2. Add route in `api.rs::router()`:
```rust
pub fn router<S>() -> Router<AppState<S>>
where
    S: Storage + 'static,
{
    Router::new()
        .route("/my-endpoint", get(handlers::my_handler))
        // ... other routes
}
```

3. Define response type:
```rust
#[derive(Serialize)]
pub struct MyResponse {
    pub field: String,
}
```

### Adding WebSocket Support

1. Add WebSocket upgrade handler:
```rust
pub async fn ws_handler<S>(
    State(state): State<AppState<S>>,
    ws: WebSocketUpgrade,
) -> Response
where
    S: Storage + 'static,
{
    ws.on_upgrade(|socket| handle_ws(socket, state.storage))
}

async fn handle_ws<S>(mut socket: WebSocket, storage: Arc<S>)
where
    S: Storage + 'static,
{
    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(text) => {
                // Handle message, send response
                socket.send(Message::Text(response)).await.ok();
            }
            _ => {}
        }
    }
}
```

2. Add route:
```rust
.route("/ws", get(ws_handler))
```

### Customizing the Dashboard

Edit `dashboard.html`:

- Built with vanilla JS (no framework required)
- Uses fetch() for API calls
- Can be replaced with SPA framework

## API Endpoints

### GET /

Returns the HTML dashboard.

### GET /api/query?sql=...

Executes SQL query and returns entries.

**Request**: `?sql=SELECT * FROM entries LIMIT 10`

**Response**: Array of Entry objects

### GET /api/stats

Returns statistics about entries.

**Response**:
```json
{
  "total": 100,
  "by_source": { "Claude": 50, "Git": 30, "Codex": 20 },
  "date_range": ["2024-01-01T00:00:00Z", "2024-12-31T23:59:59Z"]
}
```

### GET /api/entries

Lists all entries (supports pagination).

### GET /api/entries/:id

Gets single entry by ID.

## WebSocket Protocol

**Client → Server**:
```json
{"type": "query", "sql": "SELECT * FROM entries"}
```

**Server → Client**:
```json
{"type": "result", "data": [...]}
```

## Testing Patterns

### Test API Handlers

```rust
#[tokio::test]
async fn test_query_handler() {
    let storage = Arc::new(MockStorage::new());
    let state = AppState { storage };

    let params = QueryParams { sql: "SELECT * FROM entries".to_string() };
    let response = query_handler(State(state), AxumQuery(params)).await;

    assert!(response.is_ok());
}
```

### Test WebSocket

```rust
#[tokio::test]
async fn test_websocket() {
    // Create test WebSocket connection
    // Send messages
    // Verify responses
}
```

## Configuration

```rust
pub struct Config {
    pub port: u16,
    pub host: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 3000,
            host: "127.0.0.1".to_string(),
        }
    }
}
```

## Dependencies

- `axum` - Async web framework
- `tokio` - Async runtime
- `cai-storage` - Storage trait
- `cai-query` - Query engine
- `serde_json` - JSON serialization

## Security Notes

- Default binds to 127.0.0.1 (localhost only)
- No authentication by default
- For production: add auth middleware, TLS

## Getting Help

- See `README.md` for API endpoints
- Check `handlers.rs` for request handling
- Review `dashboard.html` for frontend patterns
- Look at `api.rs` for router configuration
