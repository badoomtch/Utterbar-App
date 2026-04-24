"""ORLIQ ASR sidecar — fallback provider.

Communicates with the Rust host over stdio using length-prefixed little-endian
u32 frames wrapping JSON payloads.

Requires (user must install):
    pip install nemo_toolkit[asr] soundfile numpy

This is intentionally the fallback path; the primary path is ONNX Runtime
inside the Rust process. The protocol is stable so the same script can host
other ASR backends without changing the Rust side.
"""

import base64
import json
import struct
import sys
from typing import Any, Dict, Optional


def _read_msg() -> Optional[Dict[str, Any]]:
    header = sys.stdin.buffer.read(4)
    if len(header) < 4:
        return None
    (length,) = struct.unpack("<I", header)
    payload = sys.stdin.buffer.read(length)
    if len(payload) < length:
        return None
    return json.loads(payload.decode("utf-8"))


def _write_msg(msg: Dict[str, Any]) -> None:
    data = json.dumps(msg).encode("utf-8")
    sys.stdout.buffer.write(struct.pack("<I", len(data)))
    sys.stdout.buffer.write(data)
    sys.stdout.buffer.flush()


_models: Dict[str, Any] = {}


def _load(model_id: str):
    if model_id in _models:
        return _models[model_id]
    # Lazy import so startup is fast and optional dep failures surface clearly.
    import nemo.collections.asr as nemo_asr  # type: ignore

    # NeMo's "pretrained_name" maps ORLIQ model IDs to official NVIDIA checkpoints.
    pretrained = {
        "parakeet-en-v1": "nvidia/parakeet-tdt-1.1b",
        "parakeet-multi-v1": "nvidia/parakeet-tdt_ctc-110m",
    }.get(model_id, model_id)
    model = nemo_asr.models.ASRModel.from_pretrained(pretrained)
    model.eval()
    _models[model_id] = model
    return model


def _handle(req: Dict[str, Any]) -> Dict[str, Any]:
    op = req.get("op")
    if op == "warm":
        _load(req["model_id"])
        return {"ok": True}
    if op == "transcribe":
        import numpy as np
        import soundfile as sf
        import io

        model = _load(req["model_id"])
        pcm_bytes = base64.b64decode(req["pcm_b64"])
        pcm = np.frombuffer(pcm_bytes, dtype=np.int16).astype(np.float32) / 32768.0
        sr = int(req.get("sample_rate", 16000))

        buf = io.BytesIO()
        sf.write(buf, pcm, sr, format="WAV", subtype="PCM_16")
        buf.seek(0)

        # NeMo's high-level transcribe accepts audio paths; we wrap in a temp file.
        import tempfile
        import os

        with tempfile.NamedTemporaryFile(suffix=".wav", delete=False) as tmp:
            tmp.write(buf.getvalue())
            tmp_path = tmp.name
        try:
            hypotheses = model.transcribe([tmp_path])
            text = hypotheses[0] if hypotheses else ""
            if isinstance(text, list) and text:
                text = text[0]
        finally:
            try:
                os.unlink(tmp_path)
            except OSError:
                pass

        duration_ms = int(len(pcm) * 1000 / sr)
        return {"ok": True, "text": str(text), "duration_ms": duration_ms}
    if op == "shutdown":
        return {"ok": True}
    return {"ok": False, "error": f"unknown op: {op}"}


def main() -> None:
    while True:
        msg = _read_msg()
        if msg is None:
            break
        try:
            resp = _handle(msg)
        except Exception as exc:  # noqa: BLE001
            resp = {"ok": False, "error": f"{type(exc).__name__}: {exc}"}
        _write_msg(resp)
        if msg.get("op") == "shutdown":
            break


if __name__ == "__main__":
    main()
