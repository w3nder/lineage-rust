#!/usr/bin/env python3
import argparse
import hashlib
import re
import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FILES = ["index.html", "main.js", "worker.js", "prepare-worker.js", "audio.mjs", "folder.mjs", "hud-style.mjs",
         "loading.mjs", "pointer.mjs", "preparation-cache.mjs", "remote.mjs", "prefetch.mjs", "upgrade.mjs", "preload.json"]
PACKAGE = ["killer_web.js", "killer_web_bg.wasm"]


def main():
    parser = argparse.ArgumentParser(description="Monta dist/web com os arquivos de produção do cliente web")
    parser.add_argument("--client-url", required=True, help="base do bucket com manifest.json")
    parser.add_argument("--gateway", required=True, help="endereço wss:// do gateway, sem barra final")
    parser.add_argument("--editable-gateway", action="store_true")
    parser.add_argument("--background-mb", type=int, default=3000, help="limite do download em segundo plano")
    args = parser.parse_args()
    if not re.fullmatch(r"wss?://[^\s'\"\\]+", args.gateway) or not re.fullmatch(r"https?://[^\s'\"\\]+", args.client_url):
        raise SystemExit("endereços inválidos")
    web = ROOT / "web"
    out = ROOT / "dist" / "web"
    if out.exists():
        shutil.rmtree(out)
    (out / "pkg").mkdir(parents=True)
    for name in FILES:
        shutil.copy2(web / name, out / name)
    shutil.copytree(web / "art", out / "art")
    for name in PACKAGE:
        shutil.copy2(web / "pkg" / name, out / "pkg" / name)
    digest = hashlib.sha256()
    for name in sorted(FILES) + [f"pkg/{name}" for name in PACKAGE]:
        digest.update(name.encode())
        digest.update((out / name).read_bytes())
    build = digest.hexdigest()[:16]
    editable = "true" if args.editable_gateway else "false"
    (out / "config.mjs").write_text(
        f"export const config={{\n    clientUrl:'{args.client_url.rstrip('/')}',\n    gateway:'{args.gateway.rstrip('/')}',\n    gatewayEditable:{editable},\n    build:'{build}',\n    backgroundMB:{max(0, args.background_mb)},\n}};\n")
    (out / ".assetsignore").write_text("")
    print(f"{out} pronto")


if __name__ == "__main__":
    main()
