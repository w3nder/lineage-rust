#!/usr/bin/env python3
import argparse
import functools
import http.server
import os
from pathlib import Path
import shutil
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]


def build(check=False, profile="release"):
    generator = shutil.which("wasm-bindgen")
    if not generator:
        raise SystemExit("Instale wasm-bindgen-cli na versão de wasm-bindgen registrada no Cargo.lock.")
    subprocess.run(["rustup", "target", "add", "wasm32-unknown-unknown", "--toolchain", "1.95.0"], check=True)
    while subprocess.run(["pgrep", "-f", "cargo (test|build|check|clippy)"], stdout=subprocess.DEVNULL).returncode == 0:
        print("Aguardando o build atual terminar…", flush=True)
        time.sleep(5)
    environment = dict(os.environ, CARGO_INCREMENTAL="0")
    command = ["cargo", "+1.95.0", "build", "-j2", "--profile", profile, "-p", "killer-web", "--target", "wasm32-unknown-unknown"]
    if check:
        command.extend(["--features", "platform-check"])
    subprocess.run(command, cwd=ROOT, env=environment, check=True)
    out = ROOT / ("web/checkpkg" if check else "web/pkg")
    subprocess.run([generator, str(ROOT / f"target/wasm32-unknown-unknown/{profile}/killer_web.wasm"), "--target", "web", "--out-dir", str(out)], check=True)
    optimizer = shutil.which("wasm-opt")
    if profile == "dist" and optimizer:
        wasm = out / "killer_web_bg.wasm"
        subprocess.run([optimizer, "-O3", "--all-features", str(wasm), "-o", str(wasm)], check=True)


class Handler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header("Cache-Control", "no-cache")
        self.send_header("X-Content-Type-Options", "nosniff")
        super().end_headers()

    def list_directory(self, path):
        self.send_error(404)
        return None


def main():
    parser = argparse.ArgumentParser(description="Compila e serve o cliente web local")
    parser.add_argument("command", choices=["build", "serve", "run", "check"])
    parser.add_argument("--port", type=int, default=8080)
    parser.add_argument("--profile", default="release", choices=["release", "dist"])
    args = parser.parse_args()
    if args.command in ("build", "run", "check"):
        build(args.command == "check", args.profile)
    if args.command == "check":
        print("Abra /check.html no servidor local para verificar arquivos sintéticos no Worker.")
    if args.command in ("serve", "run"):
        handler = functools.partial(Handler, directory=str(ROOT / "web"))
        with http.server.ThreadingHTTPServer(("127.0.0.1", args.port), handler) as server:
            print(f"Killer web: http://127.0.0.1:{args.port}", flush=True)
            server.serve_forever()


if __name__ == "__main__":
    main()
