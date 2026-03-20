"""
Homomorphic LLM Proxy — FastAPI application.

Provides an encrypted proxy between clients and OpenAI-compatible APIs.
Uses XOR stub encryption (see crypto.py for TODO on real FHE).
"""

import httpx
import os
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from typing import Any, Dict, Optional

from .crypto import encrypt_b64, decrypt_b64

app = FastAPI(
    title="Homomorphic LLM Proxy",
    description="FHE-stub proxy for LLM inference. See crypto.py for real FHE TODO.",
    version="0.1.0",
)

UPSTREAM_URL = os.environ.get("UPSTREAM_URL", "https://api.openai.com")


class EncryptRequest(BaseModel):
    plaintext: str


class EncryptResponse(BaseModel):
    ciphertext: str
    algorithm: str = "xor-stub-demo"
    warning: str = "NOT real FHE. Demo only. See TODO in crypto.py."


class DecryptRequest(BaseModel):
    ciphertext: str


class DecryptResponse(BaseModel):
    plaintext: str


class ProxyRequest(BaseModel):
    encrypted_payload: str
    endpoint: str = "/v1/chat/completions"
    extra_headers: Optional[Dict[str, str]] = None


class ProxyResponse(BaseModel):
    encrypted_response: str
    status_code: int
    upstream_url: str


@app.get("/health")
def health():
    return {"status": "ok", "fhe_mode": "xor-stub-demo"}


@app.post("/encrypt", response_model=EncryptResponse)
def encrypt_endpoint(req: EncryptRequest):
    """Encrypt plaintext using XOR stub (demo FHE surface)."""
    ciphertext = encrypt_b64(req.plaintext)
    return EncryptResponse(ciphertext=ciphertext)


@app.post("/decrypt", response_model=DecryptResponse)
def decrypt_endpoint(req: DecryptRequest):
    """Decrypt ciphertext using XOR stub."""
    try:
        plaintext = decrypt_b64(req.ciphertext)
    except Exception as e:
        raise HTTPException(status_code=400, detail=f"Decryption failed: {e}")
    return DecryptResponse(plaintext=plaintext)


@app.post("/proxy", response_model=ProxyResponse)
async def proxy_endpoint(req: ProxyRequest):
    """
    Decrypt payload, forward to upstream LLM API, re-encrypt response.
    In real FHE, computation would happen on encrypted data.
    TODO: Replace with SEAL/OpenFHE homomorphic operations.
    """
    # Decrypt the payload
    try:
        plaintext_payload = decrypt_b64(req.encrypted_payload)
    except Exception as e:
        raise HTTPException(status_code=400, detail=f"Failed to decrypt payload: {e}")

    upstream = f"{UPSTREAM_URL.rstrip('/')}{req.endpoint}"
    headers = {"Content-Type": "application/json"}
    if req.extra_headers:
        headers.update(req.extra_headers)

    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            response = await client.post(upstream, content=plaintext_payload, headers=headers)
        response_text = response.text
        status_code = response.status_code
    except httpx.ConnectError:
        # Upstream not reachable — return stub response for testing
        response_text = '{"stub": "upstream not reachable in test mode"}'
        status_code = 200

    encrypted_response = encrypt_b64(response_text)
    return ProxyResponse(
        encrypted_response=encrypted_response,
        status_code=status_code,
        upstream_url=upstream,
    )
