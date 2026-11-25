import sys
from paint48 import Spr, sheet, N
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

# ---- the hunter: a short hooded cloak in fog green, leather, a quiver over the shoulder
s = Spr({**SKIN, 'a': '#7f9a78', 'b': '#5d7a5c', 'c': '#3f5a44', 'l': '#b07a4a', 'm': '#855a36', 'n': '#5a3c24', 'd': '#3a3028', 'g': '#d9b57a', 'f': '#e9e1c8', 'r': '#b5483c'})
s.poly([(33, 8), (37, 6), (39, 24), (35, 25)], 'lmn')            # quiver behind the shoulder
for x in (34, 36, 38): s.line(x, 2, x, 7, 'f'); s.px(x, 2, 'r')
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
out['hunter'] = s.out()

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
out['teller'] = s.out()

# ---- the keeper: a kerchief, rolled sleeves, a long apron over hearth-red, keys at the hip
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
s.ell(33.5, 30, 2.2, 2.2, 'g'); s.ell(33.5, 30, 1.0, 1.0, 'i')          # key ring
s.line(33, 32, 32, 37, 'h'); s.line(35, 32, 36, 36, 'g'); s.px(31, 37, 'h'); s.px(37, 36, 'g'); s.px(33, 37, 'h')
s.rect(15, 43, 8, 3, 'n'); s.rect(25, 43, 8, 3, 'n'); s.rect(15, 43, 8, 1, 'm'); s.rect(25, 43, 8, 1, 'm')
out['keeper'] = s.out()

# ---- the reader: tall and narrow, an ink-blue gown, round spectacles, a book under the arm
s = Spr({**SKIN, 'a': '#6f86b4', 'b': '#4c6190', 'c': '#2f3f66', 'f': '#f1ece0', 'i': '#cfc9bb', 'g': '#e3bf78', 'h': '#a8853e', 'n': '#2a2420', 'r': '#a8483c', 's': '#7a2f28', 't': '#d9d4c7', 'x': '#9aa0aa'})
s.poly([(17, 17), (31, 17), (35, 44), (13, 44)], 'abc')                 # gown, long
for y in range(20, 44): s.px(24, y, 'c')
s.rect(13, 43, 23, 1, 'c')
s.poly([(19, 17), (29, 17), (27, 22), (21, 22)], 'fii')                 # collar bands
s.px(24, 23, 'g'); s.px(24, 26, 'g'); s.px(24, 29, 'g')
s.ell(13.5, 24, 3.2, 7.4, 'abc'); s.ell(34.5, 24, 3.2, 7.4, 'bcc')      # long sleeves
s.rect(11, 31, 5, 2, 'f'); s.rect(32, 31, 5, 2, 'i'); s.rect(12, 33, 3, 2, 'k')
s.poly([(31, 25), (40, 24), (40, 35), (31, 36)], 'rss')                 # the book under the arm
s.rect(32, 26, 1, 9, 't'); s.rect(34, 28, 4, 1, 'g'); s.rect(34, 30, 4, 1, 'g')
s.rect(33, 31, 4, 3, 'K')                                               # the hand on it
face(s, 24, 10.5, 4.8, 5.8)
s.rect(19, 9, 4, 4, 'x'); s.rect(25, 9, 4, 4, 'x'); s.rect(20, 10, 2, 2, 'f'); s.rect(26, 10, 2, 2, 'f')   # spectacles
s.px(21, 11, 'e'); s.px(27, 11, 'e'); s.rect(23, 10, 2, 1, 'x'); s.px(18, 10, 'x'); s.px(29, 10, 'x')
s.poly([(17, 6), (31, 6), (30, 2), (18, 2)], 'abc')                     # a flat scholar's cap
s.rect(15, 5, 18, 2, 'c'); s.line(31, 5, 34, 9, 'g'); s.rect(33, 9, 2, 3, 'g')   # and its tassel
s.ell(18, 8, 1.6, 2.4, 'i'); s.ell(30, 8, 1.6, 2.4, 'i')                # grey at the temples
s.rect(17, 44, 6, 3, 'n'); s.rect(25, 44, 6, 3, 'n')
out['reader'] = s.out()

sheet(out, sys.argv[1], scale=8, order=['hunter', 'teller', 'keeper', 'reader'])
