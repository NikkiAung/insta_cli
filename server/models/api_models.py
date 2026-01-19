"""API request and response Pydantic models"""

from typing import Optional
from pydantic import BaseModel, Field

from .user_models import User
from .message_models import DirectMessage, DirectThread, DirectThreadPreview


class LoginRequest(BaseModel):
    """Login request body - supports both plain and encrypted passwords"""
    username: str
    password: str = Field(default="", description="Plain text password (for testing only)")
    encrypted_password: str = Field(default="", description="RSA-encrypted password (base64)")


class PublicKeyResponse(BaseModel):
    """Public key response for CLI encryption"""
    public_key: str = Field(description="RSA public key in PEM format")


class LoginResponse(BaseModel):
    """Login response"""
    success: bool
    user: Optional[User] = None
    message: Optional[str] = None


class SendMessageRequest(BaseModel):
    """Send message request body"""
    text: str


class SendMessageResponse(BaseModel):
    """Send message response"""
    success: bool
    message: Optional[DirectMessage] = None
    error: Optional[str] = None


class InboxResponse(BaseModel):
    """Inbox listing response"""
    success: bool
    threads: list[DirectThreadPreview] = Field(default_factory=list)
    error: Optional[str] = None


class ThreadResponse(BaseModel):
    """Single thread with messages response"""
    success: bool
    thread: Optional[DirectThread] = None
    error: Optional[str] = None


class HealthResponse(BaseModel):
    """Health check response"""
    status: str
    authenticated: bool
    username: Optional[str] = None


class ErrorResponse(BaseModel):
    """Error response"""
    success: bool = False
    error: str
