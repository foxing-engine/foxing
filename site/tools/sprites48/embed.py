# Write the built sprites (atlas48.json, from make48.py) into the tiles mod as its HI table.
# usage: python3 embed.py <dir with atlas48.json> <path to tiles.js>
import json, sys
atlas = json.load(open(sys.argv[1] + '/atlas48.json'))
path = sys.argv[2]
s = open(path).read()
i = s.index('const HI = {\n'); j = s.index('\n};\n', i) + 4
lines = []
for name, f in atlas.items():
    body = 'p: ' + json.dumps(f['p']) + ', r: ' + json.dumps(f['r'])
    if 'a' in f: body += ', a: ' + json.dumps(f['a'], separators=(',', ':'))
    lines.append('  ' + json.dumps(name) + ': { ' + body + ' },')
s = s[:i] + 'const HI = {\n' + '\n'.join(lines) + '\n};\n' + s[j:]
open(path, 'w').write(s)
print('embedded', len(atlas), 'sprites;', len(s), 'bytes')
