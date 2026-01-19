"""
Pydantic models for Instagram DM CLI API

These models provide clean, typed responses for the Rust CLI to consume.
They're designed to be simpler than instagrapi's internal models.
"""

from .user_models import User, UserShort
from .message_models import DirectMessage, DirectThread, DirectThreadPreview
from .api_models import (
    LoginRequest,
    LoginResponse,
    PublicKeyResponse,
    SendMessageRequest,
    SendMessageResponse,
    InboxResponse,
    ThreadResponse,
    HealthResponse,
    ErrorResponse,
)

__all__ = [
    # User models
    "User",
    "UserShort",
    # Message models
    "DirectMessage",
    "DirectThread",
    "DirectThreadPreview",
    # API models
    "LoginRequest",
    "LoginResponse",
    "PublicKeyResponse",
    "SendMessageRequest",
    "SendMessageResponse",
    "InboxResponse",
    "ThreadResponse",
    "HealthResponse",
    "ErrorResponse",
]
