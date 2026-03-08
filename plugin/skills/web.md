# /cai web

Launch local web dashboard for visual analytics.

## Usage

```
/cai web [options]
```

## Options

| Option | Description | Default |
|--------|-------------|---------|
| `--port, -p` | Port number | 3030 |
| `--host` | Bind address | 127.0.0.1 |
| `--open` | Open in browser | false |

## Features

- **Dashboard** - Overview with charts and metrics
- **Query Builder** - Visual SQL query builder
- **Timeline** - Interactive timeline visualization
- **Agents** - Per-agent activity breakdown
- **Repositories** - Repository-centric analysis

## Examples

```bash
# Launch on default port
/cai web

# Custom port and open browser
/cai web --port 8080 --open

# Listen on all interfaces
/cai web --host 0.0.0.0
```

## Access

Open http://localhost:3030 in your browser.

## See Also

- `/cai tui` - Terminal UI
- `/cai query` - SQL queries
