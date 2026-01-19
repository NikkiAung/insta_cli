"""Session management for Instagram client"""

import logging
from pathlib import Path

from instagrapi import Client

logger = logging.getLogger(__name__)

SESSION_FILE = Path(__file__).parent.parent / ".ig_session.json"


def save_session(client: Client) -> None:
    """Save session to disk for reuse"""
    try:
        client.dump_settings(SESSION_FILE)
        logger.info("Session saved to %s", SESSION_FILE)
    except Exception as e:
        logger.warning("Failed to save session: %s", e)


def load_session(client: Client) -> bool:
    """Load session from disk. Returns True if successful."""
    if not SESSION_FILE.exists():
        logger.info("No saved session found")
        return False

    try:
        client.load_settings(SESSION_FILE)
        logger.info("Session loaded from %s", SESSION_FILE)
        return True
    except Exception as e:
        logger.warning("Failed to load session: %s", e)
        return False


def delete_session() -> None:
    """Delete saved session file"""
    try:
        if SESSION_FILE.exists():
            SESSION_FILE.unlink()
            logger.info("Session file deleted")
    except Exception as e:
        logger.warning("Failed to delete session file: %s", e)
