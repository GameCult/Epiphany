from __future__ import annotations

import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import sys
import tomllib
from typing import Any
from urllib import error
from urllib import request
from uuid import uuid4


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_CONFIG = ROOT / "state" / "face-discord.toml"
DEFAULT_ARTIFACT_DIR = ROOT / ".epiphany-face"
DISCORD_API = "https://discord.com/api/v10"
CHAT_SCHEMA_VERSION = "epiphany.face_chat.v0"
BUBBLE_SCHEMA_VERSION = "epiphany.face_bubble.v0"


def now_stamp() -> str:
    return datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")


def read_text_arg(value: str | None, *, stdin_fallback: bool = True) -> str:
    if value:
        path = Path(value)
        if path.exists():
            return path.read_text(encoding="utf-8")
        return value
    if stdin_fallback:
        return sys.stdin.read()
    return ""


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def load_config(path: Path) -> dict[str, Any]:
    if path.suffix.lower() == ".toml":
        with path.open("rb") as handle:
            data = tomllib.load(handle)
    else:
        data = load_json(path)
    if not isinstance(data, dict):
        raise ValueError(f"Face config must be an object: {path}")
    return data


def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=False) + "\n", encoding="utf-8")


def latest_face_artifacts(artifact_dir: Path, *, limit: int = 8) -> list[dict[str, Any]]:
    if not artifact_dir.exists():
        return []
    items: list[dict[str, Any]] = []
    for path in sorted(artifact_dir.glob("face-*.json"), key=lambda item: item.stat().st_mtime, reverse=True):
        try:
            payload = load_json(path)
        except (OSError, json.JSONDecodeError):
            continue
        if not isinstance(payload, dict):
            continue
        items.append(
            {
                "path": str(path),
                "name": path.name,
                "modifiedAt": datetime.fromtimestamp(
                    path.stat().st_mtime, timezone.utc
                ).replace(microsecond=0).isoformat(),
                "schemaVersion": payload.get("schema_version"),
                "status": payload.get("status"),
                "reason": payload.get("reason"),
                "content": payload.get("content"),
                "bubble": payload.get("bubble"),
                "source": payload.get("source"),
            }
        )
        if len(items) >= limit:
            break
    return items


def allowed_channel_id(config: dict[str, Any]) -> str | None:
    explicit = config.get("allowed_channel_id")
    if isinstance(explicit, str) and explicit.strip():
        return explicit.strip()
    env_name = config.get("allowed_channel_id_env")
    if isinstance(env_name, str) and env_name:
        env_value = os.environ.get(env_name)
        if env_value:
            return env_value.strip()
    return None


def bot_token(config: dict[str, Any]) -> str | None:
    env_name = config.get("bot_token_env")
    if not isinstance(env_name, str) or not env_name:
        return None
    value = os.environ.get(env_name)
    return value.strip() if value else None


def draft_payload(content: str, *, config: dict[str, Any], status: str, reason: str) -> dict[str, Any]:
    return {
        "schema_version": CHAT_SCHEMA_VERSION,
        "created_at": datetime.now(timezone.utc).replace(microsecond=0).isoformat(),
        "status": status,
        "reason": reason,
        "allowed_channel_name": config.get("allowed_channel_name", "#aquarium"),
        "allowed_channel_id": allowed_channel_id(config),
        "content": content.strip(),
    }


def write_draft(content: str, *, config: dict[str, Any], artifact_dir: Path, status: str, reason: str) -> Path:
    payload = draft_payload(content, config=config, status=status, reason=reason)
    path = artifact_dir / f"face-chat-{now_stamp()}-{uuid4().hex[:8]}.json"
    write_json(path, payload)
    return path


def bubble_payload(
    content: str,
    *,
    source: str,
    status: str = "ready",
    mood: str = "attentive",
    target: str = "aquarium",
) -> dict[str, Any]:
    return {
        "schema_version": BUBBLE_SCHEMA_VERSION,
        "created_at": datetime.now(timezone.utc).replace(microsecond=0).isoformat(),
        "status": status,
        "source": source,
        "target": target,
        "role_id": "face",
        "agent_id": "face",
        "display_name": "Face",
        "mood": mood,
        "content": content.strip(),
        "bubble": {
            "kind": "agent-chat",
            "anchorRoleId": "face",
            "opensIn": "aquarium",
            "requiresDiscord": False,
            "ttlSeconds": 90,
        },
    }


def write_bubble(
    content: str,
    *,
    artifact_dir: Path,
    source: str,
    status: str = "ready",
    mood: str = "attentive",
) -> Path:
    payload = bubble_payload(content, source=source, status=status, mood=mood)
    path = artifact_dir / f"face-bubble-{now_stamp()}-{uuid4().hex[:8]}.json"
    write_json(path, payload)
    return path


def post_message(content: str, *, channel_id: str, token: str) -> dict[str, Any]:
    body = json.dumps({"content": content}).encode("utf-8")
    req = request.Request(
        f"{DISCORD_API}/channels/{channel_id}/messages",
        data=body,
        headers={
            "Authorization": f"Bot {token}",
            "Content-Type": "application/json",
            "User-Agent": "EpiphanyFace/0.1",
        },
        method="POST",
    )
    try:
        with request.urlopen(req, timeout=20) as response:
            return json.loads(response.read().decode("utf-8"))
    except error.HTTPError as exc:
        detail = exc.read().decode("utf-8", errors="replace")
        raise RuntimeError(f"Discord post failed with HTTP {exc.code}: {detail}") from exc


def run_draft(args: argparse.Namespace) -> dict[str, Any]:
    config = load_config(args.config)
    content = read_text_arg(args.content)
    if not content.strip():
        raise ValueError("Face chat content is empty")
    path = write_draft(
        content,
        config=config,
        artifact_dir=args.artifact_dir,
        status="draft",
        reason="drafted without posting",
    )
    return {"ok": True, "posted": False, "draftPath": str(path)}


def run_bubble(args: argparse.Namespace) -> dict[str, Any]:
    content = read_text_arg(args.content)
    if not content.strip():
        raise ValueError("Face bubble content is empty")
    path = write_bubble(
        content,
        artifact_dir=args.artifact_dir,
        source=args.source,
        status=args.status,
        mood=args.mood,
    )
    return {"ok": True, "posted": False, "bubblePath": str(path), "bubble": load_json(path)}


def run_post(args: argparse.Namespace) -> dict[str, Any]:
    config = load_config(args.config)
    content = read_text_arg(args.content)
    if not content.strip():
        raise ValueError("Face chat content is empty")
    configured_channel_id = allowed_channel_id(config)
    requested_channel_id = args.channel_id or configured_channel_id
    if not configured_channel_id:
        path = write_draft(
            content,
            config=config,
            artifact_dir=args.artifact_dir,
            status="blocked",
            reason="missing #aquarium channel id",
        )
        return {"ok": False, "posted": False, "blocked": "missing-channel-id", "draftPath": str(path)}
    if requested_channel_id != configured_channel_id:
        path = write_draft(
            content,
            config=config,
            artifact_dir=args.artifact_dir,
            status="blocked",
            reason="requested channel does not match configured #aquarium channel id",
        )
        return {"ok": False, "posted": False, "blocked": "wrong-channel", "draftPath": str(path)}
    token = bot_token(config)
    if not token:
        path = write_draft(
            content,
            config=config,
            artifact_dir=args.artifact_dir,
            status="blocked",
            reason="missing Discord bot token",
        )
        return {"ok": False, "posted": False, "blocked": "missing-token", "draftPath": str(path)}
    response = post_message(content.strip(), channel_id=configured_channel_id, token=token)
    path = write_draft(
        content,
        config=config,
        artifact_dir=args.artifact_dir,
        status="posted",
        reason=f"posted message {response.get('id')}",
    )
    return {"ok": True, "posted": True, "messageId": response.get("id"), "draftPath": str(path)}


def run_smoke(args: argparse.Namespace) -> dict[str, Any]:
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)
        config = {
            "schema_version": "epiphany.face_discord.v0",
            "allowed_channel_name": "#aquarium",
            "allowed_channel_id": None,
            "allowed_channel_id_env": "EPIPHANY_FACE_AQUARIUM_CHANNEL_ID_TEST",
            "bot_token_env": "DISCORD_BOT_TOKEN_TEST",
            "policy": ["Face may post only in #aquarium."],
        }
        config_path = tmp_dir / "face-discord.json"
        write_json(config_path, config)
        draft = run_draft(
            argparse.Namespace(
                config=config_path,
                artifact_dir=tmp_dir,
                content="Face notices Body and Soul disagree about evidence shape.",
            )
        )
        bubble = run_bubble(
            argparse.Namespace(
                artifact_dir=tmp_dir,
                content="Face opens an Aquarium bubble even while Discord is unavailable.",
                source="smoke/face",
                status="ready",
                mood="attentive",
            )
        )
        blocked = run_post(
            argparse.Namespace(
                config=config_path,
                artifact_dir=tmp_dir,
                content="Face should not post without a configured #aquarium channel id.",
                channel_id=None,
            )
        )
        os.environ["EPIPHANY_FACE_AQUARIUM_CHANNEL_ID_TEST"] = "123"
        wrong = run_post(
            argparse.Namespace(
                config=config_path,
                artifact_dir=tmp_dir,
                content="Face should not post outside #aquarium.",
                channel_id="456",
            )
        )
        ok = (
            draft["ok"]
            and bubble["ok"]
            and bubble["bubble"]["schema_version"] == BUBBLE_SCHEMA_VERSION
            and bubble["bubble"]["bubble"]["requiresDiscord"] is False
            and not blocked["ok"]
            and blocked["blocked"] == "missing-channel-id"
            and not wrong["ok"]
            and wrong["blocked"] == "wrong-channel"
        )
        return {"ok": ok, "draft": draft, "bubble": bubble, "blocked": blocked, "wrongChannel": wrong}


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Draft or post Epiphany Face chat with #aquarium-only guardrails.")
    subparsers = parser.add_subparsers(dest="command", required=True)
    for command in ("draft", "post"):
        sub = subparsers.add_parser(command)
        sub.add_argument("--config", type=Path, default=DEFAULT_CONFIG)
        sub.add_argument("--artifact-dir", type=Path, default=DEFAULT_ARTIFACT_DIR)
        sub.add_argument("--content", help="Content string or path. Reads stdin when omitted.")
        if command == "post":
            sub.add_argument("--channel-id", help="Must match configured #aquarium channel id if supplied.")
    bubble = subparsers.add_parser("bubble")
    bubble.add_argument("--artifact-dir", type=Path, default=DEFAULT_ARTIFACT_DIR)
    bubble.add_argument("--content", help="Content string or path. Reads stdin when omitted.")
    bubble.add_argument("--source", default="epiphany/face")
    bubble.add_argument("--status", default="ready")
    bubble.add_argument("--mood", default="attentive")
    smoke = subparsers.add_parser("smoke")
    smoke.add_argument("--config", type=Path, default=DEFAULT_CONFIG)
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        if args.command == "draft":
            result = run_draft(args)
        elif args.command == "bubble":
            result = run_bubble(args)
        elif args.command == "post":
            result = run_post(args)
        elif args.command == "smoke":
            result = run_smoke(args)
        else:
            raise AssertionError(args.command)
        print(json.dumps(result, indent=2))
        return 0 if result.get("ok") else 1
    except Exception as exc:
        print(json.dumps({"ok": False, "error": str(exc)}, indent=2), file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
