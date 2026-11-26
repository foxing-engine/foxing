# The four figures a name may choose, one for each loop: drawn at 48 outright, with
# empty hands. What a figure holds is drawn over it (HELD, below): a thing in the
# hand, one on the other arm, one at the neck, a light at the belt.
import math as _m
from paint import sheet
from paint import Spr as _Spr
def Spr(pal): return _Spr(pal, fine=True)
out = {}
SKIN = {'k': '#e8c9a8', 'K': '#c79f7c', 'q': '#8c6a50', 'e': '#1a1612', 'w': '#f6f1e4'}

def face(s, cx, cy, rx=5.2, ry=5.6):
    s.ell(cx, cy, rx, ry, 'kKq')
    s.rect(cx - 3, cy - 0.5, 2, 2, 'e'); s.rect(cx + 2, cy - 0.5, 2, 2, 'e')
    s.px(cx - 3, cy - 0.5, 'w'); s.px(cx + 2, cy - 0.5, 'w')
    s.px(cx - 1, cy + 3, 'q'); s.px(cx, cy + 3, 'q'); s.px(cx + 1, cy + 3, 'q')

def boots(s, y, c, hi):
    s.rect(15, y, 7, 4, c); s.rect(26, y, 7, 4, c); s.rect(15, y, 7, 1, hi); s.rect(26, y, 7, 1, hi)
    s.rect(14, y + 3, 2, 1, c); s.rect(32, y + 3, 2, 1, c)

# ---- the hunter: a short hooded cloak in fog green, leather, empty-handed
s = Spr({**SKIN, 'a': '#7f9a78', 'b': '#5d7a5c', 'c': '#3f5a44', 'l': '#b07a4a', 'm': '#855a36', 'n': '#5a3c24', 'd': '#3a3028', 'g': '#d9b57a', 'f': '#e9e1c8', 'r': '#b5483c'})
s.poly([(15, 30), (33, 30), (32, 41), (16, 41)], 'lmn')          # leggings
s.rect(23, 33, 2, 8, 'n')
s.poly([(12, 19), (36, 19), (38, 33), (10, 33)], 'abc')          # cloak, short
s.poly([(10, 33), (14, 33), (12, 36)], 'c'); s.poly([(20, 33), (26, 33), (23, 37)], 'c'); s.poly([(33, 33), (38, 33), (36, 36)], 'c')
s.ell(24, 12, 9.4, 9.2, 'abc')                                   # hood
s.poly([(19, 3), (24, 0), (27, 4)], 'abb')                        # hood's peak
face(s, 24, 13.5)
s.ell(24, 8.6, 6.4, 2.6, 'c')                                    # hood's shadow on the brow
s.rect(13, 27, 22, 2, 'm'); s.rect(22, 26, 4, 4, 'g'); s.rect(23, 27, 2, 2, 'n')   # belt
s.ell(11, 26, 2.6, 6, 'abc'); s.ell(37, 26, 2.6, 6, 'bcc')       # arms under the cloak
s.rect(9, 31, 4, 3, 'k'); s.rect(35, 31, 4, 3, 'K')
boots(s, 41, 'n', 'm')
out['figure-hunter'] = s.out()

# ---- the teller: a feathered cap, a ruff, a slashed doublet in plum and gold, hose
s = Spr({**SKIN, 'a': '#a8648c', 'b': '#7d4468', 'c': '#55284a', 'g': '#e3bf78', 'h': '#b08d4a', 'f': '#f6f1e4', 'i': '#cfc9bb', 'o': '#d86a4a', 'p': '#f0a070', 'd': '#3a3028', 'n': '#2a2420', 'u': '#6f7fa8', 'v': '#4c5a80'})
s.poly([(17, 31), (23, 31), (22, 42), (17, 42)], 'uvv'); s.poly([(25, 31), (31, 31), (31, 42), (26, 42)], 'uvv')   # hose
s.ell(18.5, 31, 5.2, 4.2, 'abc'); s.ell(29.5, 31, 5.2, 4.2, 'abc')     # puffed breeches
for x in (16, 19, 28, 31): s.line(x, 28, x, 34, 'g')
s.poly([(14, 18), (34, 18), (32, 30), (16, 30)], 'abc')                # doublet
for y in range(20, 30, 2): s.px(24, y, 'g')
s.line(18, 20, 20, 28, 'g'); s.line(30, 20, 28, 28, 'h')
s.ell(11.5, 22, 4.4, 5, 'abc'); s.ell(36.5, 22, 4.4, 5, 'bcc')         # puffed sleeves
s.line(10, 19, 10, 25, 'g'); s.line(13, 19, 13, 25, 'g'); s.line(35, 19, 35, 25, 'h'); s.line(38, 19, 38, 25, 'h')
s.rect(9, 27, 5, 3, 'f'); s.rect(34, 27, 5, 3, 'i'); s.rect(10, 30, 3, 2, 'k'); s.rect(35, 30, 3, 2, 'K')
face(s, 24, 11.5)
s.ell(24, 17.6, 9, 2.6, 'fii')                                          # the ruff
for x in range(16, 33, 2): s.px(x, 18, 'i')
s.poly([(14, 7), (34, 7), (33, 3), (22, 1), (15, 4)], 'abc')            # cap
s.rect(13, 7, 23, 2, 'c')
s.line(33, 5, 41, 0, 'o'); s.line(34, 6, 42, 2, 'p'); s.line(33, 4, 39, 0, 'p'); s.px(42, 1, 'o')   # feather
s.rect(31, 5, 2, 2, 'g')
s.rect(15, 42, 8, 3, 'n'); s.rect(25, 42, 8, 3, 'n'); s.rect(14, 44, 2, 1, 'n'); s.rect(32, 44, 2, 1, 'n'); s.rect(17, 42, 3, 1, 'g'); s.rect(28, 42, 3, 1, 'g')
out['figure-teller'] = s.out()

# ---- the keeper: a kerchief, rolled sleeves, a long apron over hearth-red
s = Spr({**SKIN, 'a': '#c8704e', 'b': '#a04f36', 'c': '#733524', 'f': '#f1ece0', 'i': '#d6d0c2', 'j': '#a8a294', 'g': '#e3bf78', 'h': '#a8853e', 'n': '#4a382a', 'm': '#6b5340', 'y': '#e9d28a', 'z': '#c4a95a'})
s.poly([(12, 28), (36, 28), (38, 43), (10, 43)], 'abc')                 # skirt, wide
s.ell(24, 23, 11.5, 8, 'abc')                                           # bodice, stout
s.poly([(16, 20), (32, 20), (34, 42), (14, 42)], 'fij')                 # apron
s.rect(18, 30, 5, 5, 'i'); s.rect(18, 30, 5, 1, 'j')                    # its pocket
s.line(16, 20, 20, 15, 'i'); s.line(32, 20, 28, 15, 'j')                # straps
s.rect(14, 26, 20, 1, 'j')
s.ell(10, 23, 3.6, 5.4, 'abc'); s.ell(38, 23, 3.6, 5.4, 'bcc')          # sleeves, rolled
s.rect(7, 27, 6, 2, 'f'); s.rect(35, 27, 6, 2, 'i')
s.ell(10, 31, 2.6, 2.6, 'kKq'); s.ell(38, 31, 2.6, 2.6, 'Kqq')          # forearms
face(s, 24, 11.5, 5.6, 5.6)
s.poly([(15, 9), (33, 9), (31, 3), (24, 1), (17, 3)], 'yzz')            # kerchief
s.rect(15, 8, 18, 2, 'z'); s.poly([(32, 8), (37, 10), (34, 13)], 'yzz'); s.px(18, 5, 'f'); s.px(22, 3, 'f'); s.px(28, 5, 'f')
s.rect(15, 43, 8, 3, 'n'); s.rect(25, 43, 8, 3, 'n'); s.rect(15, 43, 8, 1, 'm'); s.rect(25, 43, 8, 1, 'm')
out['figure-keeper'] = s.out()

# ---- the reader: tall and narrow, an ink-blue gown, round spectacles
s = Spr({**SKIN, 'a': '#6f86b4', 'b': '#4c6190', 'c': '#2f3f66', 'f': '#f1ece0', 'i': '#cfc9bb', 'g': '#e3bf78', 'h': '#a8853e', 'n': '#2a2420', 'r': '#a8483c', 's': '#7a2f28', 't': '#d9d4c7', 'x': '#9aa0aa'})
s.poly([(17, 17), (31, 17), (35, 44), (13, 44)], 'abc')                 # gown, long
for y in range(20, 44): s.px(24, y, 'c')
s.rect(13, 43, 23, 1, 'c')
s.poly([(19, 17), (29, 17), (27, 22), (21, 22)], 'fii')                 # collar bands
s.px(24, 23, 'g'); s.px(24, 26, 'g'); s.px(24, 29, 'g')
s.ell(13.5, 24, 3.2, 7.4, 'abc'); s.ell(34.5, 24, 3.2, 7.4, 'bcc')      # long sleeves
s.rect(11, 31, 5, 2, 'f'); s.rect(32, 31, 5, 2, 'i'); s.rect(12, 33, 3, 2, 'k')
s.rect(33, 33, 3, 2, 'K')
face(s, 24, 10.5, 4.8, 5.8)
s.rect(19, 9, 4, 4, 'x'); s.rect(25, 9, 4, 4, 'x'); s.rect(20, 10, 2, 2, 'f'); s.rect(26, 10, 2, 2, 'f')   # spectacles
s.px(21, 11, 'e'); s.px(27, 11, 'e'); s.rect(23, 10, 2, 1, 'x'); s.px(18, 10, 'x'); s.px(29, 10, 'x')
s.poly([(17, 6), (31, 6), (30, 2), (18, 2)], 'abc')                     # a flat scholar's cap
s.rect(15, 5, 18, 2, 'c'); s.line(31, 5, 34, 9, 'g'); s.rect(33, 9, 2, 3, 'g')   # and its tassel
s.ell(18, 8, 1.6, 2.4, 'i'); s.ell(30, 8, 1.6, 2.4, 'i')                # grey at the temples
s.rect(17, 44, 6, 3, 'n'); s.rect(25, 44, 6, 3, 'n')
out['figure-reader'] = s.out()

FIGURES = out

# ---- what is held, drawn over a figure. Each is painted on a clear sheet in the
# figure's own place for it; make48.py lays it on the figure, runs the figure's
# frames over the two together, and keeps what the thing adds to each frame.
STEEL = {'s': '#e6e9ee', 't': '#aab2bd', 'u': '#6f7884', 'y': '#c9a45e', 'g': '#d9b57a', 'n': '#5a3c24', 'l': '#b07a4a', 'm': '#855a36'}
WOOD = {'L': '#e2c08a', 'M': '#c29a5e', 'N': '#8a6436', 'R': '#3a2a1c', 'S': '#f1e6c8'}
# where each figure's hands, neck and belt are: (hand x, y), (other hand x, y), (neck x, y), (belt x, y)
AT = {'hunter': ((9, 31), (35, 31), (23, 21), (16, 29)), 'teller': ((10, 30), (35, 30), (23, 20), (15, 31)),
      'keeper': ((9, 30), (37, 30), (23, 17), (14, 31)), 'reader': ((12, 33), (33, 33), (23, 23), (17, 34))}

def sheet_of(pal): return Spr({**SKIN, **pal})

def sword(fig):
    s = sheet_of(STEEL)
    for i in range(13):                                              # the blade, two wide, lit on its upper edge
        s.px(9 - i * 0.55, 30 - i, 's'); s.px(10 - i * 0.55, 30 - i, 't')
    s.px(2, 17, 's')
    s.line(6, 31, 13, 30, 'y'); s.px(6, 31, 'g'); s.px(13, 30, 'g')  # crossguard
    s.rect(9, 31, 4, 3, 'k'); s.rect(10, 34, 2, 2, 'n'); s.px(10, 36, 'y')   # the fist on the grip, and a pommel
    return s

def dart(fig):
    (hx, hy) = AT[fig][0]
    s = sheet_of({**STEEL, 'r': '#d1574a', 'R': '#8f352c', 'w': '#b08a5a', 'W': '#7a5c38'})
    for i in range(-4, 9):                                           # the shaft, two wide, up and out through the fist
        s.px(hx + 1 - i * 0.5, hy + 1 - i, 'w'); s.px(hx + 2 - i * 0.5, hy + 1 - i, 'W')
    for i in range(9, 13): s.px(hx + 1 - i * 0.5, hy + 1 - i, 's'); s.px(hx + 2 - i * 0.5, hy + 1 - i, 't')   # its point
    s.px(hx + 1 - 6.5, hy - 12, 's')
    for i in (-3, -4, -5): s.px(hx - i * 0.5, hy + 1 - i, 'r'); s.px(hx + 3 - i * 0.5, hy + 1 - i, 'R')       # the flight
    s.rect(hx, hy, 3 if fig != 'hunter' else 4, 3 if fig == 'hunter' else 2, 'k')
    return s

def lute(fig):
    s = sheet_of(WOOD)
    s.ell(31.5, 31, 5.6, 6.6, 'LMN')
    s.ell(31.5, 30, 1.8, 1.8, 'R')
    for i in range(15):                                              # the neck, two wide
        s.px(28 - i * 0.9, 26 - i * 0.55, 'N'); s.px(28 - i * 0.9, 27 - i * 0.55, 'M')
    s.rect(13, 17, 3, 2, 'N'); s.px(12, 16, 'S'); s.px(14, 16, 'S'); s.px(12, 19, 'S')   # pegbox and pegs
    s.line(18, 22, 33, 33, 'S')                                      # a string catching the light
    s.rect(34, 28, 4, 3, 'k'); s.rect(20, 23, 4, 3, 'k')             # a hand on the strings, a hand on the neck
    return s

def keys(fig):
    (hx, hy) = AT[fig][0]
    s = sheet_of({'g': '#f0cf86', 'h': '#b8913f', 'i': '#3a3028'})
    cx, cy = hx + 1.5, hy + 5.5
    s.ell(cx, cy, 3.2, 3.2, 'g'); s.ell(cx, cy, 1.6, 1.6, 'i')      # the ring, hung from the fist
    s.rect(cx - 2, cy + 3, 1, 4, 'h'); s.px(cx - 3, cy + 6, 'h'); s.px(cx - 3, cy + 4, 'h')
    s.rect(cx, cy + 3, 1, 5, 'g'); s.px(cx + 1, cy + 7, 'g'); s.px(cx + 1, cy + 5, 'g')
    s.rect(cx + 2, cy + 2, 1, 4, 'h'); s.px(cx + 3, cy + 5, 'h')
    return s

def primer(fig):
    (hx, hy) = AT[fig][0]
    s = sheet_of({'r': '#6f8f6a', 's': '#48664a', 't': '#f1ece0', 'g': '#e3bf78'})
    x, y = hx - 3, hy - 7
    s.poly([(x, y + 1), (x + 8, y), (x + 8, y + 10), (x, y + 11)], 'rss')   # a small book, held to the chest
    s.rect(x + 7, y + 1, 1, 9, 't'); s.rect(x + 2, y + 3, 4, 1, 'g'); s.rect(x + 2, y + 5, 3, 1, 'g')
    s.rect(hx, hy, 3, 2, 'k')                                        # the hand on it
    return s

def shield(fig, pal, r):
    (ox, oy) = AT[fig][1]
    s = sheet_of({**STEEL, **pal})
    cx, cy = ox + 3.5, oy - 2
    s.ell(cx, cy, r, r, 'lmn')
    for a in range(0, 360, 10):
        s.px(cx + (r - 0.4) * _m.cos(_m.radians(a)) - 0.5, cy + (r - 0.4) * _m.sin(_m.radians(a)) - 0.5, 'u' if 20 < a < 200 else 't')
    s.ell(cx, cy, 2.0, 2.0, 'stu'); s.px(cx - 1.5, cy - 1, 's')
    return s
def targe(fig): return shield(fig, {}, 5.6)                          # leather on board
def buckler(fig):                                                    # a shell, pale and ribbed
    s = shield(fig, {'l': '#f3ead6', 'm': '#d8cbae', 'n': '#8f8166'}, 5.8)
    (ox, oy) = AT[fig][1]; cx, cy = ox + 3.5, oy - 2
    for a in range(15, 360, 45): s.line(cx + 2.6 * _m.cos(_m.radians(a)), cy + 2.6 * _m.sin(_m.radians(a)), cx + 4.6 * _m.cos(_m.radians(a)), cy + 4.6 * _m.sin(_m.radians(a)), 'n')
    return s

def pendant(fig, pal):
    (nx, ny) = AT[fig][2]
    s = sheet_of({'c': '#3a3028', **pal})
    s.px(nx - 2, ny - 2, 'c'); s.px(nx - 1, ny - 1, 'c'); s.px(nx + 3, ny - 2, 'c'); s.px(nx + 2, ny - 1, 'c')   # the cord
    s.rect(nx, ny, 2, 3, 'B'); s.rect(nx - 1, ny + 1, 4, 1, 'B'); s.px(nx, ny, 'A'); s.px(nx, ny + 1, 'A'); s.px(nx + 1, ny + 2, 'C')
    return s
def charm(fig): return pendant(fig, {'A': '#fff2c8', 'B': '#e3bf78', 'C': '#a8853e'})
def lens(fig): return pendant(fig, {'A': '#ffffff', 'B': '#bfe3ef', 'C': '#6fa3b8'})
def brine(fig): return pendant(fig, {'A': '#e8fff2', 'B': '#7fc9a4', 'C': '#3f8a68'})

def lantern(fig):
    (bx, by) = AT[fig][3]
    s = sheet_of({'d': '#2a2420', 'D': '#5a4a36', 'y': '#fff2c8', 'o': '#ffcf7a', 'r': '#e09a4a'})
    s.px(bx + 2, by - 1, 'D'); s.rect(bx + 1, by, 3, 1, 'd')         # hung by its ring
    s.rect(bx, by + 1, 5, 6, 'd'); s.rect(bx + 1, by + 2, 3, 4, 'o'); s.px(bx + 2, by + 3, 'y'); s.px(bx + 2, by + 4, 'y'); s.px(bx + 1, by + 5, 'r'); s.px(bx + 3, by + 5, 'r')
    s.rect(bx, by + 7, 5, 1, 'D')
    return s

OWN = {'hunter': ('sword', sword), 'teller': ('lute', lute), 'keeper': ('keys', keys), 'reader': ('primer', primer)}
def held():
    """{figure: {kind: (palette, rows)}}: every kind a figure may be seen holding."""
    res = {}
    for fig in AT:
        kinds = {'dart': dart, OWN[fig][0]: OWN[fig][1], 'buckler': buckler, 'targe': targe, 'charm': charm, 'lens': lens, 'brine': brine, 'lantern': lantern}
        res['figure-' + fig] = {k: (lambda o: (o['p'], o['r']))(f(fig).out()) for k, f in kinds.items()}
    return res
