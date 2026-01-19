"""User-related Pydantic models"""

from typing import Optional
from pydantic import BaseModel, Field


class User(BaseModel):
    """Simplified user model"""
    pk: str = Field(description="User's primary key (ID)")
    username: str
    full_name: str = ""
    profile_pic_url: Optional[str] = None
    is_private: bool = False
    is_verified: bool = False


class UserShort(BaseModel):
    """Minimal user info for thread listings"""
    pk: str
    username: str
    full_name: str = ""
    profile_pic_url: Optional[str] = None
