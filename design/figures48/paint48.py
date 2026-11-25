# A small pixel painter for 32x32 sprites: shapes onto a grid of palette letters,
# lit from the upper left. Output: {name: {"p": palette, "r": [32 rows]}}.
import json, math, sys
N = 48

class Spr:
    def __init__(self, pal):
        self.p = dict(pal); self.g = [['.'] * N for _ in range(N)]
    def px(self, x, y, c):
        x = int(round(x)); y = int(round(y))
        if 0 <= x < N and 0 <= y < N and c: self.g[y][x] = c
    def get(self, x, y):
        return self.g[y][x] if 0 <= x < N and 0 <= y < N else '.'
    def rect(self, x, y, w, h, c):
        for j in range(int(h)):
            for i in range(int(w)): self.px(x + i, y + j, c)
    def line(self, x0, y0, x1, y1, c):
        n = int(max(abs(x1 - x0), abs(y1 - y0))) or 1
        for i in range(n + 1): self.px(x0 + (x1 - x0) * i / n, y0 + (y1 - y0) * i / n, c)
    def ell(self, cx, cy, rx, ry, ramp, lx=-0.6, ly=-0.7):
        """A shaded blob. ramp: light, mid, dark (a string of 3 letters, or 1)."""
        for y in range(N):
            for x in range(N):
                dx = (x + 0.5 - cx) / rx; dy = (y + 0.5 - cy) / ry
                d = dx * dx + dy * dy
                if d <= 1:
                    if len(ramp) == 1: self.px(x, y, ramp); continue
                    lit = -(dx * lx + dy * ly)          # >0 away from the light
                    edge = d
                    v = lit * 0.9 + edge * 0.35
                    self.px(x, y, ramp[0] if v < -0.28 else ramp[2] if v > 0.5 else ramp[1])
    def poly(self, pts, ramp):
        """A filled polygon, lit from the left: light on the left third, dark on the right."""
        ys = [p[1] for p in pts]; y0, y1 = int(min(ys)), int(max(ys))
        for y in range(y0, y1 + 1):
            xs = []
            for i in range(len(pts)):
                (xa, ya), (xb, yb) = pts[i], pts[(i + 1) % len(pts)]
                if ya == yb: continue
                if min(ya, yb) <= y + 0.5 < max(ya, yb):
                    xs.append(xa + (y + 0.5 - ya) * (xb - xa) / (yb - ya))
            xs.sort()
            for a, b in zip(xs[::2], xs[1::2]):
                a = int(math.ceil(a - 0.5)); b = int(math.floor(b - 0.5))
                w = max(1, b - a + 1)
                for x in range(a, b + 1):
                    if len(ramp) == 1: self.px(x, y, ramp); continue
                    t = (x - a) / w
                    self.px(x, y, ramp[0] if t < 0.3 else ramp[2] if t > 0.72 else ramp[1])
    def mirror(self, axis=23.5):
        for y in range(N):
            for x in range(int(axis) + 1):
                c = self.g[y][x]; mx = int(2 * axis - x)
                if c != '.' and 0 <= mx < N and self.g[y][mx] == '.': self.g[y][mx] = c
    def rows(self): return [''.join(r) for r in self.g]
    def out(self): return {"p": self.p, "r": self.rows()}

def sheet(sprites, path, scale=6, bg=(70, 76, 84), order=None):
    from PIL import Image
    names = order or list(sprites)
    img = Image.new('RGB', (len(names) * (N * scale + 8) + 8, N * scale + 16), bg)
    for k, n in enumerate(names):
        f = sprites[n]
        # the mod's outline: night behind every solid pixel's 4-neighbors
        solid = lambda x, y: 0 <= x < N and 0 <= y < N and f['r'][y][x] != '.'
        for y in range(N):
            for x in range(N):
                ch = f['r'][y][x]; rgb = None
                if ch != '.':
                    c = f['p'][ch].lstrip('#'); rgb = tuple(int(c[j:j + 2], 16) for j in (0, 2, 4))
                elif solid(x - 1, y) or solid(x + 1, y) or solid(x, y - 1) or solid(x, y + 1): rgb = (14, 18, 26)
                if rgb:
                    for dy in range(scale):
                        for dx in range(scale): img.putpixel((8 + k * (N * scale + 8) + x * scale + dx, 8 + y * scale + dy), rgb)
    img.save(path)
