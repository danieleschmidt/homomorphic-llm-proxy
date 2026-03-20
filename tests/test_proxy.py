import pytest
import base64
from fastapi.testclient import TestClient
from homomorphic_llm_proxy.app import app
from homomorphic_llm_proxy.crypto import encrypt, decrypt, encrypt_b64, decrypt_b64

client = TestClient(app)


# --- Crypto unit tests ---

def test_encrypt_returns_bytes():
    result = encrypt("hello")
    assert isinstance(result, bytes)
    assert len(result) == len("hello")

def test_decrypt_roundtrip():
    original = "Hello, FHE world!"
    assert decrypt(encrypt(original)) == original

def test_encrypt_b64_returns_string():
    result = encrypt_b64("test")
    assert isinstance(result, str)

def test_decrypt_b64_roundtrip():
    original = "test message 123"
    assert decrypt_b64(encrypt_b64(original)) == original

def test_encrypt_different_from_plaintext():
    plaintext = "secret"
    ciphertext = encrypt(plaintext)
    assert ciphertext != plaintext.encode()

def test_encrypt_empty_string():
    assert decrypt(encrypt("")) == ""

def test_encrypt_unicode():
    original = "Hello 世界 🌍"
    assert decrypt(encrypt(original)) == original


# --- API endpoint tests ---

def test_health_endpoint():
    resp = client.get("/health")
    assert resp.status_code == 200
    data = resp.json()
    assert data["status"] == "ok"
    assert "fhe_mode" in data

def test_encrypt_endpoint():
    resp = client.post("/encrypt", json={"plaintext": "hello world"})
    assert resp.status_code == 200
    data = resp.json()
    assert "ciphertext" in data
    assert data["algorithm"] == "xor-stub-demo"
    assert "warning" in data

def test_decrypt_endpoint():
    # First encrypt
    enc_resp = client.post("/encrypt", json={"plaintext": "my secret text"})
    ciphertext = enc_resp.json()["ciphertext"]
    # Then decrypt
    dec_resp = client.post("/decrypt", json={"ciphertext": ciphertext})
    assert dec_resp.status_code == 200
    assert dec_resp.json()["plaintext"] == "my secret text"

def test_encrypt_decrypt_roundtrip_via_api():
    original = "The quick brown fox"
    enc = client.post("/encrypt", json={"plaintext": original})
    ct = enc.json()["ciphertext"]
    dec = client.post("/decrypt", json={"ciphertext": ct})
    assert dec.json()["plaintext"] == original

def test_decrypt_invalid_base64():
    resp = client.post("/decrypt", json={"ciphertext": "not-valid-base64!!!"})
    assert resp.status_code == 400

def test_proxy_endpoint_stub():
    # Encrypt a payload and send through proxy
    payload = '{"model": "gpt-4", "messages": [{"role": "user", "content": "hi"}]}'
    enc_resp = client.post("/encrypt", json={"plaintext": payload})
    encrypted = enc_resp.json()["ciphertext"]

    proxy_resp = client.post("/proxy", json={
        "encrypted_payload": encrypted,
        "endpoint": "/v1/chat/completions"
    })
    assert proxy_resp.status_code == 200
    data = proxy_resp.json()
    assert "encrypted_response" in data
    assert "status_code" in data
    assert "upstream_url" in data
