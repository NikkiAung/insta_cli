"""Message and thread Pydantic models"""

from datetime import datetime
from typing import Optional
from pydantic import BaseModel, Field

from .user_models import UserShort


class DirectMessage(BaseModel):
    """A single direct message"""
    id: str = Field(description="Message ID")
    user_id: Optional[str] = Field(default=None, description="Sender's user ID")
    timestamp: datetime
    item_type: str = Field(description="Type: text, media, link, reel_share, etc.")
    text: Optional[str] = None
    is_sent_by_viewer: bool = False

    # For non-text content
    media_url: Optional[str] = None
    media_type: Optional[str] = None  # photo, video
    link_url: Optional[str] = None
    link_title: Optional[str] = None

    # Reactions
    reactions: Optional[list[dict]] = None


class DirectThread(BaseModel):
    """A DM conversation thread"""
    id: str = Field(description="Thread ID")
    pk: str = Field(description="Thread primary key")
    thread_title: str = Field(default="", description="Thread name (for groups) or username")
    users: list[UserShort] = Field(default_factory=list, description="Participants")
    last_activity_at: Optional[datetime] = None
    is_group: bool = False
    is_muted: bool = False
    has_unread: bool = False

    # Last message preview
    last_message: Optional[DirectMessage] = None

    # Messages (only populated when fetching single thread)
    messages: list[DirectMessage] = Field(default_factory=list)


class DirectThreadPreview(BaseModel):
    """Inbox thread preview (without full messages)"""
    id: str
    pk: str
    thread_title: str = ""
    users: list[UserShort] = Field(default_factory=list)
    last_activity_at: Optional[datetime] = None
    is_group: bool = False
    is_muted: bool = False
    has_unread: bool = False
    last_message_text: Optional[str] = None
    last_message_type: Optional[str] = None
    last_message_timestamp: Optional[datetime] = None
