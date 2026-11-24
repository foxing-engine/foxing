# Frames for the 32x32 sprites: each kind's standing drawing (make.py) moved a little,
# by a handful of operations on its rows, and a line per kind saying which it uses.
# No frame is drawn twice by hand: a breath, a blink, a sway, a wing-beat, a step, a
# stretch (striking) and a squash (struck) are all the one drawing, pushed.
#
# Output, per kind, under "a":
#   f:      the frames after the standing one, each as {row index: row} (the rows
#           that differ from the standing drawing; frame 0 is the drawing itself)
#   seq:    the idle loop, [[frame, ms], ...]
#   strike: the frame shown as it strikes; hit: as it is struck
#   step:   [left, right], for the kinds that walk
#   float:  true for the kinds that hang in the air (the picture bobs those)
import math
N = 32

def grid(rows): return [list(r) for r in rows]
def rows_of(g): return [''.join(r) for r in g]
def blank(): return [['.'] * N for _ in range(N)]

def bounds(g):
    ys = [y for y in range(N) if any(c != '.' for c in g[y])]
    return (ys[0], ys[-1]) if ys else (0, N - 1)

def breathe(g, split=None, dy=1):
    """The part above `split` a pixel lower, the rest where it was: a breath in."""
    top, bot = bounds(g)
    if split is None: split = top + int((bot - top) * 0.62)
    o = [r[:] for r in g]
    for y in range(split - 1, -1, -1):
        o[y] = g[y - dy][:] if y - dy >= 0 else ['.'] * N
    return o

def stretch(g, at=None):
    """A pixel taller about the feet: a row in the middle said twice."""
    top, bot = bounds(g)
    if at is None: at = top + int((bot - top) * 0.55)
    o = [r[:] for r in g]
    for y in range(0, at):
        o[y] = g[y + 1][:]
    return o

def blink(g, eyes, lid):
    o = [r[:] for r in g]
    for (x, y, w, h) in eyes:
        for j in range(h):
            for i in range(w):
                o[y + j][x + i] = lid if j < h - 1 or h == 1 else o[y + j][x + i]
    return o

def sway(g, amp, phase, y0=None, y1=None, wave=9.0, pivot='bottom'):
    """Rows pushed sideways along a wave, more the farther from the pivot end."""
    top, bot = bounds(g)
    y0 = top if y0 is None else y0; y1 = bot if y1 is None else y1
    o = [r[:] for r in g]
    span = max(1, y1 - y0)
    for y in range(y0, y1 + 1):
        far = (y1 - y) / span if pivot == 'bottom' else (y - y0) / span if pivot == 'top' else 1.0
        dx = int(round(amp * far * math.sin(phase + y / wave * 2 * math.pi)))
        if dx:
            row = ['.'] * N
            for x in range(N):
                if 0 <= x - dx < N: row[x] = g[y][x - dx]
            o[y] = row
    return o

def flap(g, f, core=3, axis=15.5, y0=0, y1=N - 1):
    """Wings drawn in toward the body: what lies past `core` of the axis is narrowed."""
    o = [r[:] for r in g]
    for y in range(y0, y1 + 1):
        row = ['.'] * N
        for x in range(N):
            d = x - axis; a = abs(d)
            if a <= core: row[x] = g[y][x]; continue
            src = core + (a - core) / f
            sx = int(round(axis + math.copysign(src, d)))
            if 0 <= sx < N: row[x] = g[y][sx]
        o[y] = row
    return o

def step(g, left, band=3):
    """One foot off the ground: the lowest rows of that half lifted a pixel."""
    top, bot = bounds(g)
    o = [r[:] for r in g]
    xs = range(0, 16) if left else range(16, N)
    for y in range(bot - band + 1, bot + 1):
        for x in xs:
            o[y][x] = g[y + 1][x] if y + 1 <= bot else '.'
    return o

def lift(g, x0, y0, w, h, dy):
    """A thing held out in the air (a lamp, a bell, a ring of keys) a pixel up or down."""
    o = [r[:] for r in g]
    for j in range(h):
        for i in range(w): o[y0 + j][x0 + i] = '.'
    for j in range(h):
        for i in range(w):
            y = y0 + j + dy
            if 0 <= y < N and g[y0 + j][x0 + i] != '.': o[y][x0 + i] = g[y0 + j][x0 + i]
    return o

def diff(base, g):
    return {str(y): ''.join(g[y]) for y in range(N) if ''.join(g[y]) != base[y]}

def fig_eyes(g):
    """A figure's two one-pixel eyes, wherever its head was put."""
    for y in range(4, 12):
        if g[y][14] == g[y][18] and g[y][14] != '.' and g[y][13] == g[y][15] == g[y][17] and g[y][13] != g[y][14]:
            return [(14, y, 1, 1), (18, y, 1, 1)], g[y][13]
    return None, None

# ---- which kind moves how ----
FLAME = {'wisp': (1.0, 14.0), 'dust devil': (1.6, 8.0), 'sigh': (1.2, 12.0), 'lamprey': (1.0, 14.0)}
TOPSWAY = {'anemone': (1.6, 15, 7.0), 'longsight': (1.5, 16, 12.0), 'bookworm': (1.3, 17, 10.0), 'drowned': (0, 0, 0)}
WINGS = {'mothwing': (2, 31), 'lampmoth': (2, 31), 'lanternfly': (2, 17)}
WALKS = {'citizen', 'docent'}
SHUFFLE = {'hourhand': 3, 'inkling': 4, 'hermit': 3, 'erratum': 3, 'starfish': 0, 'silverfish': 0, 'clipper': 3, 'footpad': 3}
EYES = {
    'citizen': ([(13, 11, 2, 2), (18, 11, 2, 2)], 'f'),
    'docent': ([(13, 9, 2, 2), (18, 9, 2, 2)], 'f'),
}
HELD = {  # a thing held out, bobbing against the breath: x, y, w, h
    'docent': (26, 21, 6, 10),
    'keeper': None,
}
FLOATS = {'wisp', 'mothwing', 'lampmoth', 'lanternfly', 'dust devil', 'sigh', 'hand', 'silverfish'}

def animate(name, rows):
    g = grid(rows)
    F = []                      # frames after the first
    def add(fr):
        d = diff(rows, fr)
        for i, e in enumerate(F):
            if e == d: return i + 1
        F.append(d); return len(F)
    a = {}
    hit = add(breathe(breathe(g)))          # struck: two pixels down on itself
    strike = add(stretch(g))                # striking: a pixel up out of itself
    if name in FLAME:
        amp, wave = FLAME[name]
        ph = [add(sway(g, amp, p, wave=wave, pivot='bottom' if name != 'lamprey' else 'none')) for p in (0.0, 1.57, 3.14, 4.71)]
        a['seq'] = [[i, 170] for i in ph]
    elif name in WINGS:
        c, y1 = WINGS[name]
        half = add(flap(g, 0.78, core=c, y1=y1)); shut = add(flap(g, 0.52, core=c, y1=y1))
        a['seq'] = [[0, 260], [half, 90], [shut, 110], [half, 90]]
    elif name in TOPSWAY and TOPSWAY[name][0]:
        amp, y1, wave = TOPSWAY[name]
        ph = [add(sway(g, amp, p, y0=None, y1=y1, wave=wave)) for p in (0.0, 1.57, 3.14, 4.71)]
        a['seq'] = [[i, 240] for i in ph]
    elif name in SHUFFLE and SHUFFLE[name]:
        b = SHUFFLE[name]
        l = add(step(g, True, b)); r = add(step(g, False, b))
        a['seq'] = [[0, 420], [l, 180], [0, 420], [r, 180]]
    else:
        br = add(breathe(g))
        seq = [[0, 1100], [br, 900], [0, 1100], [br, 900]]
        eyes, lid = EYES.get(name, (None, None))
        if not eyes: eyes, lid = fig_eyes(g)
        if eyes:
            bl = add(blink(g, eyes, lid))
            seq = [[0, 1100], [br, 900], [0, 700], [bl, 110], [0, 290], [br, 900], [0, 2300], [br, 900], [0, 500], [bl, 110], [0, 140], [bl, 110], [0, 900]]
        a['seq'] = seq
    if name in WALKS:
        a['step'] = [add(step(g, True)), add(step(g, False))]
    if name in FLOATS: a['float'] = True
    a['f'] = F; a['hit'] = hit; a['strike'] = strike
    return a

def frames(sprite):
    """Every frame of a sprite whole, the standing one first."""
    out = [sprite['r']]
    for d in sprite['a']['f']:
        out.append([d.get(str(y), sprite['r'][y]) for y in range(N)])
    return out
