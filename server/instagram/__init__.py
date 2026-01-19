"""
Instagram client wrapper using instagrapi

Handles authentication, session persistence, and provides clean methods
for DM operations.
"""

from .client import InstagramClient

# Singleton instance
instagram_client = InstagramClient()

__all__ = ["InstagramClient", "instagram_client"]
