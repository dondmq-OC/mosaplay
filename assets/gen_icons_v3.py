"""GridPlayer icon designs — Refined Holographic Blue variants."""
from PIL import Image, ImageDraw, ImageFilter, ImageMath
import math, os, random

SZ = 1024
OUT = "icons_v3"

def rounded_bg(draw, r, fill):
    draw.rounded_rectangle([60, 60, SZ - 60, SZ - 60], radius=SZ // r, fill=fill)

def play_tri(cx, cy, w, h):
    """Return triangle vertices for centered play button."""
    return [
        (cx - w * 0.38, cy - h / 2),
        (cx + w * 0.55, cy),
        (cx - w * 0.38, cy + h / 2),
    ]

def draw_tri(draw, cx, cy, w, h, fill):
    draw.polygon(play_tri(cx, cy, w, h), fill=fill)

def lerp_color(c1, c2, t):
    return tuple(int(a + (b - a) * t) for a, b in zip(c1, c2))

# ═══════════════════════════════════════════════════════════
# A1 — Crystalline Prism
# Play triangle built from faceted glass shards.
# Light refracts through each facet. Deep indigo bg.
# Subtle chromatic aberration at edges.
# ═══════════════════════════════════════════════════════════
def design_a1():
    img = Image.new('RGBA', (SZ, SZ), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Deep indigo background
    for y in range(SZ):
        t = y / SZ
        r = int(4 + 8 * t)
        g = int(6 + 12 * t)
        b = int(30 + 40 * t)
        draw.line([(0, y), (SZ, y)], fill=(r, g, b, 255))
    rounded_bg(draw, 8, (6, 8, 26, 240))

    cx, cy = SZ // 2, SZ // 2
    tw, th = SZ * 0.30, SZ * 0.36

    # Faceted glass layers — each slightly offset, different opacity/tint
    facets = [
        (-0.06, -0.04, 0.88, (60, 140, 255, 25)),   # deep blue facet
        (0.04,  -0.03, 0.92, (80, 180, 255, 40)),   # cyan facet
        (-0.03, 0.05,  0.90, (40, 100, 220, 30)),   # indigo facet
        (0.05,  0.02,  0.95, (100, 210, 255, 50)),  # bright facet
        (0,     0,     1.0,  (140, 225, 255, 230)), # main glass
    ]
    for ox, oy, scale, color in facets:
        w, h = tw * scale, th * scale
        draw_tri(draw, cx + ox * SZ, cy + oy * SZ, w, h, color)

    # Edge highlight — thin bright line on left edge of main triangle
    tris = play_tri(cx, cy, tw, th)
    draw.line([tris[0], tris[1]], fill=(200, 240, 255, 120), width=2)
    draw.line([tris[2], tris[0]], fill=(160, 220, 255, 60), width=1)

    # Subtle glow behind
    glow = Image.new('RGBA', (SZ, SZ), (0, 0, 0, 0))
    gdraw = ImageDraw.Draw(glow)
    for r in range(250, 50, -15):
        a = int(12 * (1 - r / 250))
        gdraw.ellipse([cx - r, cy - r, cx + r, cy + r], fill=(80, 180, 255, a))
    img = Image.alpha_composite(glow.filter(ImageFilter.GaussianBlur(25)), img)

    return img


# ═══════════════════════════════════════════════════════════
# A2 — Light Gate
# Play triangle as a portal/gate cut into a dark surface,
# revealing brilliant cyan light from beyond.
# Ghost copies behind = multiple portals.
# Floating light particles.
# ═══════════════════════════════════════════════════════════
def design_a2():
    img = Image.new('RGBA', (SZ, SZ), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Dark surface with subtle grid pattern
    rounded_bg(draw, 8, (4, 6, 18, 255))
    # Subtle horizontal grid lines
    for gy in range(100, SZ - 100, 40):
        draw.line([(80, gy), (SZ - 80, gy)], fill=(12, 20, 50, 20))

    cx, cy = SZ // 2, SZ // 2
    tw, th = SZ * 0.28, SZ * 0.34

    # Portal "frames" — multiple outline gates receding into the distance
    for i, (scale, alpha) in enumerate([(0.72, 15), (0.82, 25), (0.91, 45), (0.97, 80)]):
        w, h = tw * scale, th * scale
        tris = play_tri(cx, cy, w, h)
        color = (100 + i * 30, 180 + i * 15, 255, alpha)
        draw.line(tris + [tris[0]], fill=color, width=2)

    # The main gate — filled with bright light
    draw_tri(draw, cx, cy, tw, th, (60, 200, 255, 220))

    # Inner bright core
    inner_w, inner_h = tw * 0.5, th * 0.5
    draw_tri(draw, cx - tw * 0.04, cy, inner_w, inner_h, (180, 240, 255, 200))

    # Light rays emanating from behind
    rays = Image.new('RGBA', (SZ, SZ), (0, 0, 0, 0))
    rdraw = ImageDraw.Draw(rays)
    for angle in range(0, 360, 12):
        rad = math.radians(angle)
        ex = cx + math.cos(rad) * SZ
        ey = cy + math.sin(rad) * SZ
        rdraw.line([(cx, cy), (ex, ey)], fill=(80, 200, 255, 8), width=1)
    img = Image.alpha_composite(img, rays)

    # Floating light particles
    random.seed(42)
    for _ in range(40):
        px = random.randint(120, SZ - 120)
        py = random.randint(120, SZ - 120)
        ps = random.randint(1, 3)
        pa = random.randint(20, 80)
        draw.ellipse([px - ps, py - ps, px + ps, py + ps], fill=(180, 230, 255, pa))

    return img


# ═══════════════════════════════════════════════════════════
# A3 — Pulse Rings
# Concentric expanding ring/arc ghosts emanating from
# the play button. Sonar/radar aesthetic meets holographic.
# Multiple rings = multiple simultaneous signals (videos).
# Sharp bright center, fading rings outward.
# ═══════════════════════════════════════════════════════════
def design_a3():
    img = Image.new('RGBA', (SZ, SZ), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    rounded_bg(draw, 8, (6, 8, 24, 255))

    cx, cy = SZ // 2, SZ // 2
    tw, th = SZ * 0.26, SZ * 0.32

    # Expanding concentric arc rings
    for i in range(6):
        r = 180 + i * 90
        alpha = int(60 * (1 - i / 6))
        for angle_start in [0, 90, 180, 270]:
            # Draw 60-degree arcs at each quadrant
            pts = []
            for a in range(angle_start, angle_start + 60, 2):
                rad = math.radians(a)
                pts.append((cx + r * math.cos(rad), cy + r * math.sin(rad)))
            if len(pts) > 1:
                for j in range(len(pts) - 1):
                    draw.line([pts[j], pts[j + 1]], fill=(100, 200, 255, alpha), width=1)

    # Ghost triangles - offset radially
    ghosts = [
        (-0.18, -0.10, 0.82, (50, 130, 240, 25)),
        (0.16,  -0.08, 0.85, (60, 150, 250, 35)),
        (-0.10, 0.14,  0.88, (70, 170, 255, 50)),
        (0.12,  0.10,  0.92, (90, 190, 255, 80)),
        (0,     0,     1.0,  (120, 215, 255, 240)),
    ]
    for ox, oy, scale, color in ghosts:
        w, h = tw * scale, th * scale
        draw_tri(draw, cx + ox * SZ, cy + oy * SZ, w, h, color)

    # Bright center dot
    dot_r = 8
    draw.ellipse([cx + tw * 0.55 - dot_r, cy - dot_r,
                  cx + tw * 0.55 + dot_r, cy + dot_r],
                 fill=(220, 245, 255, 240))

    return img


# ═══════════════════════════════════════════════════════════
# A4 — Liquid Glass
# The play triangle rendered as a thick glass object
# with internal caustics/refraction patterns.
# Smooth gradients, soft glows, liquid-like appearance.
# Very polished, premium feel.
# ═══════════════════════════════════════════════════════════
def design_a4():
    img = Image.new('RGBA', (SZ, SZ), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    rounded_bg(draw, 8, (8, 10, 28, 255))

    cx, cy = SZ // 2, SZ // 2
    tw, th = SZ * 0.30, SZ * 0.36

    # Deep blurred shadows behind
    shadow_layers = [
        (-0.06, -0.06, 0.86, (30, 80, 200, 30)),
        (0.06,  0.04,  0.91, (40, 100, 230, 40)),
        (-0.04, 0.06,  0.94, (50, 130, 245, 60)),
    ]
    for ox, oy, scale, color in shadow_layers:
        w, h = tw * scale, th * scale
        draw_tri(draw, cx + ox * SZ, cy + oy * SZ, w, h, color)

    # Main body — gradient from dark blue at top to bright cyan at bottom
    # Simulated by drawing multiple horizontal slices
    main_tris = play_tri(cx, cy, tw, th)
    min_y = min(p[1] for p in main_tris)
    max_y = max(p[1] for p in main_tris)
    for y in range(int(min_y), int(max_y), 2):
        t = (y - min_y) / (max_y - min_y)
        # Clip triangle at this y-level
        color = lerp_color((30, 100, 220), (80, 210, 255), t)
        # Simple horizontal line clipping (approximate)
        draw.line([(int(cx - tw * 0.38), y), (int(cx + tw * 0.55), y)],
                  fill=color + (220,), width=2)

    # Glass edge highlight
    tris = play_tri(cx, cy, tw, th)
    draw.line([tris[0], tris[1]], fill=(180, 230, 255, 100), width=3)
    draw.line([tris[2], tris[0]], fill=(120, 200, 240, 50), width=2)
    draw.line([tris[1], tris[2]], fill=(60, 140, 220, 40), width=2)

    # Soft outer glow
    glow = Image.new('RGBA', (SZ, SZ), (0, 0, 0, 0))
    gdraw = ImageDraw.Draw(glow)
    for r in range(220, 60, -20):
        a = int(8 * (1 - r / 220))
        gdraw.ellipse([cx - r, cy - r, cx + r, cy + r], fill=(60, 160, 255, a))
    img = Image.alpha_composite(glow.filter(ImageFilter.GaussianBlur(18)), img)

    # Subtle corner reflections (like glass catching light)
    corner = 140
    for i in range(3):
        rx = cx - tw * 0.38 + i * 60
        ry = cy - th * 0.4 + i * 40
        draw.ellipse([rx - 3, ry - 3, rx + 3, ry + 3], fill=(200, 240, 255, 80 + i * 30))

    return img


# ═══════════════════════════════════════════════════════════
# Generate all
# ═══════════════════════════════════════════════════════════
if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)

    designs = [
        ("A1_CrystallinePrism", design_a1, "晶体棱镜 — 多面玻璃折射，色散边缘，靛蓝背景"),
        ("A2_LightGate", design_a2, "光之门 — 黑暗表面切口，青蓝光芒透出，浮游粒子"),
        ("A3_PulseRings", design_a3, "脉冲环 — 同心弧线扩散，声纳雷达感，多重信号"),
        ("A4_LiquidGlass", design_a4, "液态玻璃 — 渐变厚玻璃，焦散折射，柔软辉光"),
    ]

    for name, fn, desc in designs:
        img = fn()
        img.save(f"{OUT}/{name}.png")
        print(f"  {name}.png — {desc}")

    # Comparison sheet 2x2
    comp = Image.new('RGBA', (SZ * 2 + 30, SZ * 2 + 30), (20, 20, 28, 255))
    labels = ["A1", "A2", "A3", "A4"]
    for i, (name, fn, desc) in enumerate(designs):
        img = fn().resize((SZ // 2, SZ // 2), Image.LANCZOS)
        col, row = i % 2, i // 2
        x, y = 15 + col * (SZ // 2 + 5), 15 + row * (SZ // 2 + 5)
        comp.paste(img, (x, y), img)
        ImageDraw.Draw(comp).text((x + 15, y + 15), labels[i],
                                  fill=(255, 255, 255, 180))

    comp.save(f"{OUT}/_comparison.png")
    print(f"\nComparison: {OUT}/_comparison.png")
    print("Done — open assets/icons_v3/ in Finder")
