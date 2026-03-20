# homomorphic-llm-proxy

A proxy gateway demonstrating FHE (Fully Homomorphic Encryption) API surface for LLM inference.

> **⚠️ Demo Only:** Current encryption uses XOR stub. Real FHE requires SEAL or OpenFHE (see `crypto.py`).

## Architecture

```
Client → [Encrypt] → Proxy → [Upstream LLM API] → [Re-encrypt] → Client
```

## Endpoints

- `POST /encrypt` — Encrypt plaintext payload
- `POST /decrypt` — Decrypt ciphertext response
- `POST /proxy` — Forward encrypted payload to upstream LLM, re-encrypt response
- `GET /health` — Health check

## Usage

```bash
pip install -r requirements.txt
uvicorn homomorphic_llm_proxy.app:app --reload
```

## TODO: Real FHE

Replace XOR stub with SEAL or OpenFHE for actual homomorphic operations:
- [Microsoft SEAL](https://github.com/microsoft/SEAL)
- [OpenFHE](https://github.com/openfheorg/openfhe-development)

## Tests

```bash
pytest tests/
```

## License

MIT
