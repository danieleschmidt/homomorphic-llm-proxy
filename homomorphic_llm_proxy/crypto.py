"""
FHE Crypto Stub — XOR-based demonstration only.

TODO: Replace with real Fully Homomorphic Encryption using SEAL or OpenFHE.
Real FHE would allow computations on encrypted data without decryption.
This stub uses XOR with a key for demonstration of the API surface only.

References:
  - Microsoft SEAL: https://github.com/microsoft/SEAL
  - OpenFHE: https://github.com/openfheorg/openfhe-development
"""

import hashlib
from typing import Union

# Demo key — in real FHE this would be a cryptographic key pair
_DEMO_KEY = b"fhe_demo_key_2024"


def _get_key_bytes(length: int) -> bytes:
    """Generate repeating key bytes for XOR (demo only)."""
    key = _DEMO_KEY
    return (key * (length // len(key) + 1))[:length]


def encrypt(plaintext: Union[str, bytes]) -> bytes:
    """
    STUB: XOR-encrypt plaintext.
    NOT cryptographically secure. Demo API surface only.
    """
    if isinstance(plaintext, str):
        plaintext = plaintext.encode("utf-8")
    key_bytes = _get_key_bytes(len(plaintext))
    return bytes(a ^ b for a, b in zip(plaintext, key_bytes))


def decrypt(ciphertext: bytes) -> str:
    """
    STUB: XOR-decrypt ciphertext.
    NOT cryptographically secure. Demo API surface only.
    """
    key_bytes = _get_key_bytes(len(ciphertext))
    plaintext = bytes(a ^ b for a, b in zip(ciphertext, key_bytes))
    return plaintext.decode("utf-8")


def encrypt_b64(plaintext: str) -> str:
    """Encrypt and return base64-encoded ciphertext."""
    import base64
    return base64.b64encode(encrypt(plaintext)).decode("ascii")


def decrypt_b64(ciphertext_b64: str) -> str:
    """Decode base64 and decrypt."""
    import base64
    ciphertext = base64.b64decode(ciphertext_b64)
    return decrypt(ciphertext)
