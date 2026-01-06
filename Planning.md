# Backend Plan
This backend will be consumed by [Research Tracker KMP](https://github.com/mmnessim/researchtrackerkotlin), which is a mobile app in Kotlin targeting iOS, Android, and JVM desktop.

The mobile frontend currently consumes a [Ktor backend](https://github.com/mmnessim/research-tracker-backend). This project is intended to be a drop-in replacement for the current backend.

# RSS Feeds
The Ktor backend stores the feeds and their associated sources as code (list of maps), so updates to the list require rebuilding the entire project. The Rust backend watches a json file `feeds.json` and will update the in memory list as changes as made.

The Ktor backend asynchronously fetches from all feeds once every 15 minutes. I will probably have the Rust backend do the same.

Articles are parsed from feeds and stored in a database (H2 in Ktor, Sqlite in Rust).

# Endpoints
In order to be a drop-in replacement, the following endpoints need to be implemented

- `/search/{query}` 
  - Response 
  ```json
    [
        "rssSource": "String", // This should be changed to rss_source
        "guid": "String?", // This can be made no longer nullable in Rust
        "link": "String",
        "description": "String?",
        "pubDate": "String?" // This will later be Int (epoch millis),
        "categories": "String" // Comma separated 
    ]
  ```
- `/all`
  - Response 
  ```json
    {
        "num_articles": Int,
        "num_sources": Int
    }
  ```
- `/health`
  - Response -> 200 OK 

There may be additional endpoints for specific sources, extra query params, etc. but these will be lower priority.

# Flow 
- App launch
  - Initialize database, apply migrations 
  - launch background tasks
    - Watch `feeds.json`
      - On change -> update in memory list of feeds?
    - Poll feeds every 15 minutes 
    - Save articles in database
    - Delete articles older than 30 days - maybe once a day or so
  - Register endpoints and listen

# Deployment and Updates
Will be deployed to a Docker container in a Linux Virtual Machine on my home server. Cloudflare will provide DNS services. I will use a `docker-compose.yml` with `restart: unless-stopped`. `feeds.json` will be in a Docker volume for updating without restarting/rebuilding the container.

Ideally there could be some kind of redundancy with Docker Swarm or something. Upon launching the app, I will consider using a VPS or some other cloud hosting for higher availability. 

# Status, reworks, and planning
Right now everything is pretty ad hoc and minimally implemented. This section will contain notes and plans for specific reworks. 