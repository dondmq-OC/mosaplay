"""Generate 3 icon design variants for GridPlayer."""
from PIL import Image, ImageDraw, ImageFilter, ImageChops
import math, os

SZ = 1024
OUT = "icons_v2"

def rounded_rect(draw, xy, r, fill):
    draw.rounded_rectangle(xy, radius=r, fill=fill)

def draw_play_triangle(draw, cx, cy, w, h, fill, outline=None, outline_w=0):
    """Draw a right-pointing play triangle centered at (cx, cy)."""
    pts = [
        (cx - w * 0.38, cy - h / 2),  # left-top
        (cx + w * 0.55, cy),           # right-center
        (cx - w * 0.38, cy + h / 2),  # left-bottom
    ]
    draw.polygon(pts, fill=fill)
    if outline and outline_w > 0:
        draw.line(pts + [pts[0]], fill=outline, width=outline_w)


# ═══════════════════════════════════════════════════════════
# DESIGN A: Holographic Depth
# Blue-to-cyan gradient play buttons with depth offset,
# subtle scan lines, glowing edges. Inspired by holographic UI.
# ═══════════════════════════════════════════════════════════
def design_a():
    img = Image.new('RGBA', (SZ, SZ), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Dark background with subtle radial gradient
    for i in range(SZ // 2, 0, -1):
        alpha = int(60 + 40 * (1 - i / (SZ // 2)))
        draw.ellipse([SZ//2 - i, SZ//2 - i, SZ//2 + i, SZ//2 + i],
                     fill=(8, 12, 24, alpha))
    rounded_rect(draw, [60, 60, SZ - 60, SZ - 60], SZ // 8, (10, 14, 28, 255))

    cx, cy = SZ // 2, SZ // 2
    tw, th = SZ * 0.30, SZ * 0.36

    # Ghost layers — blue to cyan, fading, offset in Z (scale down)
    ghosts = [
        (0.78, 0, 0.14, (20, 60, 220, 30)),   # deep blue, far back
        (0.85, 0, 0.08, (30, 100, 240, 50)),   # mid blue
        (0.92, 0, 0.04, (40, 160, 255, 80)),   # bright blue
        (0.97, 0, 0.01, (60, 200, 255, 140)),  # cyan edge
        (1.0,  0, 0,     (80, 220, 255, 240)), # main — bright cyan
    ]

    for scale, ox, oy, color in ghosts:
        w, h = tw * scale, th * scale
        draw_play_triangle(draw, cx + ox * SZ, cy + oy * SZ, w, h, color)

    # Scan line overlay on the main triangle
    scan = Image.new('RGBA', (SZ, SZ), (0, 0, 0, 0))
    sdraw = ImageDraw.Draw(scan)
    for sy in range(0, SZ, 6):
        sdraw.line([(0, sy), (SZ, sy)], fill=(255, 255, 255, 8))
    img = Image.alpha_composite(img, scan)

    return img


# ═══════════════════════════════════════════════════════════
# DESIGN B: Light Trails
# Bright orange primary triangle with motion trails fading
# backward. Suggests speed, motion, multiple simultaneous
# playback. Warm/energy feel.
# ═══════════════════════════════════════════════════════════
def design_b():
    img = Image.new('RGBA', (SZ, SZ), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    rounded_rect(draw, [60, 60, SZ - 60, SZ - 60], SZ // 8, (18, 16, 22, 255))

    cx, cy = SZ // 2, SZ // 2
    tw, th = SZ * 0.30, SZ * 0.36

    # Trail layers — offset LEFT (behind the play direction)
    trails = [
        (-0.32, 0,   0.88, (90, 45, 10, 15)),   # farthest
        (-0.22, 0,   0.92, (150, 70, 15, 35)),
        (-0.13, 0,   0.95, (210, 100, 20, 70)),
        (-0.06, 0,   0.98, (240, 140, 25, 130)),
        (0,     0,   1.0,  (255, 160, 30, 255)),  # main
    ]

    for ox, oy, scale, color in trails:
        w, h = tw * scale, th * scale
        draw_play_triangle(draw, cx + ox * SZ, cy + oy * SZ, w, h, color)

    # Subtle glow behind main triangle
    glow = Image.new('RGBA', (SZ, SZ), (0, 0, 0, 0))
    gdraw = ImageDraw.Draw(glow)
    for r in range(200, 80, -20):
        alpha = int(15 * (1 - r / 200))
        gdraw.ellipse([cx - r, cy - r, cx + r, cy + r],
                      fill=(255, 140, 40, alpha))
    glow = glow.filter(ImageFilter.GaussianBlur(30))
    img = Image.alpha_composite(img, glow)

    return img


# ═══════════════════════════════════════════════════════════
# DESIGN C: Spectrum Split
# One main white triangle that "splits" into RGB/CMY copies
# offset in different directions. Prism/spectrum concept.
# Clean, modern, colorful.
# ═══════════════════════════════════════════════════════════
def design_c():
    img = Image.new('RGBA', (SZ, SZ), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    rounded_rect(draw, [60, 60, SZ - 60, SZ - 60], SZ // 8, (14, 14, 20, 255))

    cx, cy = SZ // 2, SZ // 2
    tw, th = SZ * 0.28, SZ * 0.34

    # Split copies in different directions with different hues
    splits = [
        (-0.12, -0.08, 0.94, (255, 60, 60, 90)),    # red, up-left
        (0.12,  -0.06, 0.94, (60, 200, 60, 90)),    # green, up-right
        (-0.08, 0.10,  0.94, (60, 100, 255, 90)),   # blue, down-left
        (0.10,  0.08,  0.94, (220, 180, 40, 90)),   # yellow, down-right
        (0,     0,     0.98, (180, 130, 255, 160)),  # purple center
        (0,     0,     1.0,  (240, 240, 255, 255)),  # white main
    ]

    for ox, oy, scale, color in splits:
        w, h = tw * scale, th * scale
        draw_play_triangle(draw, cx + ox * SZ, cy + oy * SZ, w, h, color)

    return img


# ═══════════════════════════════════════════════════════════
# DESIGN D: Minimalist Arc
# Single bold play triangle with 3 thin outline arcs behind,
# suggesting playback waves / concentric motion.
# Ultra-clean, Apple-style.
# ═══════════════════════════════════════════════════════════
def design_d():
    img = Image.new('RGBA', (SZ, SZ), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    rounded_rect(draw, [60, 60, SZ - 60, SZ - 60], SZ // 8, (12, 13, 20, 255))

    cx, cy = SZ // 2, SZ // 2
    tw, th = SZ * 0.28, SZ * 0.34

    # Thin outline ghost arcs behind the play button
    for i, (scale, alpha) in enumerate([(0.82, 30), (0.90, 55), (0.96, 90)]):
        w, h = tw * scale, th * scale
        pts = [
            (cx - w * 0.38, cy - h / 2),
            (cx + w * 0.55, cy),
            (cx - w * 0.38, cy + h / 2),
        ]
        draw.line(pts + [pts[0]], fill=(180, 200, 255, alpha), width=3)

    # Main filled triangle with subtle gradient
    draw_play_triangle(draw, cx, cy, tw, th, (220, 240, 255, 250))

    # Small accent dot at the "play" point
    dot_r = 6
    draw.ellipse([cx + tw * 0.55 - dot_r, cy - dot_r,
                  cx + tw * 0.55 + dot_r, cy + dot_r],
                 fill=(255, 255, 255, 200))

    return img


# ═══════════════════════════════════════════════════════════
# Generate all designs + comparison sheet
# ═══════════════════════════════════════════════════════════
if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)

    designs = [
        ("A_HolographicDepth", design_a, "全息深度蓝 — 索尼风格蓝青渐变，扫描线，深度层次"),
        ("B_LightTrails", design_b, "光轨残影 — 橙色能量三角，运动轨迹，速度感"),
        ("C_SpectrumSplit", design_c, "棱镜分光 — 白色主三角，RGB/CMY 四向分裂"),
        ("D_MinimalistArc", design_d, "极简弧线 — Apple 风格，细线轮廓弧，纯净白"),
    ]

    for name, fn, desc in designs:
        img = fn()
        img.save(f"{OUT}/{name}.png")
        print(f"  {name}.png — {desc}")

    # Generate comparison sheet (2x2 grid)
    comp = Image.new('RGBA', (SZ * 2 + 40, SZ * 2 + 40), (30, 30, 35, 255))
    positions = [(0, 0), (1, 0), (0, 1), (1, 1)]
    labels = ["A", "B", "C", "D"]

    for (name, fn, desc), (col, row), label in zip(designs, positions, labels):
        img = fn().resize((SZ // 2, SZ // 2), Image.LANCZOS)
        x = 20 + col * (SZ // 2 + 10)
        y = 20 + row * (SZ // 2 + 10)
        comp.paste(img, (x, y), img)
        # Label
        ldraw = ImageDraw.Draw(comp)
        ldraw.text((x + 12, y + 12), label, fill=(255, 255, 255, 200))

    comp.save(f"{OUT}/_comparison.png")
    print(f"\nComparison sheet: {OUT}/_comparison.png")
    print("Done.")
