"""Parsing helpers - convert instagrapi types to our Pydantic models"""

from instagrapi.types import (
    DirectThread as IGDirectThread,
    DirectMessage as IGDirectMessage,
)

from models import (
    User,
    UserShort,
    DirectMessage,
    DirectThread,
    DirectThreadPreview,
)


def parse_user(user) -> User:
    """Convert instagrapi user to our User model"""
    return User(
        pk=str(user.pk),
        username=user.username,
        full_name=user.full_name or "",
        profile_pic_url=str(user.profile_pic_url) if user.profile_pic_url else None,
        is_private=getattr(user, 'is_private', False),
        is_verified=getattr(user, 'is_verified', False),
    )


def parse_user_short(user) -> UserShort:
    """Convert instagrapi user to our UserShort model"""
    return UserShort(
        pk=str(user.pk),
        username=user.username,
        full_name=user.full_name or "",
        profile_pic_url=str(user.profile_pic_url) if user.profile_pic_url else None,
    )


def parse_message(msg: IGDirectMessage, logged_in_user_pk: str | None = None) -> DirectMessage:
    """Convert instagrapi DirectMessage to our model"""
    # Determine if sent by viewer
    is_sent_by_viewer = False
    if logged_in_user_pk and msg.user_id:
        is_sent_by_viewer = str(msg.user_id) == logged_in_user_pk

    # Extract media info if present
    media_url = None
    media_type = None
    if msg.media:
        media_type = "video" if getattr(msg.media, 'video_url', None) else "photo"
        media_url = str(msg.media.video_url or msg.media.thumbnail_url or "")

    # Extract link info
    link_url = None
    link_title = None
    if msg.item_type == "link" and hasattr(msg, 'link'):
        link_url = getattr(msg.link, 'url', None)
        link_title = getattr(msg.link, 'title', None)

    return DirectMessage(
        id=str(msg.id),
        user_id=str(msg.user_id) if msg.user_id else None,
        timestamp=msg.timestamp,
        item_type=msg.item_type or "unknown",
        text=msg.text,
        is_sent_by_viewer=is_sent_by_viewer,
        media_url=media_url,
        media_type=media_type,
        link_url=link_url,
        link_title=link_title,
    )


def parse_thread(thread: IGDirectThread, logged_in_user_pk: str | None = None) -> DirectThread:
    """Convert instagrapi DirectThread to our model (with messages)"""
    users = [parse_user_short(u) for u in thread.users]
    messages = [parse_message(m, logged_in_user_pk) for m in (thread.messages or [])]

    # Build thread title from usernames if not set
    thread_title = thread.thread_title or ""
    if not thread_title and users:
        thread_title = ", ".join(u.username for u in users)

    return DirectThread(
        id=str(thread.id),
        pk=str(thread.pk),
        thread_title=thread_title,
        users=users,
        last_activity_at=getattr(thread, 'last_activity_at', None),
        is_group=thread.is_group if hasattr(thread, 'is_group') else len(users) > 1,
        is_muted=getattr(thread, 'muted', False),
        has_unread=getattr(thread, 'has_newer', False),
        messages=messages,
    )


def parse_thread_preview(thread: IGDirectThread) -> DirectThreadPreview:
    """Convert instagrapi DirectThread to our preview model (for inbox)"""
    users = [parse_user_short(u) for u in thread.users]

    # Build thread title from usernames if not set
    thread_title = thread.thread_title or ""
    if not thread_title and users:
        thread_title = ", ".join(u.username for u in users)

    # Get last message info
    last_msg_text = None
    last_msg_type = None
    # Use last_activity_at as primary timestamp (more reliable for inbox)
    # Fall back to message timestamp if available
    last_msg_timestamp = getattr(thread, 'last_activity_at', None)

    if thread.messages:
        last_msg = thread.messages[0]
        last_msg_type = last_msg.item_type or "unknown"
        # Use message timestamp if last_activity_at not available
        if not last_msg_timestamp:
            last_msg_timestamp = last_msg.timestamp

        if last_msg.text:
            last_msg_text = last_msg.text
        else:
            # Generate preview for non-text messages
            type_previews = {
                "media": "Photo",
                "video": "Video",
                "reel_share": "Shared Reel",
                "story_share": "Shared Story",
                "media_share": "Shared Post",
                "voice_media": "Voice Message",
                "animated_media": "GIF",
                "link": "Link",
                "like": "Liked",
            }
            last_msg_text = type_previews.get(last_msg_type, f"[{last_msg_type}]")

    return DirectThreadPreview(
        id=str(thread.id),
        pk=str(thread.pk),
        thread_title=thread_title,
        users=users,
        last_activity_at=getattr(thread, 'last_activity_at', None),
        is_group=thread.is_group if hasattr(thread, 'is_group') else len(users) > 1,
        is_muted=getattr(thread, 'muted', False),
        has_unread=getattr(thread, 'has_newer', False),
        last_message_text=last_msg_text,
        last_message_type=last_msg_type,
        last_message_timestamp=last_msg_timestamp,
    )
