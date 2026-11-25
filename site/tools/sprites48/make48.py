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
dest = argv[1]
os.makedirs(dest, exist_ok=True)
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
