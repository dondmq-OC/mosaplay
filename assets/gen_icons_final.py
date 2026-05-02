"""Enhanced original ghosted play button — 3 variants."""
from PIL import Image, ImageDraw, ImageFilter
import math, os

SZ = 1024
OUT = "icons_final"

def rounded_bg(draw, r, fill):
    draw.rounded_rectangle([60, 60, SZ - 60, SZ - 60], radius=SZ // r, fill=fill)

def play_tri(cx, cy, w, h):
    return [
        (cx - w * 0.38, cy - h / 2),
        (cx + w * 0.55, cy),
        (cx - w * 0.38, cy + h / 2),
    ]

def draw_tri(draw, cx, cy, w, h, fill):
    draw.polygon(play_tri(cx, cy, w, h), fill=fill)

def draw_tri_outline(draw, cx, cy, w, h, fill, width):
    pts = play_tri(cx, cy, w, h)
    draw.line(pts + [pts[0]], fill=fill, width=width)

def blend_pix(bg, fg):
    fa = fg[3] / 255.0
    return tuple(int(f * fa + b * (1 - fa)) for f, b in zip(fg[:3], bg[:3])) + (min(255, bg[3] + fg[3]),)


# ═══════════════════════════════════════════════════════════
# E1 — "Deep Ghosts"
# 6 ghosts with wide offset range. Strong fading gradient.
# Bright orange main. Subtle radial glow behind.
# ═══════════════════════════════════════════════════════════
def design_e1():
    img = Image.new('RGBA', (SZ, SZ), (0, 0, 0, 0))
    px = img.load()

    # Dark bg
    for y in range(SZ):
        for x in range(SZ):
            dx, dy = (x - SZ/2) / (SZ/2), (y - SZ/2) / (SZ/2)
            d = math.sqrt(dx*dx + dy*dy)
            t = min(1.0, d * 1.2)
            r = int(14 + 6 * t); g = int(14 + 6 * t); b = int(20 + 8 * t)
            px[x, y] = (r, g, b, 255)

    ov = Image.new('RGBA', (SZ, SZ), (0,0,0,0))
    rounded_bg(ImageDraw.Draw(ov), 8, (16, 16, 24, 220))
    for y in range(SZ):
        for x in range(SZ):
            o = ov.getpixel((x,y))
            if o[3] > 0: px[x, y] = blend_pix(px[x,y], o)

    cx, cy = SZ // 2, SZ // 2
    tw, th = SZ * 0.30, SZ * 0.36

    # 6 ghosts — wide offset range, fading from nearly invisible → visible
    ghosts = [
        (-0.35, 0.06, 0.74, (50, 50, 60, 12)),     # far back ghost
        (-0.26, 0.04, 0.80, (60, 60, 70, 22)),
        (-0.18, 0.03, 0.86, (70, 70, 80, 40)),
        (-0.11, 0.02, 0.92, (90, 90, 100, 70)),
        (-0.05, 0.01, 0.97, (130, 130, 145, 120)),
        (0,     0,    1.0,  (255, 120, 0, 250)),    # main — bright orange
    ]

    # Background glow
    glow = Image.new('RGBA', (SZ, SZ), (0,0,0,0))
    gd = ImageDraw.Draw(glow)
    for r in range(280, 40, -15):
        a = int(14 * (1 - r / 280))
        gd.ellipse([cx - r, cy - r, cx + r, cy + r], fill=(255, 120, 0, a))
    glow = glow.filter(ImageFilter.GaussianBlur(20))
    for y in range(SZ):
        for x in range(SZ):
            g = glow.getpixel((x,y))
            if g[3] > 0: px[x, y] = blend_pix(px[x,y], g)

    # Draw ghosts + main
    for ox, oy, scale, color in ghosts:
        ov2 = Image.new('RGBA', (SZ, SZ), (0,0,0,0))
        draw_tri(ImageDraw.Draw(ov2), cx + ox * SZ, cy + oy * SZ, tw * scale, th * scale, color)
        for y in range(SZ):
            for x in range(SZ):
                p = ov2.getpixel((x,y))
                if p[3] > 0: px[x, y] = blend_pix(px[x,y], p)

    return img


# ═══════════════════════════════════════════════════════════
# E2 — "Edge Light"
# 5 ghosts with thin bright edge highlights (glassy).
# Sharper contrast, cleaner look with outline definition.
# ═══════════════════════════════════════════════════════════
def design_e2():
    img = Image.new('RGBA', (SZ, SZ), (0, 0, 0, 0))
    px = img.load()
    for y in range(SZ):
        for x in range(SZ):
            px[x, y] = (16, 16, 24, 255)
    ov = Image.new('RGBA', (SZ, SZ), (0,0,0,0))
    rounded_bg(ImageDraw.Draw(ov), 8, (18, 18, 27, 230))
    for y in range(SZ):
        for x in range(SZ):
            o = ov.getpixel((x,y))
            if o[3] > 0: px[x, y] = blend_pix(px[x,y], o)

    cx, cy = SZ // 2, SZ // 2
    tw, th = SZ * 0.30, SZ * 0.36

    # Ghosts with edge highlights
    ghosts = [
        (-0.28, 0.05, 0.76, (55, 55, 65, 18),  (90, 90, 105, 30)),
        (-0.19, 0.03, 0.84, (70, 70, 82, 38),  (120, 120, 140, 55)),
        (-0.11, 0.02, 0.91, (100, 100, 115, 75), (170, 170, 190, 90)),
        (-0.04, 0.01, 0.97, (160, 160, 175, 130), (220, 220, 235, 140)),
    ]

    for ox, oy, scale, fill_color, edge_color in ghosts:
        gcx, gcy = cx + ox * SZ, cy + oy * SZ
        gw, gh = tw * scale, th * scale
        ov2 = Image.new('RGBA', (SZ, SZ), (0,0,0,0))
        d2 = ImageDraw.Draw(ov2)
        draw_tri(d2, gcx, gcy, gw, gh, fill_color)
        draw_tri_outline(d2, gcx, gcy, gw, gh, edge_color, 2)
        for y in range(SZ):
            for x in range(SZ):
                p = ov2.getpixel((x,y))
                if p[3] > 0: px[x, y] = blend_pix(px[x,y], p)

    # Main triangle with highlight
    ov3 = Image.new('RGBA', (SZ, SZ), (0,0,0,0))
    d3 = ImageDraw.Draw(ov3)
    draw_tri(d3, cx, cy, tw, th, (255, 120, 0, 250))
    # Inner bright gradient slice
    inner_pts = [
        (cx - tw * 0.20, cy - th * 0.20),
        (cx + tw * 0.25, cy),
        (cx - tw * 0.20, cy + th * 0.20),
    ]
    d3.polygon(inner_pts, fill=(255, 160, 60, 140))
    # Edge highlight
    draw_tri_outline(d3, cx, cy, tw, th, (255, 200, 140, 180), 2)
    for y in range(SZ):
        for x in range(SZ):
            p = ov3.getpixel((x,y))
            if p[3] > 0: px[x, y] = blend_pix(px[x,y], p)

    return img


# ═══════════════════════════════════════════════════════════
# E3 — "Dramatic"
# 7 ghosts, deepest offset, strongest glow, radial bg.
# Maximum visual impact. Larger main triangle.
# ═══════════════════════════════════════════════════════════
def design_e3():
    img = Image.new('RGBA', (SZ, SZ), (0, 0, 0, 0))
    px = img.load()

    # Radial gradient bg
    for y in range(SZ):
        for x in range(SZ):
            dx, dy = (x - SZ/2) / (SZ/2), (y - SZ/2) / (SZ/2)
            d = math.sqrt(dx*dx + dy*dy)
            t = min(1.0, d)
            r = int(8 + 16 * t); g = int(8 + 16 * t); b = int(14 + 22 * t)
            px[x, y] = (r, g, b, 255)

    ov = Image.new('RGBA', (SZ, SZ), (0,0,0,0))
    rounded_bg(ImageDraw.Draw(ov), 8, (12, 12, 20, 240))
    for y in range(SZ):
        for x in range(SZ):
            o = ov.getpixel((x,y))
            if o[3] > 0: px[x, y] = blend_pix(px[x,y], o)

    cx, cy = SZ // 2, SZ // 2
    tw, th = SZ * 0.32, SZ * 0.38  # slightly larger

    # Strong background glow
    glow = Image.new('RGBA', (SZ, SZ), (0,0,0,0))
    gd = ImageDraw.Draw(glow)
    for r in range(320, 30, -12):
        a = int(20 * (1 - r / 320))
        gd.ellipse([cx - r, cy - r, cx + r, cy + r], fill=(255, 110, 20, a))
    glow = glow.filter(ImageFilter.GaussianBlur(22))
    for y in range(SZ):
        for x in range(SZ):
            g = glow.getpixel((x,y))
            if g[3] > 0: px[x, y] = blend_pix(px[x,y], g)

    # 7 ghosts
    ghosts = [
        (-0.40, 0.08, 0.68, (30, 30, 40, 8)),
        (-0.33, 0.06, 0.73, (40, 40, 50, 15)),
        (-0.26, 0.05, 0.79, (50, 50, 62, 28)),
        (-0.19, 0.03, 0.85, (68, 68, 80, 50)),
        (-0.12, 0.02, 0.91, (95, 95, 110, 85)),
        (-0.06, 0.01, 0.96, (145, 145, 160, 140)),
        (0,     0,    1.0,  (255, 115, 0, 255)),
    ]

    for ox, oy, scale, color in ghosts:
        ov2 = Image.new('RGBA', (SZ, SZ), (0,0,0,0))
        draw_tri(ImageDraw.Draw(ov2), cx + ox * SZ, cy + oy * SZ, tw * scale, th * scale, color)
        for y in range(SZ):
            for x in range(SZ):
                p = ov2.getpixel((x,y))
                if p[3] > 0: px[x, y] = blend_pix(px[x,y], p)

    # Bright edge on main
    ov3 = Image.new('RGBA', (SZ, SZ), (0,0,0,0))
    d3 = ImageDraw.Draw(ov3)
    draw_tri_outline(d3, cx, cy, tw, th, (255, 200, 130, 200), 3)
    for y in range(SZ):
        for x in range(SZ):
            p = ov3.getpixel((x,y))
            if p[3] > 0: px[x, y] = blend_pix(px[x,y], p)

    return img


# ═══════════════════════════════════════════════════════════
if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)

    designs = [
        ("E1_DeepGhosts", design_e1, "深度残影 — 6层重影，宽偏移范围，暖橙主三角+辉光"),
        ("E2_EdgeLight", design_e2, "边缘光 — 5层残影+玻璃高光边，更锐利干净"),
        ("E3_Dramatic", design_e3, "戏剧性 — 7层残影，强辉光，径向渐变背景，最大视觉冲击"),
    ]

    for name, fn, desc in designs:
        img = fn()
        img.save(f"{OUT}/{name}.png")
        print(f"  {name}.png — {desc}")

    # Comparison
    comp = Image.new('RGBA', (SZ * 3 + 30, SZ + 20), (20, 20, 26, 255))
    for i, (name, fn, desc) in enumerate(designs):
        img = fn().resize((SZ // 2, SZ // 2), Image.LANCZOS)
        x = 10 + i * (SZ // 2 + 5)
        y = 10
        comp.paste(img, (x, y), img)
        ImageDraw.Draw(comp).text((x + 15, y + 15), name.split('_')[0],
                                  fill=(255, 255, 255, 160))

    comp.save(f"{OUT}/_comparison.png")
    print(f"\nComparison: {OUT}/_comparison.png")
    print("Done — open assets/icons_final/ in Finder")
