"""Sony holographic intro style icon designs — volumetric light, god rays, sparkles."""
from PIL import Image, ImageDraw, ImageFilter
import math, os, random

SZ = 1024
OUT = "icons_sony"

def rounded_bg(draw, r, fill):
    draw.rounded_rectangle([60, 60, SZ - 60, SZ - 60], radius=SZ // r, fill=fill)

def play_tri_pts(cx, cy, w, h):
    return [
        (cx - w * 0.38, cy - h / 2),
        (cx + w * 0.55, cy),
        (cx - w * 0.38, cy + h / 2),
    ]

def lerp(a, b, t):
    return tuple(int(ai + (bi - ai) * t) for ai, bi in zip(a, b))

def blend_alpha(fg, bg):
    """Blend fg RGBA onto bg RGBA, return RGBA."""
    fa = fg[3] / 255.0
    return tuple(int(f * fa + b * (1 - fa)) for f, b in zip(fg[:3], bg[:3])) + (min(255, fg[3] + bg[3]),)


def design_sony(bg_dark, bg_light, glow_color, beam_color, core_color, spark_color, name):
    """
    Sony holographic style icon.

    bg_dark/light: background gradient from top-left to bottom-right
    glow_color: outer glow around triangle
    beam_color: god rays/light beams
    core_color: main triangle fill
    spark_color: floating sparkle particles
    """
    img = Image.new('RGBA', (SZ, SZ), (0, 0, 0, 0))
    pixels = img.load()

    cx, cy = SZ // 2, SZ // 2
    tw, th = SZ * 0.30, SZ * 0.36
    tris = play_tri_pts(cx, cy, tw, th)

    # ── 1. Background with radial gradient ──────────────
    for y in range(SZ):
        for x in range(SZ):
            # Distance from center (normalized 0-1)
            dx, dy = (x - cx) / (SZ / 2), (y - cy) / (SZ / 2)
            dist = math.sqrt(dx * dx + dy * dy)
            # Radial falloff
            t = min(1.0, dist * 1.3)
            color = lerp(bg_dark, bg_light, 0.15 + 0.15 * (1 - t))
            pixels[x, y] = color + (255,)

    # ── 2. Rounded rectangle overlay ────────────────────
    overlay = Image.new('RGBA', (SZ, SZ), (0, 0, 0, 0))
    odraw = ImageDraw.Draw(overlay)
    odraw.rounded_rectangle([60, 60, SZ - 60, SZ - 60],
                            radius=SZ // 8, fill=(bg_light[0] // 2, bg_light[1] // 2, bg_light[2] // 2, 200))
    for y in range(SZ):
        for x in range(SZ):
            op = overlay.getpixel((x, y))
            if op[3] > 0:
                bg = pixels[x, y]
                pixels[x, y] = blend_alpha(op, bg)

    # ── 3. God rays (volumetric light beams) ────────────
    rays = Image.new('RGBA', (SZ, SZ), (0, 0, 0, 0))
    rdraw = ImageDraw.Draw(rays)

    # Rays originate from behind the triangle
    ray_origin = (cx + tw * 0.05, cy)

    # Emit rays in a fan pattern
    for angle in range(-60, 61, 3):
        rad = math.radians(angle + 180)  # pointing right
        # Ray endpoint
        ex = ray_origin[0] + math.cos(rad) * SZ * 2
        ey = ray_origin[1] + math.sin(rad) * SZ * 2
        # Vary alpha based on angle (stronger center)
        dist_from_center = abs(angle) / 60.0
        alpha = int(30 * (1 - dist_from_center))
        if alpha > 0:
            rdraw.line([ray_origin, (ex, ey)], fill=beam_color + (alpha,), width=2)

        # Secondary rays (wider, softer)
        alpha2 = int(10 * (1 - dist_from_center))
        if alpha2 > 0:
            ex2 = ray_origin[0] + math.cos(rad + 0.02) * SZ * 2
            ey2 = ray_origin[1] + math.sin(rad + 0.02) * SZ * 2
            rdraw.line([ray_origin, (ex2, ey2)], fill=beam_color + (alpha2,), width=4)

    rays = rays.filter(ImageFilter.GaussianBlur(4))
    for y in range(SZ):
        for x in range(SZ):
            rp = rays.getpixel((x, y))
            if rp[3] > 0:
                pixels[x, y] = blend_alpha(rp, pixels[x, y])

    # ── 4. Outer halo/glow ──────────────────────────────
    halo = Image.new('RGBA', (SZ, SZ), (0, 0, 0, 0))
    hdraw = ImageDraw.Draw(halo)
    for r in range(300, 30, -10):
        a = int(18 * (1 - r / 300))
        hdraw.ellipse([cx - r, cy - r, cx + r, cy + r], fill=glow_color + (a,))
    halo = halo.filter(ImageFilter.GaussianBlur(20))
    for y in range(SZ):
        for x in range(SZ):
            hp = halo.getpixel((x, y))
            if hp[3] > 0:
                pixels[x, y] = blend_alpha(hp, pixels[x, y])

    # ── 5. Ghost triangles behind ───────────────────────
    ghosts = [
        (-0.16, 0,    0.80, tuple(int(c * 0.15) for c in core_color) + (20,)),
        (-0.10, 0,    0.88, tuple(int(c * 0.30) for c in core_color) + (40,)),
        (-0.05, 0,    0.94, tuple(int(c * 0.55) for c in core_color) + (80,)),
    ]
    for ox, oy, scale, color in ghosts:
        gcx = cx + ox * SZ
        gcy = cy + oy * SZ
        gw, gh = tw * scale, th * scale
        gtri = play_tri_pts(gcx, gcy, gw, gh)
        # Draw on overlay then blend
        gov = Image.new('RGBA', (SZ, SZ), (0, 0, 0, 0))
        gdv = ImageDraw.Draw(gov)
        gdv.polygon(gtri, fill=color)
        gov = gov.filter(ImageFilter.GaussianBlur(2))
        for y in range(SZ):
            for x in range(SZ):
                gp = gov.getpixel((x, y))
                if gp[3] > 0:
                    pixels[x, y] = blend_alpha(gp, pixels[x, y])

    # ── 6. Main triangle — luminous core ─────────────────
    # Draw with bright fill + slightly brighter edge
    mtri_ov = Image.new('RGBA', (SZ, SZ), (0, 0, 0, 0))
    mdraw = ImageDraw.Draw(mtri_ov)
    mdraw.polygon(tris, fill=core_color + (235,))

    # Inner bright highlight (gradient within triangle)
    highlight_pts = [
        (cx - tw * 0.25, cy - th * 0.25),
        (cx + tw * 0.30, cy),
        (cx - tw * 0.25, cy + th * 0.25),
    ]
    bright_core = tuple(min(255, int(c * 1.3)) for c in core_color)
    mdraw.polygon(highlight_pts, fill=bright_core + (150,))

    # Edge glow
    mdraw.line(tris + [tris[0]], fill=bright_core + (200,), width=3)

    mtri_ov = mtri_ov.filter(ImageFilter.GaussianBlur(1.5))
    for y in range(SZ):
        for x in range(SZ):
            mp = mtri_ov.getpixel((x, y))
            if mp[3] > 0:
                pixels[x, y] = blend_alpha(mp, pixels[x, y])

    # ── 7. Sparkle particles ─────────────────────────────
    random.seed(12345)
    for _ in range(60):
        px = random.randint(80, SZ - 80)
        py = random.randint(80, SZ - 80)
        ps = random.randint(1, 3)
        pa = random.randint(15, 100)
        # Closer to triangle = brighter
        dist_to_tri = min(
            math.sqrt((px - p[0])**2 + (py - p[1])**2) for p in tris
        )
        if dist_to_tri < 200:
            pa = min(180, pa + 60)
        for dx in range(-ps, ps + 1):
            for dy in range(-ps, ps + 1):
                nx, ny = px + dx, py + dy
                if 0 <= nx < SZ and 0 <= ny < SZ:
                    d = math.sqrt(dx*dx + dy*dy)
                    if d <= ps:
                        sa = int(pa * (1 - d / max(1, ps)))
                        sp = spark_color + (sa,)
                        pixels[nx, ny] = blend_alpha(sp, pixels[nx, ny])

    # ── 8. Lens flare (subtle, top-right) ────────────────
    flare_x, flare_y = cx + tw * 0.8, cy - th * 0.5
    flare = Image.new('RGBA', (SZ, SZ), (0, 0, 0, 0))
    fdraw = ImageDraw.Draw(flare)
    for r in range(40, 2, -3):
        a = int(40 * (1 - r / 40))
        fdraw.ellipse([flare_x - r, flare_y - r, flare_x + r, flare_y + r],
                      fill=spark_color + (a,))
    flare = flare.filter(ImageFilter.GaussianBlur(3))
    for y in range(SZ):
        for x in range(SZ):
            fp = flare.getpixel((x, y))
            if fp[3] > 0:
                pixels[x, y] = blend_alpha(fp, pixels[x, y])

    img.save(f"{OUT}/{name}.png")
    print(f"  {name}.png")


# ═══════════════════════════════════════════════════════════
# Color modes
# ═══════════════════════════════════════════════════════════
if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)

    modes = [
        # name,     bg_dark,         bg_light,       glow,           beam,           core,           spark
        ("B1_SonyGold",
         (2, 4, 24), (6, 10, 50), (200, 140, 60), (180, 130, 50), (255, 200, 100), (255, 230, 180),
         "索尼金 — 经典蓝黑底+琥珀金光，原版索尼全息感"),

        ("B2_CyanBlue",
         (2, 4, 22), (4, 10, 48), (60, 180, 240), (50, 160, 230), (100, 220, 255), (200, 240, 255),
         "青蓝 — 深海暗底+电光青蓝，科技冷峻感"),

        ("B3_PurpleNova",
         (8, 2, 22), (16, 4, 44), (180, 60, 220), (160, 50, 200), (220, 120, 255), (240, 200, 255),
         "紫爆 — 深紫暗底+粉紫光，神秘高级感"),

        ("B4_EmeraldMatrix",
         (2, 10, 8),  (4, 22, 16), (40, 200, 120), (30, 180, 100), (80, 240, 150), (180, 255, 210),
         "翡翠 — 墨绿暗底+荧光翠绿，Matrix 代码感"),

        ("B5_RoseQuartz",
         (10, 4, 14), (22, 8, 30), (220, 80, 120), (200, 60, 100), (255, 140, 170), (255, 200, 215),
         "蔷薇石英 — 暗玫红底+柔粉光，柔和高级感"),
    ]

    for name, bd, bl, gl, bm, co, sp, desc in modes:
        design_sony(bd, bl, gl, bm, co, sp, name)
        print(f"    {desc}")

    # Comparison 3+2 layout
    comp = Image.new('RGBA', (SZ * 2 + 30, SZ * 3 + 40), (18, 18, 24, 255))
    for i in range(5):
        col, row = i % 2, i // 2
        if i < 4:
            name = modes[i][0]
        else:
            # 5th goes centered in the 3rd row
            col, row = 0, 2
            name = modes[4][0]
        img = Image.open(f"{OUT}/{name}.png").resize((SZ // 2, SZ // 2), Image.LANCZOS)
        x = 15 + col * (SZ // 2 + 5)
        y = 15 + row * (SZ // 2 + 5)
        comp.paste(img, (x, y), img)
        ImageDraw.Draw(comp).text((x + 15, y + 15),
                                  name.split('_')[0], fill=(255, 255, 255, 170))

    comp.save(f"{OUT}/_comparison.png")
    print(f"\nComparison: {OUT}/_comparison.png")
    print("Done — open assets/icons_sony/ in Finder")
