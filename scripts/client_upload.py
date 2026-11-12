#!/usr/bin/env python3
import argparse
import hashlib
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path


def digest(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def build(root):
    files = []
    for path in sorted(p for p in root.rglob("*") if p.is_file() and not p.name.startswith(".")):
        files.append({"path": path.relative_to(root).as_posix(), "size": path.stat().st_size, "sha256": digest(path)})
    return {"files": files}


def aws(args, endpoint):
    return subprocess.run(["aws", "--endpoint-url", endpoint, *args], check=True)


def main():
    parser = argparse.ArgumentParser(description="Gera o manifesto do cliente e envia para um bucket R2.")
    parser.add_argument("--client", default=os.environ.get("KILLER_CLIENT_DIR"))
    parser.add_argument("--bucket", default=os.environ.get("R2_BUCKET"))
    parser.add_argument("--account", default=os.environ.get("R2_ACCOUNT_ID"))
    parser.add_argument("--prefix", default=os.environ.get("R2_PREFIX", ""))
    parser.add_argument("--manifest-only", action="store_true")
    args = parser.parse_args()
    if not (args.client and args.bucket and args.account):
        sys.exit("informe --client, --bucket e --account (ou KILLER_CLIENT_DIR, R2_BUCKET, R2_ACCOUNT_ID)")
    root = Path(args.client)
    endpoint = f"https://{args.account}.r2.cloudflarestorage.com"
    destination = f"s3://{args.bucket}/{args.prefix.strip('/')}".rstrip("/")
    manifest = build(root)
    total = sum(f["size"] for f in manifest["files"])
    print(f"{len(manifest['files'])} arquivos, {total / 2**30:.2f} GB")
    with tempfile.TemporaryDirectory() as tmp:
        target = Path(tmp) / "manifest.json"
        target.write_text(json.dumps(manifest, separators=(",", ":")))
        if args.manifest_only:
            print("manifesto gerado, nada enviado")
            return
        aws(["s3", "sync", str(root), destination, "--exclude", ".*", "--exclude", "*/.*", "--only-show-errors"], endpoint)
        aws(["s3", "cp", str(target), f"{destination}/manifest.json", "--content-type", "application/json", "--only-show-errors"], endpoint)
    print("envio concluído")


main()
