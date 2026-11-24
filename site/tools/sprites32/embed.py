# Write the built sprites (atlas32.json, from make.py) into the tiles mod as its HI table.
# usage: python3 embed.py <dir with atlas32.json (and icons16.json)> <path to tiles.js>
import json, sys
atlas = json.load(open(sys.argv[1] + '/atlas32.json'))
path = sys.argv[2]
s = open(path).read()
i = s.index('const HI = {\n'); j = s.index('\n};\n', i) + 4
lines = []
for name, f in atlas.items():
    body = 'p: ' + json.dumps(f['p']) + ', r: ' + json.dumps(f['r'])
    if 'a' in f: body += ', a: ' + json.dumps(f['a'], separators=(',', ':'))
    lines.append('  ' + json.dumps(name) + ': { ' + body + ' },')
s = s[:i] + 'const HI = {\n' + '\n'.join(lines) + '\n};\n' + s[j:]
# The things, at 16 pixels (items.py), when they were built too.
import os
if os.path.exists(sys.argv[1] + '/icons16.json'):
    icons = json.load(open(sys.argv[1] + '/icons16.json'))
    i = s.index('const ICONS = {\n'); j = s.index('\n};\n', i) + 4
    s = s[:i] + 'const ICONS = {\n' + '\n'.join('  ' + json.dumps(n) + ': { p: ' + json.dumps(f['p']) + ', r: ' + json.dumps(f['r']) + ' },' for n, f in icons.items()) + '\n};\n' + s[j:]
    print('embedded', len(icons), 'things')
open(path, 'w').write(s)
print('embedded', len(atlas), 'sprites;', len(s), 'bytes')
