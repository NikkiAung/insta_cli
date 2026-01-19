"""Direct message operations for Instagram client"""

import logging
from typing import Optional

from instagrapi import Client
from instagrapi.exceptions import LoginRequired

from models import (
    User,
    DirectMessage,
    DirectThread,
    DirectThreadPreview,
)
from .parsers import parse_user, parse_message, parse_thread, parse_thread_preview

logger = logging.getLogger(__name__)


def get_inbox(
    client: Client,
    logged_in_user: Optional[User],
    amount: int = 20
) -> list[DirectThreadPreview]:
    """
    Get DM inbox (list of threads).

    Args:
        client: Instagram client
        logged_in_user: Currently logged in user
        amount: Number of threads to fetch (default 20)

    Returns:
        List of thread previews
    """
    if not logged_in_user:
        raise LoginRequired("Not logged in")

    threads = client.direct_threads(amount=amount)
    return [parse_thread_preview(t) for t in threads]


def get_thread(
    client: Client,
    logged_in_user: Optional[User],
    thread_id: str,
    amount: int = 20
) -> DirectThread:
    """
    Get a thread with its messages.

    Args:
        client: Instagram client
        logged_in_user: Currently logged in user
        thread_id: Thread ID
        amount: Number of messages to fetch (default 20)

    Returns:
        Thread with messages
    """
    if not logged_in_user:
        raise LoginRequired("Not logged in")

    thread = client.direct_thread(thread_id=int(thread_id), amount=amount)
    return parse_thread(thread, logged_in_user.pk)


def send_message(
    client: Client,
    logged_in_user: Optional[User],
    thread_id: str,
    text: str
) -> DirectMessage:
    """
    Send a message to an existing thread.

    Args:
        client: Instagram client
        logged_in_user: Currently logged in user
        thread_id: Thread ID
        text: Message text

    Returns:
        The sent message
    """
    if not logged_in_user:
        raise LoginRequired("Not logged in")

    result = client.direct_answer(thread_id=int(thread_id), text=text)
    return parse_message(result, logged_in_user.pk)


def send_message_to_user(
    client: Client,
    logged_in_user: Optional[User],
    username: str,
    text: str
) -> DirectMessage:
    """
    Send a message to a user by username.
    Creates a new thread if one doesn't exist.

    Args:
        client: Instagram client
        logged_in_user: Currently logged in user
        username: Target username
        text: Message text

    Returns:
        The sent message
    """
    if not logged_in_user:
        raise LoginRequired("Not logged in")

    # Get user ID from username
    user_id = client.user_id_from_username(username)

    # Send message
    result = client.direct_send(text=text, user_ids=[user_id])
    return parse_message(result, logged_in_user.pk)


def search_user(
    client: Client,
    logged_in_user: Optional[User],
    username: str
) -> Optional[User]:
    """
    Search for a user by exact username.

    Args:
        client: Instagram client
        logged_in_user: Currently logged in user
        username: Username to search for

    Returns:
        User info or None if not found
    """
    if not logged_in_user:
        raise LoginRequired("Not logged in")

    try:
        user_info = client.user_info_by_username(username)
        return parse_user(user_info)
    except Exception:
        return None
