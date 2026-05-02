from PIL import Image, ImageDraw, ImageFilter
import math, os

SZ = 1024
OUT = "icons_final"

def blend_pix(bg, fg):
    fa = fg[3] / 255.0
    return tuple(int(f * fa + b * (1 - fa)) for f, b in zip(fg[:3], bg[:3])) + (min(255, bg[3] + fg[3]),)

def play_tri(cx, cy, w, h):
    return [(cx - w * 0.38, cy - h / 2), (cx + w * 0.55, cy), (cx - w * 0.38, cy + h / 2)]

def draw_tri(draw, cx, cy, w, h, fill):
    draw.polygon(play_tri(cx, cy, w, h), fill=fill)

GHOSTS_6 = [
    (-0.44, 0.09, 0.56, (28, 28, 42,  15)),
    (-0.35, 0.07, 0.64, (48, 48, 65,  48)),
    (-0.25, 0.05, 0.74, (72, 72, 92,  98)),
    (-0.15, 0.03, 0.85, (118, 118, 138, 160)),
    (-0.06, 0.01, 0.94, (188, 188, 202, 225)),
    (0,     0,    1.0,  (255, 82, 0,  255)),
]
CORNER_R = 110

def generate(style, label):
    img = Image.new('RGBA', (SZ, SZ), (0, 0, 0, 0))
    px = img.load()

    for y in range(SZ):
        for x in range(SZ):
            dx, dy = (x - SZ/2) / (SZ/2), (y - SZ/2) / (SZ/2)
            d = math.sqrt(dx*dx + dy*dy)
            t = min(1.0, d)
            r = int(5 + 20 * t); g = int(5 + 20 * t); b = int(10 + 26 * t)
            px[x, y] = (r, g, b, 255)

    ov = Image.new('RGBA', (SZ, SZ), (0,0,0,0))
    ImageDraw.Draw(ov).rounded_rectangle([60, 60, SZ - 60, SZ - 60], radius=SZ // 8, fill=(8, 8, 16, 248))
    for y in range(SZ):
        for x in range(SZ):
            o = ov.getpixel((x, y))
            if o[3] > 0:
                px[x, y] = blend_pix(px[x, y], o)

    # Frosted BG: blur + reduce opacity
    if style in ("frosted_bg", "frosted_full"):
        blurred = img.filter(ImageFilter.GaussianBlur(12))
        mask = Image.new('L', (SZ - 120, SZ - 120), 0)
        ImageDraw.Draw(mask).rounded_rectangle([0, 0, SZ - 121, SZ - 121], radius=CORNER_R, fill=255)
        for y in range(SZ):
            for x in range(SZ):
                mx, my = x - 60, y - 60
                if 0 <= mx < SZ - 120 and 0 <= my < SZ - 120 and mask.getpixel((mx, my)) > 128:
                    orig = px[x, y]
                    blur = blurred.getpixel((x, y))
                    px[x, y] = tuple(int(o * 0.3 + b * 0.7) for o, b in zip(orig[:3], blur[:3])) + (orig[3],)

    # Frosted Border: glass edge rings + top reflection
    if style in ("frosted_border", "frosted_full"):
        border_img = Image.new('RGBA', (SZ, SZ), (0, 0, 0, 0))
        bd = ImageDraw.Draw(border_img)
        bd.rounded_rectangle([58, 58, SZ - 58, SZ - 58], radius=CORNER_R, outline=(255, 255, 255, 35), width=3)
        bd.rounded_rectangle([55, 55, SZ - 55, SZ - 55], radius=CORNER_R + 5, outline=(255, 255, 255, 12), width=1)
        bd.arc([65, 65, SZ // 2 + 100, SZ // 3], 180, 280, fill=(255, 255, 255, 20), width=20)
        border_img = border_img.filter(ImageFilter.GaussianBlur(1))
        for y in range(SZ):
            for x in range(SZ):
                b = border_img.getpixel((x, y))
                if b[3] > 0:
                    px[x, y] = blend_pix(px[x, y], b)

    x_shift = 90
    cx = SZ // 2 + x_shift
    cy = SZ // 2
    tw, th = SZ * 0.44, SZ * 0.50

    glow = Image.new('RGBA', (SZ, SZ), (0, 0, 0, 0))
    gd = ImageDraw.Draw(glow)
    for r in range(440, 15, -8):
        a = int(40 * (1 - r / 440))
        gd.ellipse([SZ // 2 - r, SZ // 2 - r, SZ // 2 + r, SZ // 2 + r], fill=(255, 85, 0, a))
    glow = glow.filter(ImageFilter.GaussianBlur(24))
    for y in range(SZ):
        for x in range(SZ):
            g = glow.getpixel((x, y))
            if g[3] > 0:
                px[x, y] = blend_pix(px[x, y], g)

    for ox, oy, scale, color in GHOSTS_6:
        gcx, gcy = cx + ox * SZ, cy + oy * SZ
        gw, gh = tw * scale, th * scale
        ov2 = Image.new('RGBA', (SZ, SZ), (0, 0, 0, 0))
        draw_tri(ImageDraw.Draw(ov2), gcx, gcy, gw, gh, color)
        for y in range(SZ):
            for x in range(SZ):
                p = ov2.getpixel((x, y))
                if p[3] > 0:
                    px[x, y] = blend_pix(px[x, y], p)

    name = f"E3_Final_{label}"
    img.save(f"{OUT}/{name}.png")
    return name


os.makedirs(OUT, exist_ok=True)
styles = [("none", "Original"), ("frosted_bg", "FrostedBG"),
          ("frosted_border", "FrostedBorder"), ("frosted_full", "FrostedBoth")]
generated = []
for style, label in styles:
    name = generate(style, label)
    generated.append((name, label))
    print(f"  {name}.png")

comp = Image.new('RGBA', (SZ + 20, SZ + 20), (14, 14, 20, 255))
d = ImageDraw.Draw(comp)
for i, (name, label) in enumerate(generated):
    img = Image.open(f"{OUT}/{name}.png").resize((SZ // 2, SZ // 2), Image.LANCZOS)
    col, row = i % 2, i // 2
    x, y = 10 + col * (SZ // 2 + 5), 10 + row * (SZ // 2 + 5)
    comp.paste(img, (x, y), img)
    d.text((x + 15, y + 15), label, fill=(255, 255, 255, 170))
comp.save(f"{OUT}/_comparison_frosted.png")
print("\n_comparison_frosted.png")
