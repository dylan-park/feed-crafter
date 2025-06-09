# Feed Crafter

## About

The purpose of this project is to create a system which can easily create, serve, and manage a custom RSS feed in a simple singular package. Specifically, I am not looking to create an RSS feed representation of web content, instead my goal is to store custom messages in a standardized format that can be used by other projects. I chose RSS as a technology because it's an established, standardized format with a lot of existing tooling made for digesting it (like RSS Readers), and it is very easy to programmatically navigate. Some examples could include:

- A system alert queue system accessible by a generic RSS reader.
- A house-wide notice board, multiple people are able to view, edit, and add to.

This system provides a web interface, as well as API endpoints for managing items. You can view all the existing messages, edit and delete messages, and add new ones.

## How To Run
To run via cargo locally:
```bash
git clone https://github.com/netbymatt/ws4kp.git
cd feed-crafter
cargo run -r
```
Make sure you have all the environment variables set, or a .env file like this
```dotenv
RUST_LOG=info

SERVER_ADDRESS=127.0.0.1
SERVER_PORT=3000

CHANNEL_TITLE="Test Channel"
CHANNEL_LINK=http://example.com
CHANNEL_DESCRIPTION="An RSS feed."

MAX_ITEM_AGE_SECONDS=0
CLEANUP_INTERVAL_SECONDS=3600
```

To run via Docker Compose (docker-compose.yaml):
```yaml
services:
  feed-crafter:
    build: .
    container_name: feed-crafter
    network_mode: bridge
    environment:
      - RUST_LOG=info
      - SERVER_ADDRESS=0.0.0.0
      - SERVER_PORT=3000
      - CHANNEL_TITLE="RSS Channel" # change to name the RSS feed
      - CHANNEL_LINK=http://localhost:3000 # required by RSS standard, can be modified if you like
      - CHANNEL_DESCRIPTION="An RSS feed." # change to describe the RSS feed
      - MAX_ITEM_AGE_SECONDS=0 #change to the time in seconds any feed item should stay in the system (0 for indefinite)
      - CLEANUP_INTERVAL_SECONDS=3600 # change how ofter the system checks for old messages (must be <=MAX_ITEM_AGE_SECONDS for proper timing to work, set accordingly)
    volumes:
      - ./feed:/app/feed
    ports:
      - 3000:3000 # change the first 3000 to change the port for your local network
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-fsS", "http://localhost:3000/health"]
      interval: 60s
      timeout: 15s
      start_period: 10s
      retries: 5
```

```bash
docker compose up -d
```

To run via Docker:
```bash
# First, build the image
docker build -t feed-crafter .

# Then run the container
docker run -d \
  --name feed-crafter \
  --network bridge \
  --restart unless-stopped \
  -p 3000:3000 \
  -v ./feed/feed.xml:/app/feed/feed.xml \
  -e RUST_LOG=info \
  -e SERVER_ADDRESS=0.0.0.0 \
  -e SERVER_PORT=3000 \
  -e CHANNEL_TITLE="RSS Channel" \
  -e CHANNEL_LINK=http://localhost:3000 \
  -e CHANNEL_DESCRIPTION="An RSS feed." \
  -e MAX_ITEM_AGE_SECONDS=0 \
  -e CLEANUP_INTERVAL_SECONDS=3600 \
  feed-crafter
```

## How To Use

When Feed Crafter is first run, an RSS feed file is created based on environment variables (either in your system or in the .env file). If it finds an existing file it will use that instead.

### Web Interface

Open in your web browser: http://localhost:3000/

The home page displays some information about the loaded feed, as well as a list of all of the current items in the feed. If you click on the *Add New Item* button, you are taken to a page where you can add a new item with a title, and an optional description and link. You can click on the *Delete* button on any single item to remove it from the feed. You can also click on the *Edit* button to edit any existing item.

### API

- **GET** /api/items - Returns all RSS items in JSON format.

#### Response:

```json
{
  "success": true,
  "data": [
    {
      "id": "uuid-here",
      "title": "Item Title",
      "description": "Item description",
      "link": "https://example.com",
      "pub_date": "Mon, 01 Jan 2024 12:00:00 +0000"
    }
  ],
  "message": "Items retrieved successfully"
}
```

- **POST** /api/items - Creates a new RSS item.

#### Request:

```json
{
  "title": "New Item Title",
  "description": "Item description",  // optional
  "link": "https://example.com"  // optional
}
```

#### Response:

```json
{
  "success": true,
  "data": {
    "id": "new-uuid-here",
    "title": "New Item Title",
    "description": "Item description",
    "link": "https://example.com",
    "pub_date": "Mon, 01 Jan 2024 12:00:00 +0000"
  },
  "message": "Item added successfully"
}
```

- **DELETE** /api/items/:id - Removes an RSS item by its ID.

#### Response:

```json
{
  "success": true,
  "data": null,
  "message": "Item deleted successfully"
}
```

- **PUT** /api/items/:id - Edits an RSS item by its ID.

#### Request:

```json
{
  "title": "Edited Item Title",
  "description": "Edited Item description",  // optional
  "link": "https://example.com"  // optional
}
```

#### Response:

```json
{
  "success": true,
  "data": {
    "id": "new-uuid-here",
    "title": "Edited Item Title",
    "description": "Edited Item description",
    "link": "https://example.com",
    "pub_date": "Mon, 01 Jan 2024 12:00:00 +0000"
  },
  "message": "Item updated successfully"
}
```

## Disclaimer

This project (currently) has absolutely 0 promise of security or user authentication. This means if someone has access to the port the software is running on, they have complete and total control of your RSS feed, including viewing, adding, and deleting items. 100% of your security comes from your firewall setup.

For my use case, I have little interest in pursuing proper authentication, but it may be something I look into in the future.