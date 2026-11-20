import json, sys
sys.path.insert(0, sys.argv[1])
from paint import Spr, sheet, N
out = {}

# ---- citizen: a hooded pale robe, a dark hood-mouth with two eyes, a gold clasp
s = Spr({'h': '#f1ece0', 'c': '#d9d4c7', 'd': '#8e887c', 'x': '#6a655c', 'f': '#5b4a3c', 'e': '#1a1612', 'i': '#e9e1c8', 'g': '#d9b57a', 'o': '#f1dcb0', 'b': '#3a3028'})
s.ell(16, 20, 8.6, 9.4, 'hcd')            # robe
s.poly([(8, 22), (24, 22), (25, 29), (7, 29)], 'hcd')
s.ell(16, 10.5, 7.6, 7.6, 'hcd')          # hood
s.ell(16, 11.5, 4.8, 4.9, 'e')            # the hood's mouth
s.ell(16, 12.3, 4.2, 3.8, 'f')
s.rect(13, 11, 2, 2, 'e'); s.rect(18, 11, 2, 2, 'e')
s.px(13, 11, 'i'); s.px(18, 11, 'i')
s.ell(8.2, 21.5, 2.2, 4.6, 'hcd'); s.ell(23.8, 21.5, 2.2, 4.6, 'cdx')   # sleeves
s.rect(7, 25, 3, 2, 'c'); s.rect(22, 25, 3, 2, 'd')
for y in range(18, 28): s.px(16, y, 'd')  # the robe's seam
s.rect(15, 17, 3, 2, 'g'); s.px(15, 17, 'o')
for x in range(8, 25): s.px(x, 28, 'd')
s.rect(10, 29, 4, 2, 'b'); s.rect(18, 29, 4, 2, 'b'); s.rect(10, 29, 4, 1, 'f'); s.rect(18, 29, 4, 1, 'f')
out['citizen'] = s.out()

# ---- docent: gray-haired, long brown coat, a lamp held out
s = Spr({'s': '#f0e9dc', 'f': '#d9d4c7', 'q': '#b3ac9c', 'h': '#8e99a6', 'H': '#b6bfc9', 'k': '#1a1612', 'l': '#7a6350', 'c': '#5b4a3c', 'd': '#3e3128', 'b': '#2a211b', 'g': '#d9b57a', 'o': '#f6e7b8', 'y': '#fff6d6'})
s.poly([(9, 15), (23, 15), (25, 29), (7, 29)], 'lcd')   # coat
s.ell(16, 16.5, 7.4, 3.2, 'lcd')                         # shoulders
s.ell(16, 8.5, 5.6, 6.0, 'sfq')                          # head
s.ell(16, 5.2, 5.8, 3.0, 'Hhh'); s.rect(10, 6, 2, 4, 'h'); s.rect(20, 6, 2, 4, 'h')
s.rect(13, 9, 2, 2, 'k'); s.rect(18, 9, 2, 2, 'k'); s.rect(15, 12, 3, 1, 'q')
for y in range(17, 29): s.px(16, y, 'd')
s.px(16, 19, 'g'); s.px(16, 22, 'g'); s.px(16, 25, 'g')
s.rect(11, 15, 10, 1, 'b')                                # collar
s.ell(8, 20, 2.2, 5, 'lcd'); s.ell(24, 19, 2.2, 4, 'cdd')
s.rect(24, 22, 4, 2, 'd'); s.rect(27, 21, 2, 2, 'f')      # the arm out, the hand
s.rect(27, 23, 3, 1, 'b'); s.rect(26, 24, 5, 5, 'g'); s.rect(27, 25, 3, 3, 'o'); s.px(28, 26, 'y'); s.rect(26, 29, 5, 1, 'b')
s.rect(10, 29, 4, 2, 'b'); s.rect(18, 29, 4, 2, 'b')
out['docent'] = s.out()

# ---- wisp: a cold flame like a cut crystal, on two thin trailing threads
s = Spr({'w': '#ffffff', 'p': '#e9f6fb', 'l': '#9fd3e6', 'm': '#4f8fa8', 'k': '#24495a'})
s.poly([(16, 1), (22, 11), (23.5, 17), (16, 25), (8.5, 17), (10, 11)], 'k')
s.poly([(16, 2.5), (21, 11), (22.3, 17), (16, 23.5), (9.7, 17), (11, 11)], 'lmm')
s.poly([(16, 4), (19.5, 11), (20.5, 16.5), (16, 21.5), (11.5, 16.5), (12.5, 11)], 'pll')
s.poly([(15, 6), (17.5, 11), (18, 15.5), (15.5, 19), (13, 15.5), (13.5, 11)], 'wpp')
s.line(16, 3, 16, 22, 'p'); s.line(11, 12, 21, 12, 'l'); s.line(12, 12, 16, 22, 'l')
s.px(13, 8, 'w'); s.px(14, 7, 'w')
s.line(14, 24, 11, 30, 'm'); s.line(18, 24, 21, 30, 'm'); s.px(10, 31, 'k'); s.px(22, 31, 'k'); s.line(16, 24, 16, 28, 'k')
s.px(6, 9, 'p'); s.px(25, 6, 'l'); s.px(26, 20, 'p'); s.px(5, 21, 'l'); s.px(25, 7, 'p')
out['wisp'] = s.out()

# ---- hollow: a stooped green figure in black water, nothing where a face goes
s = Spr({'h': '#93b09c', 'g': '#6f8f7a', 'd': '#3f5a5a', 'w': '#2a3b3b', 'k': '#141c1c', 'e': '#0b0d12', 'r': '#4d6a6a', 'i': '#7fa9a9'})
s.ell(16, 19, 8.4, 9, 'hgd')
s.poly([(8, 20), (24, 20), (26, 27), (6, 27)], 'hgd')
s.ell(16, 8.5, 6.6, 7.2, 'hgd')
s.ell(16, 10, 4.2, 5.2, 'e'); s.ell(16, 12.5, 3.2, 2.2, 'k')
s.px(14, 9, 'w'); s.px(18, 9, 'w')
s.ell(7, 19, 2.4, 5.5, 'hgd'); s.ell(25, 19, 2.4, 5.5, 'gdd')
s.rect(5, 23, 3, 3, 'g'); s.rect(24, 23, 3, 3, 'd')
for y in range(17, 26, 2): s.px(13, y, 'd'); s.px(19, y + 1, 'd')
s.rect(2, 26, 28, 5, 'w'); s.rect(2, 26, 28, 1, 'r'); s.rect(4, 28, 5, 1, 'k'); s.rect(14, 29, 6, 1, 'k'); s.rect(23, 28, 4, 1, 'k')
s.px(9, 26, 'i'); s.px(10, 26, 'i'); s.px(21, 26, 'i'); s.px(22, 26, 'i'); s.px(3, 27, 'r'); s.px(28, 27, 'r')
out['hollow'] = s.out()

# ---- mothwing: wings spread, an eye on each, the body a dark bar
s = Spr({'s': '#f4efcf', 'w': '#d8d0a0', 'm': '#8a7a50', 'e': '#3a3020', 'o': '#c9a060', 'r': '#a8452f', 'k': '#241d12'})
s.poly([(15, 9), (4, 4), (0.5, 9), (1, 16), (7, 19), (15, 17)], 'swm')        # upper wing
s.poly([(15, 17), (7, 19), (3, 23), (6, 28), (12, 27), (15, 22)], 'wmm')      # lower wing
s.ell(7.5, 11.5, 3.2, 3.2, 'o'); s.ell(7.5, 11.5, 2, 2, 'e'); s.px(7, 11, 's')
s.ell(8.5, 22.5, 2, 2, 'o'); s.px(8, 22, 'e')
s.line(2, 9, 13, 10, 'm'); s.line(2, 15, 13, 15, 'm'); s.line(4, 5, 14, 9, 's')
s.line(5, 26, 13, 20, 'e'); s.px(1, 12, 'm'); s.px(1, 13, 'm')
s.mirror()
s.ell(16, 16, 2.1, 9, 'mee'); s.ell(16, 8.5, 2.6, 2.6, 'mee')
s.px(15, 8, 'r'); s.px(17, 8, 'r')
for y in (13, 16, 19, 22): s.rect(15, y, 3, 1, 'k')
s.line(15, 6, 11, 1, 'e'); s.line(17, 6, 21, 1, 'e'); s.px(10, 1, 'o'); s.px(22, 1, 'o'); s.px(11, 0, 'o'); s.px(21, 0, 'o')
out['mothwing'] = s.out()

# ---- inkling: a blot on four legs with one eye, holding a page over its head
s = Spr({'l': '#aeb2ea', 'i': '#7a7fd1', 'k': '#2a2d6b', 'n': '#16183d', 'w': '#f6f3ea', 'p': '#e8e4da', 'd': '#bfb9ab', 'e': '#ffffff'})
s.poly([(15, 2), (30, 1), (29, 12), (17, 14)], 'wpd')         # the page
for y, a, b in ((4, 18, 27), (6, 18, 26), (8, 18, 27), (10, 19, 24)): s.line(a, y, b, y - 0.3, 'd')
s.px(26, 10, 'k'); s.px(27, 10, 'k'); s.px(27, 11, 'k')        # a blot on it
s.line(13, 16, 18, 11, 'k'); s.line(14, 16, 19, 11, 'i')       # the arm that holds it
s.ell(11.5, 20, 8.2, 6.6, 'lik')
s.ell(9.5, 18.6, 3.3, 3.3, 'e'); s.ell(10, 19, 1.9, 1.9, 'n'); s.px(9, 18, 'e')
s.px(5, 16, 'l'); s.px(6, 15, 'l'); s.px(16, 23, 'k'); s.px(4, 22, 'k')
for x0, x1 in ((6, 3), (10, 9), (14, 16), (17, 21)):
    s.line(x0, 25, x1, 30, 'k'); s.line(x0 + 1, 25, x1 + 1, 30, 'i')
    s.px(x1, 31, 'n'); s.px(x1 + 1, 31, 'n')
s.px(2, 25, 'k'); s.px(22, 27, 'k'); s.px(1, 28, 'n')            # drips
out['inkling'] = s.out()

# ---- ragpicker: hooded, a sack over the shoulder, a hooked stick
s = Spr({'s': '#c9b990', 'r': '#a08a60', 'd': '#6e6250', 'k': '#2a2420', 'h': '#8a8a7a', 'e': '#e8d9a8', 'b': '#3d342b', 't': '#b9a36e', 'p': '#7a5a3a'})
s.line(4, 6, 4, 30, 'h'); s.line(5, 6, 5, 30, 'd'); s.line(4, 6, 8, 3, 'h'); s.line(8, 3, 9, 6, 'h')   # the stick
s.ell(24.5, 9, 6, 6.6, 'srd'); s.rect(21, 14, 3, 2, 'd'); s.line(22, 4, 26, 4, 'k'); s.px(27, 8, 'p'); s.rect(22, 9, 2, 2, 'p')  # the sack
s.poly([(9, 15), (22, 15), (24, 29), (7, 29)], 'trd')
s.ell(15, 10.5, 6.4, 6.8, 'trd')
s.ell(15, 12, 4, 4.4, 'k'); s.px(13, 12, 'e'); s.px(17, 12, 'e')
s.ell(7.5, 20, 2.4, 5, 'trd'); s.rect(4, 22, 4, 3, 'r')
s.ell(22.5, 17, 2.4, 4, 'rdd'); s.rect(21, 13, 3, 3, 'r')
for (x, y) in ((11, 20), (18, 23), (13, 26), (20, 19)): s.rect(x, y, 3, 3, 'd'); s.px(x, y, 'k'); s.px(x + 2, y + 2, 'k')    # patches
s.line(9, 29, 22, 29, 'd'); s.px(10, 30, 'r'); s.px(14, 30, 'r'); s.px(19, 30, 'r')
s.rect(10, 30, 4, 2, 'b'); s.rect(17, 30, 4, 2, 'b')
out['ragpicker'] = s.out()

# ---- revenant: broad, wrapped in grave-cloth, two red eyes
s = Spr({'h': '#e6dccb', 'b': '#cfc3b0', 's': '#8a7d6a', 't': '#6b5a4a', 'k': '#2b2420', 'r': '#c96a6a', 'R': '#f2a0a0', 'd': '#a89a84'})
s.poly([(7, 14), (25, 14), (26, 28), (6, 28)], 'hbs')
s.ell(16, 15, 9.6, 4, 'hbs')
s.ell(16, 7.5, 6.6, 6.4, 'hbs')
s.rect(11, 7, 10, 4, 't'); s.rect(12, 8, 3, 2, 'r'); s.rect(17, 8, 3, 2, 'r'); s.px(12, 8, 'R'); s.px(17, 8, 'R')
s.rect(14, 12, 4, 1, 'k'); s.px(15, 13, 'k'); s.px(17, 13, 'k')
for y in (3, 5, 16, 19, 22, 25): s.line(8 if y > 10 else 11, y + 1, 24 if y > 10 else 21, y, 'd')    # the wrappings
s.ell(4.5, 19, 2.6, 6.5, 'hbs'); s.ell(27.5, 19, 2.6, 6.5, 'bss')
s.rect(3, 25, 3, 3, 't'); s.rect(26, 25, 3, 3, 't')
s.line(5, 15, 5, 22, 'd'); s.line(27, 16, 27, 22, 't')
for y in (17, 21, 25): s.rect(15, y, 2, 2, 't'); s.px(15, y, 'k')
s.line(21, 26, 23, 30, 'b'); s.line(22, 26, 24, 30, 's')         # a loose end
s.rect(9, 28, 5, 3, 's'); s.rect(18, 28, 5, 3, 's'); s.rect(9, 30, 5, 1, 'k'); s.rect(18, 30, 5, 1, 'k')
out['revenant'] = s.out()

# ---- sitter: a gilt frame, one sitter in it and one already out of it
s = Spr({'L': '#f0d79a', 'l': '#c9a86a', 'g': '#8a6f3a', 'G': '#5c4a26', 'k': '#3a2a33', 'n': '#241a20', 'M': '#d3a1c4', 'm': '#b07aa0', 'u': '#7d5371', 'c': '#f3e9f3'})
s.rect(1, 2, 22, 27, 'g'); s.rect(1, 2, 22, 2, 'L'); s.rect(1, 2, 2, 27, 'l'); s.rect(21, 4, 2, 25, 'G'); s.rect(1, 27, 22, 2, 'G')
s.rect(3, 4, 18, 23, 'l'); s.rect(4, 5, 16, 21, 'k'); s.rect(4, 5, 16, 1, 'n'); s.rect(4, 5, 1, 21, 'n')
s.px(1, 2, 'c'); s.px(22, 2, 'L'); s.px(1, 28, 'g')
s.ell(10, 20, 5.6, 7, 'Mmu'); s.ell(10, 11.5, 4.2, 4.2, 'Mmu')         # the one who stays
s.rect(8, 11, 2, 2, 'c'); s.rect(11, 11, 2, 2, 'c')
s.rect(4, 26, 16, 1, 'k')
s.ell(23.5, 22.5, 5.4, 7.5, 'Mmu'); s.ell(23.5, 13.5, 4.3, 4.3, 'Mmu')   # the one who has stepped out
s.rect(21, 13, 2, 2, 'c'); s.rect(25, 13, 2, 2, 'c'); s.px(22, 14, 'n'); s.px(26, 14, 'n')
s.rect(18, 20, 3, 2, 'm'); s.rect(18, 22, 2, 2, 'M')                    # a hand still on the frame
s.rect(20, 29, 4, 2, 'u'); s.rect(25, 29, 4, 2, 'u')
out['sitter'] = s.out()

# ---- drowned: blue, upright in the water, weed on one shoulder
s = Spr({'h': '#8fc4d4', 'b': '#4f8a9e', 'd': '#2f4858', 'k': '#0e1518', 'g': '#6f8f7a', 'G': '#49685a', 'w': '#b9e2ee', 'e': '#dff4fa', 'n': '#1c2c36'})
s.poly([(9, 14), (23, 14), (24, 25), (8, 25)], 'hbd')
s.ell(16, 15, 7.6, 3.4, 'hbd')
s.ell(16, 8, 5.6, 6.2, 'hbd')
s.rect(13, 7, 2, 3, 'k'); s.rect(18, 7, 2, 3, 'k'); s.px(13, 7, 'e'); s.px(18, 7, 'e'); s.rect(14, 12, 4, 1, 'n')
s.line(16, 1, 16, 3, 'b'); s.px(17, 1, 'h')                                  # a curl of hair, afloat
s.ell(7, 19, 2.2, 5.5, 'hbd'); s.ell(25, 19, 2.2, 5.5, 'bdd')
s.line(21, 13, 24, 21, 'g'); s.line(22, 13, 25, 20, 'G'); s.px(20, 14, 'g'); s.px(23, 22, 'g'); s.px(11, 20, 'g'); s.px(12, 21, 'G')   # weed
s.px(18, 18, 'd'); s.px(13, 17, 'd'); s.px(15, 21, 'd')
s.rect(1, 24, 30, 2, 'h'); s.rect(1, 24, 30, 1, 'w'); s.rect(1, 26, 30, 5, 'd')
s.rect(4, 28, 6, 1, 'k'); s.rect(19, 28, 8, 1, 'k'); s.rect(12, 30, 5, 1, 'n')
s.px(6, 23, 'w'); s.px(26, 23, 'w'); s.px(27, 22, 'w'); s.px(3, 22, 'w')        # drops
out['drowned'] = s.out()

# ---- hourhand: a clock gone walking, its hands at ten to two
s = Spr({'F': '#fff7dc', 'f': '#f1e4c0', 'q': '#d8c796', 'B': '#f0c670', 'b': '#d1a04a', 's': '#8a6f3a', 'd': '#5a4420', 'k': '#2a2018', 'r': '#b0442c'})
s.ell(16, 14, 12.6, 12.6, 'Bbs')
for a in range(0, 360, 30):
    import math
    s.rect(16 + math.cos(math.radians(a)) * 12.4 - 1, 14 + math.sin(math.radians(a)) * 12.4 - 1, 2, 2, 'd')
s.ell(16, 14, 9.6, 9.6, 'Ffq')
for a in range(0, 360, 30):
    s.px(16 + math.cos(math.radians(a)) * 8.2 - 0.5, 14 + math.sin(math.radians(a)) * 8.2 - 0.5, 'd')
s.line(16, 14, 11, 8, 'k'); s.line(15, 14, 10, 8, 'k')       # the hour
s.line(16, 14, 22, 6, 'k')                                    # the minute
s.line(16, 14, 16, 20, 'r')
s.rect(15, 13, 2, 2, 'd')
s.rect(13, 0, 6, 2, 'b'); s.rect(15, 0, 2, 1, 'B')            # the winder
s.line(4, 16, 1, 20, 's'); s.line(27, 16, 30, 12, 's'); s.rect(0, 20, 2, 2, 'b'); s.rect(30, 10, 2, 2, 'b')
s.line(11, 26, 9, 30, 's'); s.line(21, 26, 23, 30, 's'); s.rect(7, 30, 4, 2, 'd'); s.rect(22, 30, 4, 2, 'd')
out['hourhand'] = s.out()

# ---- keeper: tall and pale, a dark band where the eyes go, a ring of keys
s = Spr({'W': '#f6f6fb', 'w': '#e0e0ee', 'g': '#8a8a9e', 'G': '#62627a', 'k': '#2b2b33', 'Y': '#f2cf7a', 'y': '#d1a04a', 'o': '#8a6f3a', 'e': '#c9c9de'})
s.poly([(9, 12), (21, 12), (23, 29), (7, 29)], 'Wwg')
s.ell(15, 13, 6.6, 3, 'Wwg')
s.ell(15, 6.5, 5, 5.6, 'Wwg')
s.rect(10, 5, 10, 3, 'k'); s.px(12, 6, 'e'); s.px(17, 6, 'e')
s.rect(11, 1, 8, 2, 'w'); s.rect(11, 1, 8, 1, 'W'); s.rect(10, 3, 10, 1, 'g')      # a flat cap
for y in range(14, 29, 3): s.line(10, y, 20, y, 'g')
s.line(15, 13, 15, 28, 'g')
s.ell(7, 18, 2.2, 5.5, 'Wwg'); s.ell(22.5, 18, 2.2, 5.5, 'wgG')
s.rect(22, 22, 3, 2, 'w')
s.ell(25.5, 26.5, 4.6, 4.6, 'y'); s.ell(25.5, 26.5, 2.8, 2.8, '.')
for y in range(N):
    for x in range(N):
        if s.g[y][x] == '.' and (x - 25.5 + 0.5) ** 2 + (y - 26.5 + 0.5) ** 2 < 2.8 ** 2 and x < 23: s.g[y][x] = 'g'
s.px(23, 23, 'Y'); s.px(24, 22, 'Y'); s.px(22, 25, 'Y')
s.line(28, 29, 30, 31, 'y'); s.px(31, 30, 'y'); s.line(25, 31, 25, 31, 'o'); s.line(21, 29, 20, 31, 'y'); s.px(19, 31, 'o')
s.rect(9, 29, 4, 2, 'G'); s.rect(17, 29, 4, 2, 'G'); s.rect(9, 30, 4, 1, 'k'); s.rect(17, 30, 4, 1, 'k')
out['keeper'] = s.out()

# ---- lanternfly: a small dark fly, glass wings, and a lit lamp where its tail should be
s = Spr({'W': '#eef4f2', 'w': '#b9cfd0', 'v': '#7f9aa0', 'k': '#1d1a16', 'b': '#4a3d2c', 'n': '#2e261c', 'Y': '#fff3c0', 'y': '#f4c95d', 'o': '#c98a2b', 'r': '#8a5a1e', 'e': '#f6e7b8'})
s.ell(8, 11, 6.5, 4.2, 'Wwv'); s.ell(24, 11, 6.5, 4.2, 'wvv')            # fore wings
s.ell(9.5, 17, 4.6, 3, 'Wwv'); s.ell(22.5, 17, 4.6, 3, 'wvv')            # hind wings
s.line(4, 11, 13, 12, 'v'); s.line(19, 12, 28, 11, 'v'); s.line(7, 17, 13, 16, 'v'); s.line(19, 16, 25, 17, 'v')
s.ell(16, 8.5, 3.4, 3.2, 'bnk')                                          # head
s.px(14, 8, 'e'); s.px(18, 8, 'e'); s.line(14, 5, 11, 2, 'k'); s.line(18, 5, 21, 2, 'k'); s.px(10, 2, 'b'); s.px(22, 2, 'b')
s.ell(16, 14, 3.2, 4, 'bnk')                                             # thorax
s.line(13, 13, 9, 15, 'k'); s.line(19, 13, 23, 15, 'k')
s.ell(16, 23, 5.2, 6.2, 'Yyo')                                           # the lamp
s.ell(16, 22, 2.4, 3, 'Y'); s.rect(13, 17, 6, 1, 'r'); s.rect(13, 28, 6, 1, 'r'); s.line(11, 19, 11, 27, 'r'); s.line(21, 19, 21, 27, 'r')
s.px(16, 30, 'o'); s.px(16, 31, 'r')
for (x, y) in ((8, 24), (24, 22), (6, 28), (26, 27)): s.px(x, y, 'y')    # light, loose
out['lanternfly'] = s.out()

# ---- bookworm: a fat green worm with spectacles, halfway out of the book it is eating
s = Spr({'G': '#c9e39a', 'g': '#8fbf5a', 'd': '#55803a', 'k': '#1f2a16', 'P': '#f4ecd6', 'p': '#d9cdb0', 'q': '#a89a7c', 'C': '#a8452f', 'c': '#7a2f20', 'x': '#4a1c14', 'e': '#f6f6ee', 'l': '#d1a04a'})
s.poly([(2, 22), (30, 22), (31, 30), (1, 30)], 'Ccx')                    # the cover
s.poly([(3, 19), (16, 21), (16, 28), (3, 27)], 'Ppq'); s.poly([(16, 21), (29, 19), (29, 27), (16, 28)], 'ppq')
for y in (22, 24, 26): s.line(5, y - 1, 14, y, 'q'); s.line(18, y, 27, y - 1, 'q')
s.line(16, 21, 16, 28, 'q')
s.ell(21, 22.5, 3, 2.4, 'k')                                             # the hole it came out of
for i, (x, y, r) in enumerate(((21, 20, 3.6), (20, 16.5, 3.8), (18, 13, 4), (16, 9.5, 4.2))):
    s.ell(x, y, r, r * 0.9, 'Ggd')
s.ell(14.5, 6.5, 4.8, 4.6, 'Ggd')                                        # head
s.line(17, 18, 23, 18, 'd'); s.line(15, 15, 21, 14, 'd'); s.line(13, 11, 20, 11, 'd')
s.rect(10, 5, 4, 3, 'l'); s.rect(15, 5, 4, 3, 'l'); s.rect(11, 6, 2, 1, 'e'); s.rect(16, 6, 2, 1, 'e'); s.px(14, 6, 'l'); s.px(12, 6, 'k'); s.px(17, 6, 'k')
s.line(12, 9, 16, 9, 'k'); s.px(11, 2, 'd'); s.px(17, 1, 'd'); s.line(12, 3, 12, 2, 'd'); s.line(16, 2, 17, 2, 'd')
for (x, y) in ((6, 17), (9, 15), (26, 16)): s.px(x, y, 'P'); s.px(x + 1, y + 1, 'p')   # crumbs of page
out['bookworm'] = s.out()

# ---- proofreader: thin, standing, a green eyeshade, a blue pencil as long as an arm, a galley trailing
s = Spr({'S': '#e9e2d0', 's': '#c8bea6', 't': '#8d846e', 'k': '#201c18', 'V': '#7fc48f', 'v': '#3f8a56', 'f': '#e6cdb0', 'F': '#c4a888', 'B': '#6f9ad8', 'b': '#3d62a8', 'P': '#f6f1e2', 'p': '#d6cfba', 'r': '#b0442c', 'w': '#3a3630', 'W': '#55504a'})
s.poly([(11, 13), (21, 13), (22, 28), (10, 28)], 'Wwk')                  # a long waistcoat
s.rect(14, 13, 4, 12, 'S'); s.line(16, 14, 16, 24, 's'); s.px(16, 16, 'k'); s.px(16, 19, 'k'); s.px(16, 22, 'k')
s.ell(16, 8, 4.4, 4.8, 'fFF')                                            # the face
s.rect(10, 5, 12, 2, 'V'); s.rect(9, 6, 14, 1, 'v'); s.rect(11, 3, 10, 2, 'k')   # the eyeshade
s.px(14, 8, 'k'); s.px(18, 8, 'k'); s.line(14, 11, 18, 11, 'k')
s.ell(8.5, 18, 1.8, 5.5, 'Wwk'); s.ell(23.5, 17, 1.8, 5, 'wkk')
s.rect(7, 22, 3, 2, 'f'); s.rect(23, 20, 3, 2, 'f')
s.line(25, 21, 30, 6, 'B'); s.line(26, 21, 31, 6, 'b'); s.px(30, 5, 'r'); s.px(31, 4, 'k'); s.px(25, 22, 'F')   # the pencil
s.poly([(2, 20), (9, 20), (9, 31), (2, 31)], 'Ppp')                      # the galley
for y in (22, 24, 26, 28): s.line(3, y, 7, y, 't')
s.line(4, 24, 8, 23, 'b'); s.px(8, 26, 'b'); s.px(3, 28, 'b')            # its marks
s.rect(11, 28, 4, 3, 'w'); s.rect(17, 28, 4, 3, 'w'); s.rect(11, 30, 4, 1, 'k'); s.rect(17, 30, 4, 1, 'k')
out['proofreader'] = s.out()

order = ['citizen', 'docent', 'wisp', 'hollow', 'mothwing', 'inkling', 'ragpicker', 'revenant', 'sitter', 'drowned', 'hourhand', 'keeper', 'lanternfly', 'bookworm', 'proofreader']
json.dump(out, open(sys.argv[1] + '/atlas32.json', 'w'))
sheet(out, sys.argv[1] + '/sheet32.png', 6, order=order)
sheet(out, sys.argv[1] + '/sheet32-small.png', 2, bg=(92, 98, 104), order=order)
for n in order:
    assert all(len(r) == N for r in out[n]['r']) and len(out[n]['r']) == N, n
    used = set(''.join(out[n]['r'])) - {'.'}
    assert used <= set(out[n]['p']), (n, used - set(out[n]['p']))
print('ok', len(out))
