# The painter at 48: the same calls as sprites32/paint.py, in the same 32-wide
# coordinates, drawn half again as fine. A shape (an ellipse, a polygon) is cut
# afresh on the finer grid, so its curve is a curve of 48 and not 32 blown up; a
# single pixel or a one-wide line, which has no shape to recut, comes out two wide
# and one wide by turns. make.py is run against this file unchanged, and the four
# figures (figures.py) are drawn here at 48 outright with `Spr.fine`.
import json, math, sys
N = 32            # the coordinates the callers speak
SIZE = 48         # the grid they are drawn on
K = SIZE / N

def _edge(v): return int(math.floor(v * K + 0.5))

class Spr:
    def __init__(self, pal, fine=False):
        self.p = dict(pal); self.g = [['.'] * SIZE for _ in range(SIZE)]
        self.k = 1.0 if fine else K
    # -- one cell of the finer grid
    def _put(self, x, y, c):
        if 0 <= x < SIZE and 0 <= y < SIZE and c: self.g[y][x] = c
    def _span(self, v, w=1):
        if self.k == 1.0: return range(int(round(v)), int(round(v)) + int(w))
        return range(_edge(v), max(_edge(v) + 1, _edge(v + w)))
    def px(self, x, y, c):
        x = int(round(x)); y = int(round(y))
        for j in self._span(y):
            for i in self._span(x): self._put(i, j, c)
    def get(self, x, y):
        if self.k == 1.0: i, j = int(x), int(y)
        else: i, j = _edge(x), _edge(y)
        return self.g[j][i] if 0 <= i < SIZE and 0 <= j < SIZE else '.'
    def rect(self, x, y, w, h, c):
        for j in self._span(y, int(h)):
            for i in self._span(x, int(w)): self._put(i, j, c)
    def line(self, x0, y0, x1, y1, c):
        k = self.k
        if k == 1.0:
            n = int(max(abs(x1 - x0), abs(y1 - y0))) or 1
            for i in range(n + 1): self._put(int(round(x0 + (x1 - x0) * i / n)), int(round(y0 + (y1 - y0) * i / n)), c)
            return
        # centers of the end pixels, on the finer grid; a line a pixel wide at 32 is
        # a pixel and a half here: two wide where it runs steep or flat, one on a slant
        ax, ay, bx, by = (x0 + 0.5) * k - 0.5, (y0 + 0.5) * k - 0.5, (x1 + 0.5) * k - 0.5, (y1 + 0.5) * k - 0.5
        n = int(max(abs(bx - ax), abs(by - ay))) or 1
        steep = abs(by - ay) >= abs(bx - ax)
        slant = min(abs(bx - ax), abs(by - ay)) > 0.35 * max(abs(bx - ax), abs(by - ay))
        for i in range(n + 1):
            x = ax + (bx - ax) * i / n; y = ay + (by - ay) * i / n
            self._put(int(round(x)), int(round(y)), c)
            if not slant:
                if steep: self._put(int(round(x)) + 1, int(round(y)), c)
                else: self._put(int(round(x)), int(round(y)) + 1, c)
    def ell(self, cx, cy, rx, ry, ramp, lx=-0.6, ly=-0.7):
        k = self.k; cx *= k; cy *= k; rx *= k; ry *= k
        for y in range(SIZE):
            for x in range(SIZE):
                dx = (x + 0.5 - cx) / rx; dy = (y + 0.5 - cy) / ry
                d = dx * dx + dy * dy
                if d <= 1:
                    if len(ramp) == 1: self._put(x, y, ramp); continue
                    lit = -(dx * lx + dy * ly)
                    v = lit * 0.9 + d * 0.35
                    self._put(x, y, ramp[0] if v < -0.28 else ramp[2] if v > 0.5 else ramp[1])
    def poly(self, pts, ramp):
        k = self.k
        if k != 1.0: pts = [((x + 0.5) * k - 0.5, (y + 0.5) * k - 0.5 + (0.25 if k != 1 else 0)) for x, y in pts]
        ys = [p[1] for p in pts]; y0, y1 = int(math.floor(min(ys))), int(math.ceil(max(ys)))
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
                    if len(ramp) == 1: self._put(x, y, ramp); continue
                    t = (x - a) / w
                    self._put(x, y, ramp[0] if t < 0.3 else ramp[2] if t > 0.72 else ramp[1])
    def mirror(self, axis=15.5):
        ax = (axis + 0.5) * self.k - 0.5
        for y in range(SIZE):
            for x in range(int(ax) + 1):
                c = self.g[y][x]; mx = int(round(2 * ax - x))
                if c != '.' and 0 <= mx < SIZE and self.g[y][mx] == '.': self.g[y][mx] = c
    def rows(self): return [''.join(r) for r in self.g]
    def out(self): return {"p": self.p, "r": self.rows()}

def sheet(sprites, path, scale=4, bg=(70, 76, 84), order=None, per_row=14):
    from PIL import Image
    names = order or list(sprites)
    cell = SIZE * scale + 8
    rows_n = (len(names) + per_row - 1) // per_row
    img = Image.new('RGB', (min(len(names), per_row) * cell + 8, rows_n * cell + 8), bg)
    for k, n in enumerate(names):
        f = sprites[n]; ox = 8 + (k % per_row) * cell; oy = 8 + (k // per_row) * cell
        solid = lambda x, y: 0 <= x < SIZE and 0 <= y < SIZE and f['r'][y][x] != '.'
        for y in range(SIZE):
            for x in range(SIZE):
                ch = f['r'][y][x]; rgb = None
                if ch != '.':
                    c = f['p'][ch].lstrip('#'); rgb = tuple(int(c[j:j + 2], 16) for j in (0, 2, 4))
                elif solid(x - 1, y) or solid(x + 1, y) or solid(x, y - 1) or solid(x, y + 1): rgb = (14, 18, 26)
                if rgb:
                    for dy in range(scale):
                        for dx in range(scale): img.putpixel((ox + x * scale + dx, oy + y * scale + dy), rgb)
    img.save(path)
