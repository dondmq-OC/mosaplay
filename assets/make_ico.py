import struct, io, os
from PIL import Image

def make_ico():
    sizes = [16, 32, 48, 64, 128, 256]
    images = []
    for s in sizes:
        img = Image.open(f'icon_{s}.png')
        if img.mode != 'RGBA':
            img = img.convert('RGBA')
        images.append((s, img))

    count = len(images)
    header = struct.pack('<HHH', 0, 1, count)
    dir_size = 6 + 16 * count
    entries = b''
    image_data = b''
    offset = dir_size

    for w, img in images:
        h = w
        buf = io.BytesIO()
        img.save(buf, format='PNG')
        png = buf.getvalue()
        size = len(png)
        entries += struct.pack('<BBBBHHII',
            w if w < 256 else 0,
            h if h < 256 else 0,
            0, 0, 1, 32, size, offset
        )
        image_data += png
        offset += size

    with open('GridPlayer.ico', 'wb') as f:
        f.write(header + entries + image_data)

    print(f"ICO: {os.path.getsize('GridPlayer.ico')} bytes, {count} sizes")

if __name__ == '__main__':
    make_ico()
