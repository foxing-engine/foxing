# Every kind at 48: the drawings of sprites32/make.py run against the finer painter
# here, then the four figures a name may choose (figures.py), then the frames.
# usage: python3 make48.py <out dir>
import json, os, sys
here = os.path.dirname(os.path.abspath(__file__))
src = open(os.path.join(here, '..', 'sprites32', 'make.py')).read()
src = src[:src.index('# ---- the frames')]
sys.path.insert(0, here)
argv = sys.argv; sys.argv = [argv[0], here]
env = {'__name__': 'drawings'}
exec(compile(src, 'sprites32/make.py', 'exec'), env)
sys.argv = argv
out, order = env['out'], env['order']
from paint import sheet, SIZE
import figures
for name, f in figures.FIGURES.items(): out[name] = f
order = list(figures.FIGURES) + order
for n in order:
    assert len(out[n]['r']) == SIZE and all(len(r) == SIZE for r in out[n]['r']), n
    used = set(''.join(out[n]['r'])) - {'.'}
    assert used <= set(out[n]['p']), (n, used - set(out[n]['p']))
# ---- the frames: each drawing moved a little (animate.py says how, kind by kind)
from animate import animate, frames
for n in order: out[n]['a'] = animate(n, out[n]['r'])
# ---- what a figure holds (figures.held): each thing laid on its figure and run
# through the figure's frames with it, and what it adds to each frame kept as runs,
# {row: [[x, cells], ...]}. Its colors get letters no figure uses, one table for all.
from animate import frames as _frames
pool = {}
def letter(hexcolor):
    if hexcolor not in pool: pool[hexcolor] = chr(0xC0 + len(pool))
    return pool[hexcolor]
held = {}
for fig, kinds in figures.held().items():
    base = out[fig]; base_frames = _frames(base)
    held[fig] = {}
    for kind, (pal, rows) in kinds.items():
        both = [''.join(letter(pal[o]) if o != '.' else b for b, o in zip(br, orow)) for br, orow in zip(base['r'], rows)]
        with_it = {'r': both, 'a': animate(fig, both)}
        fr = _frames(with_it)
        assert len(fr) == len(base_frames), (fig, kind, len(fr), len(base_frames))
        per = []
        for bf, wf in zip(base_frames, fr):
            runs = {}
            for y in range(SIZE):
                x = 0
                while x < SIZE:
                    if wf[y][x] != bf[y][x]:
                        x0 = x
                        while x < SIZE and wf[y][x] != bf[y][x]: x += 1
                        assert '.' not in wf[y][x0:x], (fig, kind, y)
                        runs.setdefault(str(y), []).append([x0, wf[y][x0:x]])
                    else: x += 1
            per.append(runs)
        held[fig][kind] = [0 if k and r == per[0] else r for k, r in enumerate(per)]
dest = argv[1]
os.makedirs(dest, exist_ok=True)
json.dump({'p': {v: k for k, v in pool.items()}, 'f': held}, open(dest + '/held48.json', 'w'))
# a sheet to look at: each figure empty-handed, then with its own thing, then with everything else
def laid(fig, kinds):
    rows = [list(r) for r in out[fig]['r']]
    for k in kinds:
        for y, runs in held[fig][k][0].items():
            for x0, cells in runs:
                for i, c in enumerate(cells): rows[int(y)][x0 + i] = c
    return {'p': {**out[fig]['p'], **{v: k for k, v in pool.items()}}, 'r': [''.join(r) for r in rows]}
look = {}
for fig in figures.FIGURES:
    own = figures.OWN[fig[7:]][0]
    look[fig] = out[fig]; look[fig + '+own'] = laid(fig, [own, 'targe', 'charm']); look[fig + '+dart'] = laid(fig, ['dart', 'buckler', 'lens', 'lantern'])
sheet(look, dest + '/held-look.png', 6, order=list(look), per_row=6)
json.dump({n: out[n] for n in order}, open(dest + '/atlas48.json', 'w'))
sheet(out, dest + '/sheet48.png', 3, order=order, per_row=10)
sheet(out, dest + '/sheet48-small.png', 1, bg=(92, 98, 104), order=order, per_row=20)
print('ok', len(order))

def frame_sheet(path, names, scale=3):
    from PIL import Image
    most = max(len(frames(out[n])) for n in names)
    img = Image.new('RGB', (most * (SIZE * scale + 6) + 6, len(names) * (SIZE * scale + 6) + 6), (70, 76, 84))
    for j, n in enumerate(names):
        f = out[n]
        for k, rows in enumerate(frames(f)):
            for y in range(SIZE):
                for x in range(SIZE):
                    ch = rows[y][x]
                    if ch == '.': continue
                    c = f['p'][ch].lstrip('#'); rgb = tuple(int(c[i:i + 2], 16) for i in (0, 2, 4))
                    for dy in range(scale):
                        for dx in range(scale): img.putpixel((6 + k * (SIZE * scale + 6) + x * scale + dx, 6 + j * (SIZE * scale + 6) + y * scale + dy), rgb)
    img.save(path)
frame_sheet(dest + '/frames-figures.png', order[:6])
print('frames', sum(len(out[n]['a']['f']) for n in order), 'bytes', len(json.dumps({n: out[n] for n in order}, separators=(',', ':'))))
