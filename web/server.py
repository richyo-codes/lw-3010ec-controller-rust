#!/usr/bin/env python3
"""Static file server with proper MIME types for WebAssembly."""

import http.server
import socketserver
import sys
import os

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 8080

# Ensure .wasm files get the correct MIME type
MIME_TYPES = {**http.server.SimpleHTTPRequestHandler.extensions_map}
MIME_TYPES[""] = "application/octet-stream"
MIME_TYPES[".wasm"] = "application/wasm"

class Handler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=os.getcwd(), **kwargs)

    def guess_type(self, path):
        # Custom MIME mapping before the default lookup
        extra = {".wasm": "application/wasm", ".d.ts": "application/typescript"}
        # Strip query/hash for extension lookup
        clean = path.split("?")[0].split("#")[0]
        for ext, mime in extra.items():
            if clean.endswith(ext):
                return mime
        return super().guess_type(path)

    def end_headers(self):
        # Cache static assets, not the page
        if self.path.startswith("/pkg/"):
            self.send_header("Cache-Control", "public, max-age=31536000, immutable")
        super().end_headers()

    def translate_path(self, path):
        # Normalize path, strip leading /
        path = path.split("?")[0].split("#")[0]
        if path.startswith("/"):
            path = path[1:]
        return os.path.normpath(path)

    def log_message(self, format, *args):
        # Format like the stdlib default but without traceback spam
        print(f"[{self.log_date_time_string()}] {format % args}", flush=True)

    def log_error(self, format, *args):
        # Suppress noisy 404s (favicon, etc.)
        if "404" not in str(args):
            super().log_error(format, *args)

socketserver.TCPServer.allow_reuse_address = True
with socketserver.TCPServer(("", PORT), Handler) as httpd:
    print(f"Serving web/ on http://127.0.0.1:{PORT}")
    print(f"Files: {', '.join(f for f in os.listdir('.') if not f.startswith('.'))}")
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        pass
