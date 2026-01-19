# RSS Backend

A high-performance Rust backend service for the
[RSSTracker](https://github.com/mmnessim/RSSTracker) Kotlin Multiplatform mobile
app. This service polls RSS feeds, stores articles, and provides a REST API for
mobile clients targeting iOS, Android, and JVM platforms.

## Features

- **RSS Feed Polling**: Automatically monitors and fetches updates from RSS
  feeds
- **Multi-Database Support**: SQLite (default) and PostgreSQL options
- **Search Integration**: Full-text search powered by Meilisearch
- **RESTful API**: Clean HTTP endpoints for mobile app integration
- **Docker Ready**: Complete containerization with Docker Compose
- **High Performance**: Built with Axum for fast async HTTP handling
- **Content Sanitization**: HTML cleaning and text extraction for
  mobile-friendly content

## Tech Stack

- **Language**: Rust 2024 Edition
- **Web Framework**: Axum
- **Database**: SQLite/PostgreSQL with SQLx
- **Search Engine**: Meilisearch
- **RSS Parsing**: feed-rs
- **Runtime**: Tokio async runtime
- **Containerization**: Docker & Docker Compose

## Quick Start

### Prerequisites

- Docker & Docker Compose
- Git

### Deploy with Docker Compose

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd rss_backend_two
   ```

2. **Start the services**:
   ```bash
   docker-compose up -d
   ```

This will start:

- RSS Backend API on port `3000`
- PostgreSQL database on port `5432`
- Meilisearch on port `7700`

### Local Development

#### With SQLite (Default)

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Install system dependencies**:
   ```bash
   # macOS
   brew install pkg-config openssl sqlite3

   # Ubuntu/Debian
   sudo apt-get install pkg-config libssl-dev libsqlite3-dev
   ```

3. **Run the application**:
   ```bash
   cargo run
   ```

#### With PostgreSQL

1. **Set up PostgreSQL** and create a database

2. **Set the database URL**:
   ```bash
   export DATABASE_URL="postgres://username:password@localhost/rss_db"
   ```

3. **Build with PostgreSQL feature**:
   ```bash
   cargo run --no-default-features --features postgres
   ```

## Configuration

### Environment Variables

- `DATABASE_URL`: Database connection string
- `RUST_LOG`: Logging level (info, debug, trace, etc.)

### RSS Feeds

RSS feeds are configured in [feeds.json](feeds.json). The file contains an array
of feed objects:

```json
[
    {
        "source": "Source Name",
        "url": "https://example.com/feed.xml"
    }
]
```

The service automatically watches this file for changes and updates the feed
list without restart.

## API Endpoints

### Feed Management

- `GET /feeds` - List all configured RSS feeds
- `GET /sources` - Get unique source names

### Statistics

- `GET /stats` - Get feed and article counts

### Articles

- `GET /search` - Search articles (Meilisearch integration)

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Mobile App    │────│  RSS Backend    │────│   Meilisearch   │
│ (iOS/Android/   │    │    (Axum)       │    │  (Full-text     │
│     JVM)        │    │                 │    │   Search)       │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                              │
                        ┌─────────────────┐
                        │    Database     │
                        │(SQLite/Postgres)│
                        │                 │
                        └─────────────────┘
```

### Key Components

- **Feed Watcher**: Monitors `feeds.json` and polls RSS sources
- **Article Storage**: Persists articles with deduplication
- **Search Indexing**: Indexes articles in Meilisearch for fast searching
- **REST API**: Provides endpoints for the mobile app

## Project Structure

```
src/
├── main.rs              # Application entry point
├── watcher.rs           # RSS feed monitoring and polling
├── data/
│   ├── crud.rs          # Database operations
│   ├── models.rs        # Data models
│   └── migrations/      # Database migrations
├── routes/
│   ├── handlers.rs      # HTTP request handlers
│   └── mw.rs            # Middleware
└── util/                # Utility functions
```

## Development

### Database Migrations

Migrations are automatically applied on startup. To add new migrations:

```bash
sqlx migrate add <migration_name>
```

### Testing

```bash
cargo test
```

### Building for Production

```bash
cargo build --release
```

## Deployment Options

### 1. Docker Compose (Recommended)

Use the provided [docker-compose.yml](docker-compose.yml) for a complete stack
with database and search engine.

### 2. Container Only

```bash
docker build -t rss-backend .
docker run -p 3000:3000 -e DATABASE_URL="..." rss-backend
```

### 3. Native Binary

Build and deploy the binary directly on your server with the database of your
choice.

## Related Projects

- [RSSTracker Mobile App](https://github.com/mmnessim/RSSTracker) - Kotlin
  Multiplatform mobile client
