#!/usr/bin/env python3
"""Discard visual captures whose change vs HEAD is below the noise threshold.

Two CI runs minutes apart can differ by a handful of low-amplitude pixels
(antialiasing jitter, measured: 45 px at delta <= 32/255 on 1440x900) even
with a frozen clock and an anchored seed. This script restores the committed
baseline for any capture whose *significant* change (max channel delta > 32)
stays under NOISE_THRESHOLD px, so the visual-preview job only commits real
content changes. New files (pages without a baseline yet) are always kept.

Run from the repo root, after the capture step, before the commit step.
Requires: Pillow (pip install pillow), git.
"""

import subprocess
import sys
from io import BytesIO

from PIL import Image, ImageChops

NOISE_THRESHOLD_PX = 64
CHANNEL_DELTA = 32
VISUAL_DIR = "frontend/e2e/visual"


def significant_px(baseline_bytes: bytes, captured_bytes: bytes) -> int:
    a = Image.open(BytesIO(baseline_bytes)).convert("RGB")
    b = Image.open(BytesIO(captured_bytes)).convert("RGB")
    if a.size != b.size:
        return sys.maxsize  # layout change — never noise
    diff = ImageChops.difference(a, b)
    data = diff.tobytes()
    ch = 3
    return sum(
        1
        for i in range(0, len(data), ch)
        if data[i] > CHANNEL_DELTA or data[i + 1] > CHANNEL_DELTA or data[i + 2] > CHANNEL_DELTA
    )


def main() -> int:
    status = subprocess.run(
        ["git", "status", "--porcelain", "--", VISUAL_DIR],
        capture_output=True,
        text=True,
        check=True,
    )
    changed = [
        line[3:].strip()
        for line in status.stdout.splitlines()
        if line.strip() and not line.startswith("??")  # new files always kept
    ]
    discarded = kept = 0
    for path in changed:
        head = subprocess.run(
            ["git", "show", f"HEAD:{path}"],
            capture_output=True,
        )
        if head.returncode != 0:
            kept += 1  # no baseline at HEAD — new page, keep
            continue
        with open(path, "rb") as f:
            captured = f.read()
        n = significant_px(head.stdout, captured)
        if n < NOISE_THRESHOLD_PX:
            subprocess.run(["git", "checkout", "--", path], check=True)
            discarded += 1
            print(f"noise ({n} px <= {NOISE_THRESHOLD_PX}): {path} restored")
        else:
            kept += 1
            print(f"real change ({n} px): {path} kept")
    print(f"{discarded} discarded, {kept} kept")
    return 0


if __name__ == "__main__":
    sys.exit(main())
