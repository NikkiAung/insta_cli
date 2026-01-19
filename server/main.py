"""
Instagram DM CLI - FastAPI Server

A lightweight API server for Instagram DMs, designed for use with a Rust CLI.
Uses instagrapi for Instagram communication.
"""

import logging
import os
from contextlib import asynccontextmanager

from dotenv import load_dotenv
from fastapi import FastAPI, HTTPException, status

# Load environment variables from .env file
load_dotenv()
from fastapi.responses import JSONResponse
from instagrapi.exceptions import LoginRequired

from models import (
    LoginRequest,
    LoginResponse,
    PublicKeyResponse,
    SendMessageRequest,
    SendMessageResponse,
    InboxResponse,
    ThreadResponse,
    HealthResponse,
    ErrorResponse,
    User,
)
from instagram import instagram_client
from crypto import get_public_key_pem, decrypt_password, ensure_keys_exist

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)


@asynccontextmanager
async def lifespan(app: FastAPI):
    """
    Startup and shutdown events.
    Auto-login if credentials are in environment.
    """
    # Startup - ensure RSA keys exist for encryption
    ensure_keys_exist()

    username = os.getenv("IG_USERNAME")
    password = os.getenv("IG_PASSWORD")
    
    if username and password:
        logger.info("Auto-logging in with environment credentials...")
        success, error = instagram_client.login(username, password)
        if success:
            logger.info("Auto-login successful")
        else:
            logger.error("Auto-login failed: %s", error)
    else:
        logger.info("No credentials in env. Use POST /auth/login to authenticate.")
    
    yield
    
    # Shutdown
    logger.info("Shutting down...")


app = FastAPI(
    title="Instagram DM CLI Server",
    description="A lightweight API for Instagram DMs",
    version="0.1.0",
    lifespan=lifespan,
)


# ============================================================================
# Error Handlers
# ============================================================================

@app.exception_handler(LoginRequired)
async def login_required_handler(request, exc):
    return JSONResponse(
        status_code=status.HTTP_401_UNAUTHORIZED,
        content={"success": False, "error": "Not logged in"}
    )


@app.exception_handler(Exception)
async def generic_exception_handler(request, exc):
    logger.error("Unhandled exception: %s", exc)
    return JSONResponse(
        status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
        content={"success": False, "error": str(exc)}
    )


# ============================================================================
# Health Check
# ============================================================================

@app.get("/health", response_model=HealthResponse, tags=["Health"])
async def health_check():
    """Check server status and authentication state"""
    user = instagram_client.get_current_user()
    return HealthResponse(
        status="ok",
        authenticated=instagram_client.is_authenticated(),
        username=user.username if user else None,
    )


# ============================================================================
# Authentication
# ============================================================================

@app.get("/auth/public-key", response_model=PublicKeyResponse, tags=["Auth"])
async def get_public_key():
    """
    Get the server's RSA public key for encrypting credentials.

    The CLI should:
    1. Fetch this public key
    2. Encrypt the password using RSA-OAEP with SHA-256
    3. Send the encrypted password (base64 encoded) in the login request
    """
    return PublicKeyResponse(public_key=get_public_key_pem())


@app.post("/auth/login", response_model=LoginResponse, tags=["Auth"])
async def login(request: LoginRequest):
    """
    Login to Instagram.

    Supports both encrypted and plain text passwords:
    - encrypted_password: RSA-encrypted password (base64) - recommended
    - password: Plain text password (for testing only)

    If successful, session is saved and will be restored on server restart.
    """
    # Determine which password to use
    if request.encrypted_password:
        try:
            password = decrypt_password(request.encrypted_password)
            logger.info("Using encrypted password for login")
        except ValueError as e:
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail="Failed to decrypt password"
            )
    elif request.password:
        logger.warning("Using plain text password - consider using encryption")
        password = request.password
    else:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Either password or encrypted_password is required"
        )

    success, error = instagram_client.login(request.username, password)

    if success:
        return LoginResponse(
            success=True,
            user=instagram_client.get_current_user(),
            message="Login successful"
        )
    else:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail=error or "Login failed"
        )


@app.post("/auth/logout", response_model=dict, tags=["Auth"])
async def logout():
    """Logout and clear saved session"""
    instagram_client.logout()
    return {"success": True, "message": "Logged out"}


# ============================================================================
# Direct Messages
# ============================================================================

@app.get("/inbox", response_model=InboxResponse, tags=["DM"])
async def get_inbox(limit: int = 20):
    """
    Get DM inbox (list of conversations).
    
    Args:
        limit: Number of threads to fetch (default 20, max 100)
    """
    limit = min(max(limit, 1), 100)  # Clamp between 1 and 100
    
    try:
        threads = instagram_client.get_inbox(amount=limit)
        return InboxResponse(success=True, threads=threads)
    except LoginRequired:
        raise
    except Exception as e:
        logger.error("Failed to fetch inbox: %s", e)
        return InboxResponse(success=False, error=str(e))


@app.get("/thread/{thread_id}", response_model=ThreadResponse, tags=["DM"])
async def get_thread(thread_id: str, limit: int = 20):
    """
    Get a conversation thread with messages.
    
    Args:
        thread_id: Thread ID
        limit: Number of messages to fetch (default 20, max 100)
    """
    limit = min(max(limit, 1), 100)
    
    try:
        thread = instagram_client.get_thread(thread_id, amount=limit)
        return ThreadResponse(success=True, thread=thread)
    except LoginRequired:
        raise
    except Exception as e:
        logger.error("Failed to fetch thread %s: %s", thread_id, e)
        return ThreadResponse(success=False, error=str(e))


@app.post("/thread/{thread_id}/send", response_model=SendMessageResponse, tags=["DM"])
async def send_message_to_thread(thread_id: str, request: SendMessageRequest):
    """
    Send a message to an existing thread.
    
    Args:
        thread_id: Thread ID
        request: Message content
    """
    try:
        message = instagram_client.send_message(thread_id, request.text)
        return SendMessageResponse(success=True, message=message)
    except LoginRequired:
        raise
    except Exception as e:
        logger.error("Failed to send message to thread %s: %s", thread_id, e)
        return SendMessageResponse(success=False, error=str(e))


@app.post("/send/{username}", response_model=SendMessageResponse, tags=["DM"])
async def send_message_to_user(username: str, request: SendMessageRequest):
    """
    Send a message to a user by username.
    
    Creates a new thread if one doesn't exist with this user.
    
    Args:
        username: Target username (without @)
        request: Message content
    """
    # Remove @ if present
    username = username.lstrip("@")
    
    try:
        message = instagram_client.send_message_to_user(username, request.text)
        return SendMessageResponse(success=True, message=message)
    except LoginRequired:
        raise
    except Exception as e:
        logger.error("Failed to send message to %s: %s", username, e)
        return SendMessageResponse(success=False, error=str(e))


# ============================================================================
# User Search
# ============================================================================

@app.get("/user/{username}", response_model=dict, tags=["User"])
async def search_user(username: str):
    """
    Search for a user by username.
    
    Args:
        username: Username to search for (without @)
    """
    username = username.lstrip("@")
    
    try:
        user = instagram_client.search_user(username)
        if user:
            return {"success": True, "user": user}
        else:
            raise HTTPException(
                status_code=status.HTTP_404_NOT_FOUND,
                detail=f"User '{username}' not found"
            )
    except LoginRequired:
        raise
    except HTTPException:
        raise
    except Exception as e:
        logger.error("Failed to search user %s: %s", username, e)
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail=str(e)
        )


# ============================================================================
# Main
# ============================================================================

if __name__ == "__main__":
    import uvicorn
    
    port = int(os.getenv("PORT", 8000))
    
    print(f"""
🚀 Instagram DM CLI Server
━━━━━━━━━━━━━━━━━━━━━━━━━━

Starting on http://localhost:{port}

Endpoints:
  GET  /health                - Check server status
  GET  /auth/public-key       - Get encryption public key
  POST /auth/login            - Login (encrypted or plain password)
  POST /auth/logout           - Logout and clear session
  GET  /inbox                 - Get DM inbox
  GET  /thread/{{thread_id}}    - Get messages in a thread
  POST /thread/{{thread_id}}/send - Send message to thread
  POST /send/{{username}}       - Send message to user
  GET  /user/{{username}}       - Search for a user

Docs: http://localhost:{port}/docs
━━━━━━━━━━━━━━━━━━━━━━━━━━
""")
    
    uvicorn.run(
        "main:app",
        host="0.0.0.0",
        port=port,
        reload=True,
        log_level="info"
    )