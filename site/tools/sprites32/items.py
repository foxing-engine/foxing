# The things of the world at 16x16 (they were 8x8): what lies on a floor and what a
# satchel holds, painted with the same small painter as the creatures, lit from the
# upper left. Output: icons16.json, {name: {"p": palette, "r": [16 rows]}}, and a sheet.
# usage: python3 items.py <dir with paint.py>   (embed.py writes them into tiles.js)
import json, sys, math
sys.path.insert(0, sys.argv[1])
import paint
paint.N = 16
from paint import Spr
N = 16
out = {}
def I(name, pal): s = Spr(pal); out[name] = s; return s

# hollow water: a drop, a light in it
s = I('water', {'l': '#dff3fa', 'm': '#9fd3e6', 'd': '#4a7f96', 'k': '#1f3f50'})
s.poly([(8, 1), (12.5, 8), (12.5, 11), (3.5, 11), (3.5, 8)], 'k'); s.ell(8, 10, 4.6, 4.6, 'k')
s.poly([(8, 3), (11, 8.5), (5, 8.5)], 'lmd'); s.ell(8, 10, 3.6, 3.6, 'lmd'); s.px(6, 9, 'l'); s.px(6, 10, 'l'); s.px(7, 8, 'l')

# moth dust: a little heap, motes rising off it
s = I('dust', {'l': '#e6e0cf', 'm': '#bdb7a6', 'd': '#8e887c', 'k': '#4a463e', 'g': '#f6e7b8'})
s.ell(8, 12.5, 5.6, 2.6, 'k'); s.ell(8, 12, 4.8, 2.2, 'lmd'); s.ell(7, 10.5, 2.6, 1.6, 'lmd')
for x, y, c in ((4, 6, 'g'), (9, 4, 'l'), (11, 7, 'g'), (6, 3, 'm'), (12, 3, 'm'), (8, 7, 'l')): s.px(x, y, c)

# a dab of ink: a squat pot, ink at the lip, a drip
s = I('ink', {'l': '#8a92d8', 'm': '#5a62a8', 'd': '#2a2f5a', 'k': '#12142c', 'g': '#c8ccd8', 'G': '#8a90a0'})
s.ell(8, 10.5, 5.2, 4.6, 'k'); s.ell(8, 10.5, 4.3, 3.8, 'lmd'); s.rect(5, 4, 6, 3, 'k'); s.rect(6, 5, 4, 1, 'g'); s.rect(6, 6, 4, 1, 'G')
s.rect(6, 3, 4, 1, 'd'); s.px(12, 6, 'd'); s.px(12, 7, 'm'); s.px(6, 9, 'l')

# a wisp ember: a coal with a flame still on it
s = I('ember', {'y': '#fff2c8', 'o': '#ffcf7a', 'r': '#d1734f', 'd': '#7a3424', 'k': '#2a1410'})
s.ell(8, 11.5, 4.8, 3.4, 'k'); s.ell(8, 11.5, 3.9, 2.6, 'rdd'); s.px(6, 11, 'o'); s.px(9, 12, 'r'); s.px(7, 12, 'o')
s.poly([(8, 1.5), (11, 7), (9.5, 10), (6.5, 10), (5, 7)], 'r'); s.poly([(8, 4), (9.8, 7.5), (8.8, 9.5), (7, 9.5), (6.2, 7.5)], 'o'); s.px(8, 7, 'y'); s.px(8, 8, 'y'); s.px(11, 3, 'o'); s.px(4, 4, 'r')

# rag thread: a wound spool, an end loose
s = I('thread', {'l': '#f0d79a', 'm': '#c9a24f', 'd': '#8a6a2f', 'k': '#3a2a12', 'w': '#8a6f4a', 'W': '#5c4a30'})
s.rect(4, 2, 8, 2, 'W'); s.rect(4, 2, 8, 1, 'w'); s.rect(4, 12, 8, 2, 'W'); s.rect(4, 12, 8, 1, 'w')
s.rect(5, 4, 6, 8, 'm'); s.rect(5, 4, 2, 8, 'l'); s.rect(9, 4, 2, 8, 'd')
for y in (5, 7, 9, 11): s.line(5, y, 10, y - 1, 'd')
s.line(11, 6, 13, 8, 'm'); s.line(13, 8, 12, 11, 'm'); s.px(13, 12, 'l')

# a small key
s = I('key', {'l': '#f6e7b8', 'm': '#d9b57a', 'd': '#8a6a2f', 'k': '#3a2a12'})
s.ell(5, 5, 3.8, 3.8, 'k'); s.ell(5, 5, 3, 3, 'lmd'); s.ell(5, 5, 1.2, 1.2, 'k')
s.line(7, 7, 13, 13, 'k'); s.line(7, 6, 13, 12, 'm'); s.line(8, 6, 13, 11, 'l')
s.rect(11, 12, 1, 3, 'd'); s.rect(13, 12, 1, 2, 'd'); s.px(9, 10, 'd')

# a moth charm: a pale moth on a cord
s = I('charm', {'l': '#f6f3ea', 'm': '#e8e4da', 'd': '#9a8fb0', 'k': '#3a3048', 'c': '#8a6a2f', 'e': '#d1734f'})
s.line(8, 0, 8, 4, 'c'); s.px(7, 1, 'c'); s.px(9, 1, 'c')
s.poly([(8, 6), (2, 4), (1, 9), (5, 12), (8, 10)], 'lmd'); s.poly([(8, 6), (14, 4), (15, 9), (11, 12), (8, 10)], 'mdd')
s.rect(7, 5, 2, 8, 'k'); s.px(4, 7, 'e'); s.px(11, 7, 'e'); s.px(4, 8, 'd'); s.px(11, 8, 'k'); s.px(8, 14, 'd')

# a lantern: a brass cage, a flame, a ring to carry it by
s = I('lantern', {'l': '#f6e7b8', 'm': '#d9b57a', 'd': '#8a6a2f', 'k': '#3a2a12', 'y': '#fff6d6', 'o': '#ffcf7a'})
s.ell(8, 2, 2.4, 2, 'd'); s.ell(8, 2, 1.2, 0.9, '.'); s.rect(5, 4, 6, 2, 'm'); s.rect(5, 4, 6, 1, 'l'); s.rect(4, 13, 8, 2, 'd'); s.rect(4, 13, 8, 1, 'm')
s.rect(5, 6, 6, 7, 'o'); s.rect(6, 7, 4, 5, 'y'); s.px(8, 8, 'o'); s.px(8, 9, 'o'); s.px(7, 10, 'o')
s.rect(4, 6, 1, 7, 'd'); s.rect(11, 6, 1, 7, 'k'); s.rect(8, 6, 1, 1, 'd'); s.rect(8, 12, 1, 1, 'd')

# a book: bound, a ribbon, pages at the edge
s = I('book', {'l': '#a86a54', 'm': '#7a4a3a', 'd': '#4a2a20', 'k': '#24120c', 'p': '#ece5d3', 'q': '#b8ae98', 'g': '#d9b57a', 'r': '#b0442c'})
s.poly([(3, 2), (13, 2), (13, 14), (3, 14)], 'k'); s.rect(4, 3, 8, 10, 'm'); s.rect(4, 3, 2, 10, 'l'); s.rect(10, 3, 2, 10, 'd')
s.rect(12, 4, 1, 9, 'p'); s.rect(4, 13, 9, 1, 'q'); s.rect(6, 5, 4, 1, 'g'); s.rect(6, 7, 4, 1, 'g'); s.rect(7, 13, 1, 2, 'r'); s.px(4, 3, 'g'); s.px(11, 3, 'g')

# a bandage: a roll, the end unwound
s = I('bandage', {'l': '#ffffff', 'm': '#ece5d3', 'd': '#c9c0b0', 'k': '#6a6458', 'r': '#c96a6a'})
s.ell(6.5, 8, 5, 5, 'k'); s.ell(6.5, 8, 4.2, 4.2, 'lmd'); s.ell(6.5, 8, 2, 2, 'd'); s.ell(6.5, 8, 0.9, 0.9, 'k')
s.poly([(9, 11), (15, 11), (15, 14), (8, 14)], 'k'); s.rect(9, 12, 6, 1, 'm'); s.rect(9, 13, 6, 1, 'd'); s.px(13, 12, 'r'); s.px(12, 12, 'r')

# a wing, shed
s = I('wing', {'l': '#f4efcf', 'm': '#c9bfa8', 'd': '#8a8070', 'k': '#3a3428', 'e': '#a8452f'})
s.poly([(2, 12), (4, 3), (10, 1), (14, 4), (13, 9), (8, 13)], 'k'); s.poly([(3.5, 11), (5, 4), (10, 2.5), (12.8, 4.5), (12, 8.5), (8, 11.8)], 'lmd')
s.line(4, 11, 11, 4, 'd'); s.line(5, 11, 12, 7, 'd'); s.ell(9, 6, 1.4, 1.4, 'e'); s.px(9, 6, 'k')

# a lamp (a fixture): a green shade, a brass foot, light under it
s = I('lamp', {'G': '#9fd8a4', 'g': '#5fa86a', 'd': '#2f6a3c', 'k': '#16301c', 'y': '#fff6d6', 'o': '#ffe6a0', 'b': '#d9b57a', 'B': '#8a6a2f'})
s.poly([(4, 2), (12, 2), (15, 8), (1, 8)], 'k'); s.poly([(5, 3), (11, 3), (13.4, 7), (2.6, 7)], 'Ggd'); s.rect(3, 8, 10, 1, 'o'); s.rect(5, 9, 6, 1, 'y')
s.rect(7, 9, 2, 4, 'b'); s.rect(8, 9, 1, 4, 'B'); s.rect(4, 13, 8, 2, 'B'); s.rect(4, 13, 8, 1, 'b')

# a bell
s = I('bell', {'l': '#f6e7b8', 'm': '#d9b57a', 'd': '#b08a3a', 'k': '#3a2a12', 'c': '#5c4a26'})
s.rect(7, 1, 2, 2, 'c'); s.poly([(5, 3), (11, 3), (13, 11), (3, 11)], 'k'); s.poly([(6, 4), (10, 4), (11.6, 10), (4.4, 10)], 'lmd'); s.ell(8, 5, 2.6, 2, 'lmd')
s.rect(2, 11, 12, 2, 'k'); s.rect(3, 11, 10, 1, 'm'); s.px(4, 11, 'l'); s.ell(8, 14, 1.4, 1.2, 'c'); s.px(6, 6, 'l'); s.px(6, 7, 'l')

# a basin: stone, water in it
s = I('basin', {'l': '#c8c2b4', 'm': '#8e887c', 'd': '#5c574e', 'k': '#2a2824', 'w': '#9fd3e6', 'W': '#dff3fa', 'b': '#4a7f96'})
s.ell(8, 7, 7.2, 3.4, 'k'); s.poly([(1.5, 7), (14.5, 7), (11, 13), (5, 13)], 'k'); s.poly([(2.6, 8), (13.4, 8), (10.4, 12), (5.6, 12)], 'lmd')
s.ell(8, 7, 6.2, 2.5, 'lmd'); s.ell(8, 7, 5, 1.7, 'w'); s.rect(5, 6, 3, 1, 'W'); s.rect(9, 8, 2, 1, 'b'); s.rect(5, 13, 6, 2, 'd'); s.rect(5, 13, 6, 1, 'm')

# a plaque: brass on a board, lines engraved
s = I('plaque', {'w': '#7a644a', 'W': '#5a4a3a', 'k': '#2a2018', 'l': '#f6e7b8', 'm': '#d9b57a', 'd': '#8a6a2f'})
s.rect(1, 3, 14, 10, 'k'); s.rect(2, 4, 12, 8, 'W'); s.rect(2, 4, 12, 1, 'w'); s.rect(3, 5, 10, 6, 'm'); s.rect(3, 5, 10, 1, 'l'); s.rect(3, 10, 10, 1, 'd')
s.line(5, 7, 10, 7, 'd'); s.line(5, 9, 9, 9, 'd'); s.px(3, 5, 'k'); s.px(12, 5, 'k'); s.px(3, 10, 'k'); s.px(12, 10, 'k')

# river weed
s = I('weed', {'l': '#a8d890', 'm': '#7fb069', 'd': '#4a7a3a', 'k': '#1f3a18'})
for (x0, top, c) in ((5, 2, 'm'), (8, 1, 'l'), (11, 4, 'd')):
    for y in range(top, 15):
        x = x0 + round(math.sin(y * 0.9 + x0) * 1.2); s.px(x, y, c); s.px(x + 1, y, 'd' if c != 'd' else 'k')
s.px(4, 6, 'l'); s.px(13, 8, 'm'); s.px(7, 9, 'l'); s.rect(4, 14, 9, 1, 'k')

# an usher's ribbon
s = I('ribbon', {'l': '#f0a0ac', 'm': '#d87a8a', 'd': '#9a3a4a', 'k': '#4a1420', 'g': '#d9b57a'})
s.poly([(2, 3), (14, 3), (14, 7), (2, 7)], 'k'); s.rect(3, 4, 10, 2, 'm'); s.rect(3, 4, 10, 1, 'l'); s.rect(7, 3, 2, 4, 'g')
s.poly([(4, 7), (7, 7), (6, 14), (4.5, 12), (3, 14)], 'mdd'); s.poly([(9, 7), (12, 7), (13, 14), (11.5, 12), (10, 14)], 'ddk')

# an axolotl's frill: three plumes from one root
s = I('frill', {'l': '#fde3ea', 'm': '#f6cdd8', 'd': '#e59ab0', 'k': '#8a3f5c', 'r': '#e0607a'})
s.poly([(7, 13), (1, 7), (2, 3), (6, 7)], 'k'); s.poly([(6.6, 11.6), (2.2, 7), (2.8, 5), (5.4, 7.6)], 'lmd')
s.poly([(6, 12), (6, 4), (8, 0.5), (10, 4), (10, 12)], 'k'); s.poly([(7, 11), (7, 4.4), (8, 2.4), (9, 4.4), (9, 11)], 'lmd')
s.poly([(9, 13), (15, 7), (14, 3), (10, 7)], 'k'); s.poly([(9.4, 11.6), (13.8, 7), (13.2, 5), (10.6, 7.6)], 'mdd')
for x, y in ((3, 6), (8, 4), (8, 7), (12, 6), (4, 8), (11, 9)): s.px(x, y, 'r')
s.ell(8, 13.5, 2.6, 1.6, 'k'); s.ell(8, 13.3, 1.7, 0.9, 'd')

# a needle, threaded
s = I('needle', {'l': '#f2f6fa', 'm': '#b0b8c0', 'd': '#6a727a', 'r': '#d0735f', 'R': '#a8452f'})
s.line(2, 14, 13, 3, 'd'); s.line(2, 13, 13, 2, 'm'); s.line(3, 12, 12, 3, 'l'); s.px(13, 2, 'd'); s.px(12, 2, 'm'); s.px(13, 3, 'm'); s.px(12, 3, '.')
s.line(12, 3, 14, 7, 'r'); s.line(14, 7, 10, 10, 'r'); s.line(10, 10, 13, 13, 'R'); s.px(13, 14, 'r')

# an anemone's sting: a barb, a bead of venom at its point
s = I('sting', {'l': '#ecd6fa', 'm': '#d0a8f0', 'd': '#8a5ab0', 'k': '#3a1f55', 'p': '#f0a8c0', 'g': '#b8f0a0'})
s.poly([(2, 14), (3, 9), (10, 2), (13, 1.5), (12.5, 5), (6, 12)], 'k'); s.poly([(3.6, 12.4), (4.4, 9.6), (10.4, 3.4), (11.6, 3.2), (11.4, 4.6), (5.6, 11)], 'lmd')
s.ell(3.5, 13, 2, 1.8, 'p'); s.px(3, 12, 'l'); s.ell(13.5, 2, 1.4, 1.4, 'g'); s.px(14, 5, 'g'); s.px(8, 7, 'l'); s.px(9, 6, 'l')

# a starfish arm
s = I('arm', {'l': '#f6b27a', 'm': '#d9824a', 'd': '#a8552a', 'k': '#3a1c10', 'p': '#f4d2b0'})
s.poly([(2, 13), (4, 9), (12, 1.5), (14, 3), (8, 12), (5, 15)], 'k'); s.poly([(3.4, 12.6), (5, 9.6), (12, 3), (12.8, 3.6), (7.4, 11.4), (5, 13.6)], 'lmd')
for x, y in ((6, 10), (8, 8), (10, 6)): s.px(x, y, 'p')
s.px(4, 12, 'p'); s.px(11, 4, 'l')

# a spare minute: a watch face come loose, one hand
s = I('minute', {'l': '#fff7dc', 'm': '#f1e4c0', 'd': '#d8c796', 'g': '#d9b57a', 'G': '#8a6a2f', 'k': '#3a2a12', 'r': '#a8452f'})
s.ell(8, 9, 6, 6, 'k'); s.ell(8, 9, 5.2, 5.2, 'g'); s.ell(8, 9, 4.2, 4.2, 'lmd'); s.rect(7, 1, 2, 2, 'G'); s.px(8, 3, 'g')
s.line(8, 9, 8, 6, 'k'); s.line(8, 9, 10, 10, 'r'); s.px(8, 5, 'G'); s.px(12, 9, 'G'); s.px(8, 13, 'G'); s.px(4, 9, 'G')

# a sitter's patience: an hourglass, most of it still above
s = I('patience', {'w': '#8a6a2f', 'W': '#5c4a26', 'k': '#2a2018', 'g': '#cfe0e6', 'G': '#9ab0b8', 's': '#e6cd8a', 'S': '#b89a50'})
s.rect(3, 1, 10, 2, 'W'); s.rect(3, 1, 10, 1, 'w'); s.rect(3, 13, 10, 2, 'W'); s.rect(3, 13, 10, 1, 'w')
s.poly([(4, 3), (12, 3), (8.6, 8), (12, 13), (4, 13), (7.4, 8)], 'gGG'); s.poly([(5.4, 4), (10.6, 4), (8, 7)], 's'); s.px(8, 8, 'S'); s.px(8, 9, 's'); s.px(8, 10, 's')
s.poly([(6, 12.6), (10, 12.6), (8, 11)], 'S'); s.px(5, 4, 'g'); s.px(5, 12, 'g')

# coin sweat: a gold bead, wet
s = I('sweat', {'y': '#fffbe0', 'l': '#fff0b0', 'm': '#d9b57a', 'd': '#8a6a2f', 'k': '#3a2a12'})
s.poly([(8, 2), (12, 9), (4, 9)], 'k'); s.ell(8, 10, 4.6, 4.2, 'k'); s.poly([(8, 4), (10.6, 9), (5.4, 9)], 'lmd'); s.ell(8, 10, 3.6, 3.2, 'lmd')
s.px(6, 9, 'y'); s.px(6, 10, 'y'); s.px(7, 8, 'y'); s.px(12, 4, 'l'); s.px(3, 5, 'm'); s.px(13, 13, 'm')

# a lamprey's tooth
s = I('tooth', {'l': '#ffffff', 'm': '#ece5d3', 'd': '#b8ae98', 'k': '#4a463e', 'r': '#a84a4a'})
s.poly([(3, 2), (13, 2), (12, 7), (9, 14.5), (7, 14.5), (4, 7)], 'k'); s.poly([(4.4, 3), (11.6, 3), (10.8, 7), (8.6, 13), (7.4, 13), (5.2, 7)], 'lmd')
s.rect(4, 2, 8, 1, 'r'); s.px(6, 5, 'l'); s.px(6, 6, 'l')

# a hermit's shell: a spiral
s = I('shell', {'l': '#f0d7b0', 'm': '#c9a27a', 'd': '#7a5a3a', 'k': '#2e2014'})
s.ell(8, 9, 6.4, 5.6, 'k'); s.ell(8, 9, 5.5, 4.7, 'lmd'); s.poly([(3, 6), (6, 1.5), (9, 5)], 'k'); s.poly([(4.4, 5.6), (6, 3), (7.6, 5.2)], 'lmd')
for a in range(0, 20):
    t = a / 20.0; ang = t * 9.0; r = 0.6 + t * 3.6
    s.px(8.5 + math.cos(ang) * r, 9.5 + math.sin(ang) * r * 0.8, 'd')
s.ell(12, 12, 1.8, 1.4, 'k')

# clipped silver: slivers off a coin's edge
s = I('sliver', {'l': '#f4f8fb', 'm': '#c0c8d0', 'd': '#8a929a', 'k': '#3a4048'})
for (x0, y0, x1, y1) in ((3, 12, 9, 3), (6, 13, 13, 6), (9, 14, 13, 11)):
    s.line(x0 + 1, y0, x1 + 1, y1, 'k'); s.line(x0, y0, x1, y1, 'm'); s.line(x0, y0 - 1, x1, y1 - 1, 'l')
s.px(4, 5, 'l'); s.px(12, 3, 'm'); s.px(2, 9, 'd')

# a censor's lens: smoked glass in a horn rim, a handle
s = I('lens', {'w': '#7a644a', 'W': '#5a4a3a', 'k': '#2a2018', 'l': '#cfeaf2', 'm': '#9fd3e6', 'd': '#5a8a9e', 'r': '#ff5a4a'})
s.ell(6.5, 6.5, 5.6, 5.6, 'k'); s.ell(6.5, 6.5, 4.8, 4.8, 'w'); s.ell(6.5, 6.5, 3.6, 3.6, 'lmd'); s.px(5, 5, 'l'); s.px(8, 8, 'r')
s.line(10, 10, 14, 14, 'k'); s.line(10, 11, 13, 14, 'W'); s.line(11, 10, 14, 13, 'w')

# a catalogue card: ruled, a hole for the rod
s = I('card', {'l': '#f8f1de', 'm': '#ece5d3', 'd': '#c9c0b0', 'k': '#5a5448', 'i': '#7a6f5f', 'r': '#b0442c'})
s.rect(1, 3, 14, 10, 'k'); s.rect(2, 4, 12, 8, 'm'); s.rect(2, 4, 12, 1, 'l'); s.rect(2, 11, 12, 1, 'd'); s.rect(3, 6, 10, 1, 'r')
s.line(3, 8, 11, 8, 'i'); s.line(3, 10, 9, 10, 'i'); s.ell(8, 12, 1, 1, 'k')

# a ripe hour: a fruit with a clock's blush, a leaf
s = I('hour', {'l': '#f6b27a', 'm': '#d87a4a', 'd': '#a8452f', 'k': '#3a1410', 'g': '#7fb069', 'G': '#4a7a3a', 'y': '#fff2c8'})
s.ell(8, 9.5, 5.6, 5.4, 'k'); s.ell(8, 9.5, 4.7, 4.5, 'lmd'); s.line(8, 4, 9, 2, 'G'); s.poly([(9, 2), (13, 1), (12, 4)], 'g'); s.px(11, 2, 'G')
s.px(6, 8, 'y'); s.px(6, 7, 'y'); s.px(8, 5, 'k'); s.line(8, 10, 8, 8, 'k'); s.line(8, 10, 10, 10, 'k')

# a brine pearl
s = I('pearl', {'w': '#ffffff', 'l': '#e8ecf0', 'm': '#a8c0d0', 'd': '#6a8898', 'k': '#2a3a44', 'p': '#f0d8e4'})
s.ell(8, 8.5, 5.6, 5.6, 'k'); s.ell(8, 8.5, 4.7, 4.7, 'lmd'); s.px(6, 6, 'w'); s.px(6, 7, 'w'); s.px(7, 6, 'w'); s.px(10, 11, 'p'); s.px(9, 11, 'p'); s.px(13, 3, 'l'); s.px(2, 13, 'm')

# a jar (of fog, of shallows): glass, a cork, something pale turning in it
s = I('jar', {'l': '#f2f6f8', 'm': '#dfe6ea', 'd': '#9aa8b4', 'k': '#3a444c', 'c': '#b08a5a', 'C': '#7a5c36', 'f': '#ffffff'})
s.rect(5, 1, 6, 3, 'C'); s.rect(5, 1, 6, 1, 'c'); s.rect(4, 4, 8, 1, 'k'); s.ell(8, 10, 5.4, 5.4, 'k'); s.rect(4, 5, 8, 4, 'k')
s.ell(8, 10, 4.5, 4.5, 'mdd'); s.rect(5, 5, 6, 4, 'm'); s.rect(5, 5, 1, 8, 'l'); s.ell(8.5, 10.5, 2.4, 1.8, 'f'); s.px(7, 8, 'f'); s.px(10, 12, 'l')

# a shell buckler
s = I('buckler', {'l': '#e6c8a0', 'm': '#c9a27a', 'd': '#8a6a4a', 'k': '#2e2014', 'g': '#d9b57a', 'G': '#8a6a2f'})
s.ell(8, 8, 7, 7, 'k'); s.ell(8, 8, 6.1, 6.1, 'G'); s.ell(8, 8, 5.2, 5.2, 'lmd'); s.ell(8, 8, 1.8, 1.8, 'g'); s.px(7, 7, 'l')
for a in range(8):
    ang = a * math.pi / 4; s.line(8 + math.cos(ang) * 2.4, 8 + math.sin(ang) * 2.4, 8 + math.cos(ang) * 4.6, 8 + math.sin(ang) * 4.6, 'd')

# a frill salve: a small pot, pink at the brim
s = I('salve', {'l': '#b08a5a', 'm': '#8a6a4a', 'd': '#5c4630', 'k': '#2a1e12', 'p': '#f6cdd8', 'P': '#e59ab0'})
s.ell(8, 6, 5.6, 2.6, 'k'); s.poly([(2.5, 6), (13.5, 6), (12, 14), (4, 14)], 'k'); s.poly([(3.6, 7), (12.4, 7), (11.2, 13), (4.8, 13)], 'lmd')
s.ell(8, 6, 4.6, 1.8, 'P'); s.ell(7.5, 5.6, 2.6, 0.9, 'p'); s.rect(5, 10, 6, 1, 'd')

# a stinging dart
s = I('dart', {'l': '#ecd6fa', 'm': '#8a5ab0', 'k': '#3a1f55', 's': '#f2f6fa', 'S': '#c0c8d0', 'd': '#6a727a', 'f': '#d0735f'})
s.line(3, 13, 11, 5, 'd'); s.line(3, 12, 11, 4, 'S'); s.poly([(10, 2), (14, 1.5), (13.5, 6)], 'k'); s.poly([(11, 3), (13.2, 2.6), (12.8, 5)], 'lmm')
s.poly([(1, 11), (4, 11), (5, 14), (2, 15)], 'f'); s.px(2, 12, 'l'); s.px(4, 13, 'k'); s.px(12, 3, 'l')

# an orchard cordial: a bottle
s = I('cordial', {'g': '#cfe0e6', 'G': '#8aa8b0', 'k': '#2a3a40', 'l': '#ffcf7a', 'm': '#d87a4a', 'd': '#a8452f', 'c': '#b08a5a', 'p': '#ece5d3'})
s.rect(7, 1, 2, 2, 'c'); s.rect(6, 3, 4, 3, 'k'); s.rect(7, 3, 2, 3, 'g'); s.poly([(4, 6), (12, 6), (12, 15), (4, 15)], 'k'); s.rect(5, 7, 6, 7, 'G'); s.rect(5, 9, 6, 5, 'm'); s.rect(5, 9, 2, 5, 'l'); s.rect(9, 9, 2, 5, 'd')
s.rect(6, 10, 4, 3, 'p'); s.px(7, 11, 'd'); s.px(8, 11, 'd'); s.px(5, 7, 'g')

# a thing: a parcel tied with string
s = I('thing', {'l': '#d6cfbc', 'm': '#bdb7a6', 'd': '#7a6f5f', 'k': '#3a342a', 's': '#8a6a2f'})
s.poly([(2, 5), (14, 5), (14, 14), (2, 14)], 'k'); s.rect(3, 6, 10, 7, 'm'); s.rect(3, 6, 3, 7, 'l'); s.rect(10, 6, 3, 7, 'd')
s.rect(7, 5, 2, 9, 's'); s.rect(3, 9, 10, 1, 's'); s.ell(8, 4, 2.4, 1.6, 's'); s.px(8, 4, 'k')

res = {n: s.out() for n, s in out.items()}
for n, f in res.items():
    f['r'] = [r[:N] for r in f['r'][:N]]
    used = set(''.join(f['r'])) - {'.'}
    assert used <= set(f['p']), (n, used - set(f['p']))
    f['p'] = {k: v for k, v in f['p'].items() if k in used}
json.dump(res, open(sys.argv[1] + '/icons16.json', 'w'))
from PIL import Image
sc = 8; names = list(res); per = 13
img = Image.new('RGB', (per * (N * sc + 8) + 8, ((len(names) + per - 1) // per) * (N * sc + 8) + 8), (58, 66, 60))
for k, n in enumerate(names):
    f = res[n]; ox = 8 + (k % per) * (N * sc + 8); oy = 8 + (k // per) * (N * sc + 8)
    for y in range(N):
        for x in range(N):
            ch = f['r'][y][x]
            if ch == '.': continue
            c = f['p'][ch].lstrip('#'); rgb = tuple(int(c[i:i + 2], 16) for i in (0, 2, 4))
            for dy in range(sc):
                for dx in range(sc): img.putpixel((ox + x * sc + dx, oy + y * sc + dy), rgb)
img.save(sys.argv[1] + '/icons16.png')
print('ok', len(res))
