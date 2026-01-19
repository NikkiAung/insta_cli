"""
RSA encryption utilities for secure credential transmission.

The CLI encrypts passwords with the server's public key before sending.
The server decrypts using its private key.
"""

import base64
import logging
from pathlib import Path

from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import padding, rsa

logger = logging.getLogger(__name__)

# Store keys in a hidden directory
KEYS_DIR = Path(__file__).parent / ".keys"
PRIVATE_KEY_PATH = KEYS_DIR / "private.pem"
PUBLIC_KEY_PATH = KEYS_DIR / "public.pem"


def ensure_keys_exist() -> None:
    """Generate RSA key pair if not already exists."""
    if PRIVATE_KEY_PATH.exists() and PUBLIC_KEY_PATH.exists():
        logger.debug("RSA keys already exist")
        return

    logger.info("Generating new RSA key pair...")
    KEYS_DIR.mkdir(exist_ok=True)

    # Generate private key
    private_key = rsa.generate_private_key(
        public_exponent=65537,
        key_size=2048,
    )

    # Save private key
    PRIVATE_KEY_PATH.write_bytes(
        private_key.private_bytes(
            encoding=serialization.Encoding.PEM,
            format=serialization.PrivateFormat.PKCS8,
            encryption_algorithm=serialization.NoEncryption(),
        )
    )

    # Save public key
    PUBLIC_KEY_PATH.write_bytes(
        private_key.public_key().public_bytes(
            encoding=serialization.Encoding.PEM,
            format=serialization.PublicFormat.SubjectPublicKeyInfo,
        )
    )

    logger.info("RSA key pair generated and saved to %s", KEYS_DIR)


def get_public_key_pem() -> str:
    """Get the public key as a PEM string for the CLI to use."""
    ensure_keys_exist()
    return PUBLIC_KEY_PATH.read_text()


def decrypt_password(encrypted_base64: str) -> str:
    """
    Decrypt a password that was encrypted by the CLI.

    Args:
        encrypted_base64: Base64-encoded encrypted password

    Returns:
        The decrypted password string

    Raises:
        ValueError: If decryption fails
    """
    ensure_keys_exist()

    try:
        # Load private key
        private_key = serialization.load_pem_private_key(
            PRIVATE_KEY_PATH.read_bytes(),
            password=None,
        )

        # Decode from base64
        encrypted_data = base64.b64decode(encrypted_base64)

        # Decrypt using OAEP padding (more secure than PKCS1v15)
        decrypted = private_key.decrypt(
            encrypted_data,
            padding.OAEP(
                mgf=padding.MGF1(algorithm=hashes.SHA256()),
                algorithm=hashes.SHA256(),
                label=None,
            ),
        )

        return decrypted.decode("utf-8")

    except Exception as e:
        logger.error("Failed to decrypt password: %s", e)
        raise ValueError("Failed to decrypt password") from e
