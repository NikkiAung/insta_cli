# Instagram DM CLI - FastAPI Server

A lightweight Python/FastAPI backend for Instagram DMs, designed for use with a Rust CLI.

Uses [instagrapi](https://github.com/subzeroid/instagrapi) - a modern, actively maintained Instagram Private API library with built-in Pydantic models.

## Features

- ✅ **Read DM inbox** - list all conversations
- ✅ **Read messages** - fetch messages from a thread
- ✅ **Send messages** - to existing threads or by username
- ✅ **Session persistence** - saved to `.ig_session.json`
- ✅ **Pydantic models** - clean, typed API responses
- ✅ **Auto-docs** - Swagger UI at `/docs`

## Setup

### 1. Install dependencies

```bash
# Using pip
pip install -e .

# Or with uv (faster)
uv pip install -e .

# Or manually
pip install fastapi uvicorn instagrapi pydantic pydantic-settings python-dotenv
```

### 2. Set up environment (optional)

```bash
cp .env.example .env
# Edit .env with your Instagram credentials
```

### 3. Run the server

```bash
# With environment variables
IG_USERNAME=your_username IG_PASSWORD=your_password python main.py

# Or just run and login via API
python main.py
```

Server runs on `http://localhost:8000` by default.

## API Endpoints

### Authentication

```bash
# Login
curl -X POST http://localhost:8000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "your_username", "password": "your_password"}'

# Logout
curl -X POST http://localhost:8000/auth/logout

# Check status
curl http://localhost:8000/health
```

### Direct Messages

```bash
# Get inbox (list of conversations)
curl http://localhost:8000/inbox

# Get inbox with limit
curl "http://localhost:8000/inbox?limit=10"

# Get messages from a thread
curl http://localhost:8000/thread/340282366841710300949128...

# Send message to existing thread
curl -X POST http://localhost:8000/thread/340282366841710300949128.../send \
  -H "Content-Type: application/json" \
  -d '{"text": "Hello from CLI!"}'

# Send message to user by username (creates thread if needed)
curl -X POST http://localhost:8000/send/friend_username \
  -H "Content-Type: application/json" \
  -d '{"text": "Hey!"}'
```

### User Search

```bash
# Search for a user
curl http://localhost:8000/user/some_username
```

## API Documentation

Once the server is running, visit:

- **Swagger UI**: http://localhost:8000/docs
- **ReDoc**: http://localhost:8000/redoc

## Response Models

All responses use clean Pydantic models:

```python
# Thread preview (inbox listing)
{
  "id": "340282366841710300949128...",
  "pk": "18123276039123479",
  "thread_title": "friend_username",
  "users": [{"pk": "123", "username": "friend", "full_name": "Friend Name"}],
  "last_activity_at": "2024-01-15T10:30:00Z",
  "is_group": false,
  "has_unread": true,
  "last_message_text": "Hey, how are you?",
  "last_message_type": "text"
}

# Message
{
  "id": "30076199257494728...",
  "user_id": "123456",
  "timestamp": "2024-01-15T10:30:00Z",
  "item_type": "text",
  "text": "Hello!",
  "is_sent_by_viewer": false
}
```

## Session Persistence

The server saves your Instagram session to `.ig_session.json` after successful login. This means:

- You don't need to login every time you restart the server
- Sessions can last for weeks/months
- Delete `.ig_session.json` to force a fresh login

## Important Notes

⚠️ **Security**: Never commit `.env` or `.ig_session.json` to version control.

⚠️ **Rate Limiting**: Instagram will throttle or ban accounts that make too many requests. The client adds delays between requests automatically.

⚠️ **2FA**: Two-factor authentication is not yet supported. You may need to disable it temporarily or use an app password.

⚠️ **Challenges**: If Instagram detects unusual activity, you may need to verify your account in the official app first.

## Project Structure

```
server/
├── main.py           # FastAPI app and routes
├── instagram.py      # Instagram client wrapper
├── models.py         # Pydantic models
├── pyproject.toml    # Dependencies
├── .env.example      # Environment template
└── .gitignore
```
