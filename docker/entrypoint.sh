#!/bin/sh
# ── entrypoint.sh — La Ruche ─────────────────────────────────────
# Starts the Rust backend in the background,
# then replaces the shell process with Caddy (PID 1).
# Docker sends SIGTERM to Caddy which stops gracefully;
# the backend (child process) receives SIGKILL after the stop_grace delay.

set -e

echo "[laruche] Starting backend..."
/app/laruche &
BACKEND_PID=$!

# Attends que le backend soit prêt (max 10s) avant de démarrer Caddy
# Évite les 502 immédiats et rend les crashs démarrage visibles
WAIT=0
until kill -0 "$BACKEND_PID" 2>/dev/null && wget -qO- http://localhost:8080/notice > /dev/null 2>&1; do
    if ! kill -0 "$BACKEND_PID" 2>/dev/null; then
        echo "[laruche] FATAL: backend process has crashed, check logs above."
        exit 1
    fi
    WAIT=$((WAIT + 1))
    if [ "$WAIT" -ge 20 ]; then
        echo "[laruche] WARNING: backend did not become ready in 10s, starting Caddy anyway."
        break
    fi
    sleep 0.5
done
echo "[laruche] Backend ready."

echo "[laruche] Starting Caddy..."
exec caddy run --config /etc/caddy/Caddyfile --adapter caddyfile
