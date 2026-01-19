"""Authentication handling for Instagram client"""

import logging
from typing import Optional

from instagrapi import Client
from instagrapi.exceptions import (
    LoginRequired,
    ChallengeRequired,
    TwoFactorRequired,
    BadPassword,
    PleaseWaitFewMinutes,
)

from models import User
from .session import save_session, load_session
from .parsers import parse_user

logger = logging.getLogger(__name__)


def login(
    client: Client,
    username: str,
    password: str
) -> tuple[bool, Optional[str], Optional[User]]:
    """
    Login to Instagram. Returns (success, error_message, user).

    Tries to restore session first, then fresh login if needed.
    """
    # Try to restore existing session
    if load_session(client):
        try:
            # Verify session is still valid
            client.get_timeline_feed()
            user_info = client.account_info()
            user = parse_user(user_info)
            logger.info("Restored session for %s", user_info.username)
            return True, None, user
        except LoginRequired:
            logger.info("Saved session expired, logging in fresh...")
        except Exception as e:
            logger.warning("Session validation failed: %s", e)

    # Fresh login
    try:
        logger.info("Logging in as %s...", username)
        client.login(username, password)

        user_info = client.account_info()
        user = parse_user(user_info)

        # Save session for future use
        save_session(client)

        logger.info("Logged in as %s", user_info.username)
        return True, None, user

    except BadPassword:
        return False, "Invalid password", None
    except ChallengeRequired:
        return False, "Challenge required - please verify your account in the Instagram app", None
    except TwoFactorRequired:
        return False, "Two-factor authentication required - not yet supported", None
    except PleaseWaitFewMinutes:
        return False, "Rate limited - please wait a few minutes and try again", None
    except Exception as e:
        logger.error("Login failed: %s", e)
        return False, str(e), None
