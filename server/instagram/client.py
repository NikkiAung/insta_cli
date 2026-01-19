"""Main Instagram client class"""

import logging
from typing import Optional

from instagrapi import Client

from models import (
    User,
    DirectMessage,
    DirectThread,
    DirectThreadPreview,
)
from .session import delete_session
from .auth import login as auth_login
from .messages import (
    get_inbox as msg_get_inbox,
    get_thread as msg_get_thread,
    send_message as msg_send_message,
    send_message_to_user as msg_send_message_to_user,
    search_user as msg_search_user,
)

logger = logging.getLogger(__name__)


class InstagramClient:
    """
    Wrapper around instagrapi.Client with session persistence
    and clean response formatting.
    """

    def __init__(self):
        self.client = Client()
        self.client.delay_range = [1, 3]  # Add delay between requests
        self._logged_in_user: Optional[User] = None

    # ========================================================================
    # Authentication
    # ========================================================================

    def login(self, username: str, password: str) -> tuple[bool, Optional[str]]:
        """
        Login to Instagram. Returns (success, error_message).

        Tries to restore session first, then fresh login if needed.
        """
        success, error, user = auth_login(self.client, username, password)
        if success:
            self._logged_in_user = user
        return success, error

    def logout(self) -> None:
        """Logout and clear session"""
        delete_session()
        self.client = Client()
        self._logged_in_user = None
        logger.info("Logged out")

    def is_authenticated(self) -> bool:
        """Check if currently logged in"""
        return self._logged_in_user is not None

    def get_current_user(self) -> Optional[User]:
        """Get the currently logged in user"""
        return self._logged_in_user

    # ========================================================================
    # Direct Messages
    # ========================================================================

    def get_inbox(self, amount: int = 20) -> list[DirectThreadPreview]:
        """Get DM inbox (list of threads)."""
        return msg_get_inbox(self.client, self._logged_in_user, amount)

    def get_thread(self, thread_id: str, amount: int = 20) -> DirectThread:
        """Get a thread with its messages."""
        return msg_get_thread(self.client, self._logged_in_user, thread_id, amount)

    def send_message(self, thread_id: str, text: str) -> DirectMessage:
        """Send a message to an existing thread."""
        return msg_send_message(self.client, self._logged_in_user, thread_id, text)

    def send_message_to_user(self, username: str, text: str) -> DirectMessage:
        """Send a message to a user by username."""
        return msg_send_message_to_user(self.client, self._logged_in_user, username, text)

    def search_user(self, username: str) -> Optional[User]:
        """Search for a user by exact username."""
        return msg_search_user(self.client, self._logged_in_user, username)
