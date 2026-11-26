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
        if s.get(x, y) == '.' and (x - 25.5 + 0.5) ** 2 + (y - 26.5 + 0.5) ** 2 < 2.8 ** 2 and x < 23: s.px(x, y, 'g')
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

# ==== the batch of 11-19 (evening): the fifteen that were still letters ====

# ---- lampmoth: a broad pale moth, wings open like a lampshade, a green lamp's light on it
s = Spr({'W': '#fbf4d2', 'w': '#e8d8a0', 'm': '#b9a66c', 'd': '#7d6c40', 'k': '#2a2416', 'G': '#9fd8a4', 'g': '#5fa870', 'e': '#1a1612', 'f': '#f4ecd6', 'o': '#c9a060'})
s.poly([(15, 9), (2, 4), (0, 15), (5, 22), (15, 19)], 'Wwm'); s.poly([(17, 9), (30, 4), (31, 15), (27, 22), (17, 19)], 'wmm')
s.poly([(15, 18), (6, 22), (8, 29), (15, 25)], 'Wwm'); s.poly([(17, 18), (26, 22), (24, 29), (17, 25)], 'wmd')
s.ell(7, 12, 2.6, 2.6, 'Ggg'); s.px(7, 12, 'k'); s.ell(25, 12, 2.6, 2.6, 'Ggg'); s.px(25, 12, 'k')      # eyespots, lamp green
s.line(2, 6, 12, 10, 'm'); s.line(30, 6, 20, 10, 'd'); s.line(3, 16, 13, 15, 'm'); s.line(29, 16, 19, 15, 'd')
for x in (4, 9, 23, 27): s.px(x, 20, 'd')
s.ell(16, 16, 2.6, 8.5, 'fwm')                                                                           # the furred body
s.ell(16, 8, 2.8, 2.6, 'fwm'); s.px(15, 8, 'e'); s.px(17, 8, 'e')
s.line(15, 6, 11, 1, 'd'); s.line(17, 6, 21, 1, 'd'); s.line(11, 1, 9, 2, 'o'); s.line(21, 1, 23, 2, 'o')
for y in (13, 16, 19, 22): s.line(14, y, 18, y, 'm')
out['lampmoth'] = s.out()

# ---- shusher: tall, grey, buttoned to the chin, one finger where a mouth would be
s = Spr({'H': '#b9b9cc', 'h': '#8a8aa0', 'd': '#5c5c72', 'k': '#23232e', 'f': '#e4dccf', 'F': '#c2b8a8', 'w': '#f3f0ea', 'b': '#3a3a4a', 'g': '#d1a04a'})
s.poly([(10, 13), (22, 13), (24, 30), (8, 30)], 'Hhd')
s.ell(16, 14, 6.6, 3, 'Hhd')
s.ell(16, 7.5, 5, 5.6, 'fFF')
s.rect(11, 2, 10, 3, 'k'); s.rect(10, 4, 12, 1, 'b'); s.ell(16, 2.5, 3, 2, 'k')                         # a tight bun of hair
s.px(14, 7, 'k'); s.px(18, 7, 'k'); s.line(13, 5, 15, 6, 'k'); s.line(19, 5, 17, 6, 'k')               # the brows
s.rect(15, 8, 2, 6, 'f'); s.px(15, 8, 'w'); s.rect(15, 13, 2, 2, 'F')                                   # the finger
s.ell(13.5, 17, 2.2, 4, 'Hhd'); s.rect(13, 13, 3, 3, 'h')
s.ell(23.5, 20, 2.2, 6, 'hdd'); s.rect(22, 25, 3, 2, 'F')
s.line(16, 16, 16, 29, 'd')
for y in (18, 21, 24, 27): s.px(17, y, 'g')
s.rect(11, 12, 10, 1, 'w')                                                                               # a white collar
s.rect(10, 30, 5, 2, 'b'); s.rect(17, 30, 5, 2, 'b')
out['shusher'] = s.out()

# ---- erratum: a slip of paper standing on its corner, a struck line and a corrected one, two ink eyes
s = Spr({'P': '#f8f1de', 'p': '#d0c0a8', 'q': '#a39478', 'k': '#221d16', 'r': '#b0442c', 'R': '#d96a4c', 'i': '#3d4a78', 's': '#6f6452'})
s.poly([(8, 3), (26, 6), (23, 27), (5, 24)], 'Ppq')
s.poly([(22, 5), (26, 6), (25, 10)], 'q'); s.line(22, 5, 25, 10, 's')                                   # a dog-ear
s.rect(11, 8, 3, 3, 'k'); s.rect(18, 9, 3, 3, 'k'); s.px(11, 8, 'P'); s.px(18, 9, 'P')
s.line(10, 14, 21, 15, 's'); s.line(9, 14, 22, 16, 'r'); s.line(9, 15, 22, 15, 'R')                     # struck through
s.line(9, 18, 20, 19, 'i'); s.line(9, 21, 17, 22, 'i')                                                  # what it should have said
s.px(22, 13, 'r'); s.line(22, 12, 24, 10, 'r')                                                          # a caret
s.line(9, 25, 6, 30, 'q'); s.line(18, 27, 21, 30, 'q'); s.rect(4, 30, 4, 1, 'k'); s.rect(20, 30, 4, 1, 'k')
s.line(5, 12, 1, 9, 'q'); s.line(25, 14, 30, 12, 'q'); s.px(0, 8, 'k'); s.px(31, 11, 'k')
for (x, y) in ((28, 22), (27, 26), (2, 18)): s.px(x, y, 'p'); s.px(x + 1, y + 1, 'q')
out['erratum'] = s.out()

# ---- redactor: a dark coat, a blind-slat band across the eyes, a blue pencil, three black bars in the air
s = Spr({'C': '#6f8fc4', 'c': '#5a78b0', 'd': '#34497a', 'k': '#14161f', 'n': '#23283a', 'f': '#d8ccb8', 'F': '#b3a690', 'B': '#8fb0f0', 'b': '#3d62a8', 'w': '#ece6d6'})
s.poly([(9, 13), (22, 13), (24, 30), (7, 30)], 'ndk')
s.ell(15.5, 14, 7, 3, 'ndk')
s.ell(15.5, 7.5, 5, 5.6, 'fFF')
s.rect(9, 5, 13, 4, 'k'); s.line(10, 6, 20, 6, 'c'); s.line(10, 8, 20, 8, 'd')                          # the band
s.line(13, 11, 18, 11, 'k')
s.rect(10, 1, 11, 3, 'n'); s.rect(8, 3, 15, 1, 'k')                                                     # a hat brim
s.poly([(13, 13), (18, 13), (15.5, 19)], 'w'); s.line(15, 14, 15, 18, 'c')                              # shirt and blue tie
s.ell(7.5, 19, 2.2, 6, 'ndk'); s.rect(6, 24, 3, 2, 'F')
s.ell(23.5, 16, 2.2, 4, 'dkk'); s.rect(23, 11, 3, 3, 'F')
s.line(25, 12, 30, 3, 'B'); s.line(26, 12, 31, 3, 'b'); s.px(30, 2, 'k')                                # the pencil, raised
for (x, y, w) in ((0, 8, 6), (0, 12, 5), (1, 16, 4)): s.rect(x, y, w, 2, 'k'); s.rect(x, y, w, 1, 'b') # struck lines
s.rect(9, 30, 5, 2, 'k'); s.rect(17, 30, 5, 2, 'k')
out['redactor'] = s.out()

# ---- starfish: five arms the orange of a dying coal, pale tube feet, a ring of small eyes
s = Spr({'O': '#f6b27a', 'o': '#d9824a', 'd': '#a8552a', 'k': '#3a1c10', 'y': '#ffe0a8', 'e': '#1a0e08', 'p': '#f4d2c0', 'r': '#7a3518'})
import math as _m
for i in range(5):
    a = _m.radians(-90 + i * 72); cx, cy = 16, 16.5
    tip = (cx + _m.cos(a) * 15, cy + _m.sin(a) * 15)
    l = (cx + _m.cos(a + 1.15) * 5.4, cy + _m.sin(a + 1.15) * 5.4); r = (cx + _m.cos(a - 1.15) * 5.4, cy + _m.sin(a - 1.15) * 5.4)
    s.poly([tip, l, (cx, cy), r], 'Ood')
    s.line(cx + _m.cos(a) * 4, cy + _m.sin(a) * 4, cx + _m.cos(a) * 13, cy + _m.sin(a) * 13, 'y')
    for t in (6, 9, 12): s.px(cx + _m.cos(a) * t + _m.cos(a + 1.57) * 1.6, cy + _m.sin(a) * t + _m.sin(a + 1.57) * 1.6, 'r')
    s.px(tip[0], tip[1], 'k')
s.ell(16, 16.5, 4.6, 4.4, 'Ood')
for i in range(5):
    a = _m.radians(-54 + i * 72); s.px(16 + _m.cos(a) * 2.6, 16.5 + _m.sin(a) * 2.6, 'e')
s.px(16, 16, 'y'); s.px(15, 17, 'k'); s.px(17, 17, 'k')
out['starfish'] = s.out()

# ---- anemone: a rose column on a stone, a crown of waving arms, a dark mouth
s = Spr({'P': '#f0a8c0', 'p': '#c46a8a', 'd': '#8a3f5c', 'k': '#3a1626', 'T': '#ffd9e6', 't': '#e58fb0', 'm': '#2a0e1a', 'g': '#8c8f8a', 'G': '#5d605c', 'y': '#fff0c0'})
s.ell(16, 29, 12, 3, 'gGG')
s.poly([(10, 14), (22, 14), (24, 28), (8, 28)], 'Ppd')
s.ell(16, 28, 8.4, 2.4, 'ppd')
for y in (17, 20, 23, 26): s.line(11, y, 21, y, 'd')
for i, (x, tx, ty) in enumerate(((9, 2, 6), (11, 6, 1), (13, 10, 4), (15, 14, 0), (17, 18, 3), (19, 23, 0), (21, 27, 3), (23, 30, 7))):
    s.line(x, 14, (x + tx) / 2 + (1 if i % 2 else -1), (14 + ty) / 2, 't'); s.line((x + tx) / 2 + (1 if i % 2 else -1), (14 + ty) / 2, tx, ty, 't')
    s.px(tx, ty, 'y'); s.line(x + 1, 14, (x + tx) / 2 + 1, (14 + ty) / 2 + 1, 'p')
s.ell(16, 14, 6.6, 2.6, 'Ttp'); s.ell(16, 14, 3, 1.3, 'm')
s.px(13, 19, 'k'); s.px(19, 19, 'k'); s.px(13, 18, 'T'); s.px(19, 18, 'T')
out['anemone'] = s.out()

# ---- axolotl: pink as a gum, smiling, three frills a side opening like a book
s = Spr({'P': '#f7c9d4', 'p': '#e3a3b4', 'd': '#b9788c', 'k': '#3a2028', 'R': '#e0607a', 'r': '#a83a55', 'e': '#1a1014', 'w': '#ffffff', 'b': '#8fc4d4', 'B': '#4f8a9e'})
s.ell(16, 29, 13, 2.6, 'bBB')
s.ell(17, 22, 9, 5, 'Ppd')                                                                               # the body
s.poly([(24, 20), (31, 16), (31, 22), (25, 25)], 'ppd'); s.line(25, 20, 31, 16, 'R')                    # the tail fin
s.ell(13, 13, 8.4, 6.4, 'Ppd')                                                                           # the head
for i, (dx, dy) in enumerate(((-3, -6), (-6, -3), (-7, 1))):
    s.line(6, 11 + i * 2, 6 + dx, 11 + i * 2 + dy, 'R'); s.line(7, 11 + i * 2, 7 + dx, 11 + i * 2 + dy, 'r'); s.px(6 + dx, 11 + i * 2 + dy, 'w')
for i, (dx, dy) in enumerate(((3, -6), (6, -3), (7, 1))):
    s.line(20, 9 + i * 2, 20 + dx, 9 + i * 2 + dy, 'R'); s.line(21, 9 + i * 2, 21 + dx, 9 + i * 2 + dy, 'r'); s.px(20 + dx, 9 + i * 2 + dy, 'w')
s.rect(8, 11, 2, 2, 'e'); s.rect(16, 11, 2, 2, 'e'); s.px(8, 11, 'w'); s.px(16, 11, 'w')
s.line(9, 16, 11, 17, 'k'); s.line(11, 17, 15, 17, 'k'); s.line(15, 17, 17, 16, 'k')                    # the smile
s.px(7, 14, 'R'); s.px(19, 14, 'R')
s.line(11, 26, 9, 29, 'd'); s.line(21, 26, 23, 29, 'd'); s.rect(7, 29, 3, 1, 'p'); s.rect(22, 29, 3, 1, 'p')
out['axolotl'] = s.out()

# ---- hermit: a crab at home in a brass inkwell, six legs, two eyes on stalks, ink at the brim
s = Spr({'B': '#f0d79a', 'b': '#c9a86a', 'o': '#8a6f3a', 'O': '#5c4a26', 'k': '#1c1610', 'i': '#2a2f55', 'I': '#4a55a0', 'C': '#d8a888', 'c': '#b08a6a', 'd': '#7a5a44', 'e': '#f6f6ee'})
s.poly([(9, 8), (23, 8), (26, 22), (6, 22)], 'Bbo')                                                      # the inkwell
s.ell(16, 8, 7.4, 2.6, 'bOO'); s.ell(16, 8, 5.4, 1.6, 'iIi'); s.rect(6, 21, 20, 2, 'O')
s.rect(12, 12, 8, 6, 'e'); s.rect(13, 13, 6, 1, 'o'); s.rect(13, 15, 4, 1, 'o'); s.rect(12, 12, 8, 1, 'b')   # its label
s.px(21, 10, 'i'); s.px(22, 12, 'i'); s.px(22, 13, 'I')                                                 # a run of ink
s.ell(16, 24, 7, 3, 'Ccd')                                                                               # what lives in it
s.line(13, 22, 12, 17, 'c'); s.line(19, 22, 20, 17, 'c'); s.rect(11, 15, 2, 2, 'e'); s.rect(20, 15, 2, 2, 'e'); s.px(12, 16, 'k'); s.px(20, 16, 'k')
for (x0, x1, y1) in ((10, 4, 29), (11, 7, 30), (12, 10, 31)): s.line(x0, 25, x1, y1 - 3, 'c'); s.line(x1, y1 - 3, x1 - 1, y1, 'd')
for (x0, x1, y1) in ((22, 28, 29), (21, 25, 30), (20, 22, 31)): s.line(x0, 25, x1, y1 - 3, 'c'); s.line(x1, y1 - 3, x1 + 1, y1, 'd')
s.ell(4.5, 21, 3, 2.4, 'Ccd'); s.line(2, 20, 5, 22, 'k'); s.ell(27.5, 22, 2.4, 2, 'cdd')               # the claws
out['hermit'] = s.out()

# ---- mudskipper: up out of the puddle on its elbows, eyes on top of its head, a sail of a fin
s = Spr({'M': '#b3a67e', 'm': '#8a7f5e', 'd': '#5c5440', 'k': '#221f16', 'e': '#f4f0dc', 'F': '#c9b06a', 'f': '#8a6f3a', 'u': '#4a4436', 'U': '#6a624e', 's': '#d8d0b0'})
s.ell(17, 29, 13, 2.6, 'Uuu'); s.rect(6, 28, 5, 1, 's'); s.rect(22, 30, 4, 1, 's')
s.poly([(8, 16), (22, 17), (30, 24), (28, 27), (14, 26), (7, 23)], 'Mmd')                                # the body, sloping back
s.poly([(27, 22), (31, 18), (31, 28)], 'mdd')
s.poly([(13, 16), (16, 7), (19, 9), (22, 8), (24, 17)], 'Ffd'); s.line(16, 8, 15, 16, 'f'); s.line(19, 10, 19, 16, 'f'); s.line(22, 9, 22, 16, 'f')   # the fin
s.ell(9, 15, 6.4, 5, 'Mmd')                                                                              # the head
s.ell(6.5, 9.5, 2.6, 2.6, 'Mmd'); s.ell(12, 9, 2.6, 2.6, 'Mmd')
s.rect(6, 9, 2, 2, 'e'); s.rect(12, 8, 2, 2, 'e'); s.px(6, 9, 'k'); s.px(12, 8, 'k')
s.line(3, 17, 12, 18, 'k'); s.px(3, 16, 'k')                                                             # a wide flat mouth
for (x, y) in ((16, 20), (20, 21), (24, 22), (18, 23)): s.px(x, y, 'd')
s.poly([(10, 21), (14, 22), (11, 28), (6, 28)], 'mdd'); s.rect(5, 28, 6, 1, 'k')                         # the elbow it walks on
s.poly([(18, 24), (22, 25), (21, 29), (16, 29)], 'mdd')
out['mudskipper'] = s.out()

# ---- newtling: small and wet-green, an orange belly, a head that looks with both sides
s = Spr({'G': '#a9d6b0', 'g': '#7fae8a', 'd': '#4f7a5e', 'k': '#16241c', 'O': '#f2a25a', 'o': '#c9742e', 'e': '#f6e7b8', 'w': '#dff4fa', 's': '#b08a6a', 'S': '#7a5a44'})
s.rect(2, 26, 28, 2, 's'); s.rect(2, 28, 28, 1, 'S'); s.rect(2, 26, 28, 1, 'w')                          # a wet shelf
s.ell(15, 21, 8, 4.4, 'Ggd'); s.ell(15, 23, 6, 2, 'Ooo')
s.poly([(21, 19), (28, 16), (31, 10), (29, 9), (26, 14), (21, 22)], 'ggd'); s.line(28, 15, 30, 10, 'G'); s.px(30, 8, 'd')   # the tail, curled up
s.ell(9, 14, 6, 4.6, 'Ggd')                                                                              # the head, turned
s.ell(4.5, 12, 2.2, 2.2, 'e'); s.px(4, 12, 'k'); s.ell(13.5, 12, 2.2, 2.2, 'e'); s.px(14, 12, 'k')
s.line(5, 17, 12, 17, 'k'); s.px(8, 15, 'd'); s.px(10, 15, 'd')
for x in (13, 16, 19): s.px(x, 18, 'd'); s.px(x + 1, 20, 'd')
s.line(10, 23, 7, 26, 'd'); s.line(19, 23, 22, 26, 'd'); s.rect(5, 25, 3, 1, 'g'); s.rect(21, 25, 3, 1, 'g')
s.px(5, 24, 'G'); s.px(23, 24, 'G')
s.px(26, 22, 'w'); s.px(27, 24, 'w'); s.px(3, 22, 'w')
out['newtling'] = s.out()

# ---- custodian: behind a counter, a ledger in one hand, and the other hand empty and nearer
s = Spr({'C': '#7c7c90', 'c': '#5a5a6a', 'd': '#3a3a48', 'k': '#16161c', 'f': '#ddd0bc', 'F': '#b8aa94', 'L': '#a8452f', 'l': '#7a2f20', 'P': '#f4ecd6', 'W': '#8a6f4a', 'w': '#5c4a30', 'g': '#d1a04a', 'e': '#f6f6ee'})
s.poly([(8, 12), (23, 12), (24, 24), (7, 24)], 'Ccd')
s.ell(15.5, 13, 7.6, 3, 'Ccd')
s.ell(15.5, 6.5, 5, 5.4, 'fFF')
s.rect(10, 1, 11, 3, 'd'); s.rect(9, 3, 13, 2, 'k'); s.rect(14, 1, 3, 2, 'g')                           # a peaked cap, a badge
s.rect(12, 6, 3, 2, 'e'); s.rect(16, 6, 3, 2, 'e'); s.px(13, 7, 'k'); s.px(17, 7, 'k'); s.line(12, 5, 19, 5, 'k'); s.px(15, 6, 'k')   # spectacles
s.line(13, 10, 18, 10, 'k')
s.line(15, 14, 15, 23, 'd'); s.px(16, 16, 'g'); s.px(16, 19, 'g')
s.ell(24.5, 17, 2.2, 5, 'cdd'); s.poly([(21, 15), (30, 13), (31, 22), (22, 23)], 'Lll'); s.rect(23, 16, 6, 4, 'P'); s.line(24, 17, 27, 17, 'l')   # the ledger
s.ell(5.5, 17, 2.4, 5, 'Ccd'); s.ell(4, 23, 3.6, 3, 'fFF'); s.line(1, 21, 1, 24, 'f'); s.line(3, 20, 3, 23, 'F'); s.line(5, 20, 5, 23, 'F')    # the empty hand, large
s.rect(0, 24, 32, 3, 'W'); s.rect(0, 24, 32, 1, 'g'); s.rect(0, 27, 32, 5, 'w')                          # the counter
for x in (5, 15, 25): s.line(x, 27, x, 31, 'k')
out['custodian'] = s.out()

# ---- dust devil: a column of grey coming up a stair, tight at the foot, two points of light in it
s = Spr({'D': '#d6cfbc', 'd': '#9a9280', 'g': '#6c6658', 'k': '#2e2b24', 'e': '#fff3c0', 'p': '#f4ecd6', 'l': '#b08a5a', 's': '#57524a', 'S': '#7c766a'})
for i, (y, w, off) in enumerate(((2, 11, 2), (5, 10, -2), (8, 9, 2), (11, 8, -1), (14, 7, 2), (17, 6, -1), (20, 5, 1), (23, 3.6, 0), (26, 2.4, 0))):
    s.ell(16 + off, y + 1.5, w, 2.6, 'Ddg')
    s.line(16 + off - w + 2, y + 2.6, 16 + off + w - 3, y + 3.4, 'g')
s.rect(13, 7, 2, 2, 'e'); s.rect(19, 8, 2, 2, 'e'); s.px(13, 7, 'p'); s.px(19, 8, 'p')
for (x, y, c) in ((3, 9, 'p'), (28, 6, 'l'), (27, 15, 'p'), (5, 17, 'l'), (25, 21, 'D'), (7, 23, 'D'), (29, 11, 'D'), (2, 3, 'D')): s.px(x, y, c)
s.rect(4, 12, 3, 2, 'p'); s.px(4, 12, 'D'); s.line(25, 2, 27, 4, 'l')                                   # a page and a twig, caught up
s.rect(8, 28, 16, 2, 'S'); s.rect(4, 30, 24, 2, 's'); s.rect(8, 28, 16, 1, 'D')                          # the stair
out['dust devil'] = s.out()

# ---- shelf mouse: looking up out of a glove it has been living in
s = Spr({'M': '#b9a892', 'm': '#8a7a6a', 'd': '#5c5046', 'k': '#1c1814', 'P': '#f0b8b8', 'p': '#c98a8a', 'e': '#14100c', 'w': '#ffffff', 'L': '#8a5a3a', 'l': '#6a4028', 'x': '#3e2416', 't': '#d9c8a0'})
s.poly([(6, 18), (26, 18), (27, 30), (5, 30)], 'Llx')                                                    # the glove's cuff
for i, x in enumerate((4, 9, 14, 19)): s.ell(x + 3, 26, 2.6, 5, 'Llx')
s.ell(27, 22, 2.6, 4.4, 'lxx')
s.line(7, 20, 25, 20, 't'); s.line(7, 22, 25, 22, 'x')
s.ell(16, 13, 7, 6, 'Mmd')                                                                               # the mouse
s.ell(9.5, 6.5, 3.6, 3.8, 'Mmd'); s.ell(9.5, 6.5, 2, 2.2, 'Ppp'); s.ell(22.5, 6.5, 3.6, 3.8, 'mdd'); s.ell(22.5, 6.5, 2, 2.2, 'ppp')
s.rect(12, 11, 2, 3, 'e'); s.rect(18, 11, 2, 3, 'e'); s.px(12, 11, 'w'); s.px(18, 11, 'w')
s.rect(15, 15, 2, 2, 'P'); s.line(14, 17, 18, 17, 'k'); s.px(16, 18, 'w')
s.line(13, 16, 7, 14, 'd'); s.line(13, 17, 7, 18, 'd'); s.line(19, 16, 25, 14, 'd'); s.line(19, 17, 25, 18, 'd')
s.rect(10, 18, 3, 2, 'P'); s.rect(19, 18, 3, 2, 'P')                                                    # paws on the cuff
s.line(27, 17, 30, 12, 'p'); s.line(30, 12, 29, 8, 'p')                                                 # a tail over the side
out['shelf mouse'] = s.out()

# ---- silverfish: a bright tapering thing between two cards, three tails, long feelers
s = Spr({'S': '#f2f6fa', 's': '#c0c8d0', 'd': '#8a94a0', 'k': '#2a3038', 'e': '#101418', 'C': '#f4ecd6', 'c': '#d6cab0', 'q': '#a39478', 'l': '#6f6452'})
s.poly([(1, 3), (14, 1), (15, 30), (2, 31)], 'Ccq'); s.poly([(18, 1), (31, 3), (30, 31), (17, 30)], 'ccq')   # two catalogue cards
for y in (6, 9, 12): s.line(3, y, 11, y - 1, 'l'); s.line(20, y - 1, 28, y, 'l')
s.ell(8, 26, 1.4, 1.4, 'l'); s.ell(24, 26, 1.4, 1.4, 'l')
s.rect(4, 16, 9, 8, 'C'); s.rect(20, 15, 8, 9, 'c')                                                      # blank where it was
for i, (y, w) in enumerate(((7, 3.4), (10, 4), (13, 3.8), (16, 3.4), (19, 2.8), (22, 2.2), (25, 1.6))):
    s.ell(16, y, w, 2, 'Ssd')
s.ell(16, 4.5, 2.8, 2.4, 'Ssd'); s.px(15, 4, 'e'); s.px(17, 4, 'e')
for y in (9, 12, 15, 18, 21): s.line(13, y + 1, 19, y + 1, 'd')
s.line(15, 3, 10, 0, 'k'); s.line(17, 3, 22, 0, 'k')
s.line(16, 26, 16, 31, 'd'); s.line(15, 26, 11, 31, 'd'); s.line(17, 26, 21, 31, 'd')
for y in (9, 13, 17): s.line(12, y, 10, y + 1, 'k'); s.line(20, y, 22, y + 1, 'k')
out['silverfish'] = s.out()

# ---- hand of Morphos: a pale hand out of a sleeve that is not there, finger and thumb about to pinch
s = Spr({'H': '#efe6f4', 'h': '#c9b7d9', 'd': '#8f7ba6', 'k': '#2a2036', 'n': '#f6d6dc', 'v': '#5a4a78', 'V': '#3a2e52', 's': '#fff6d6'})
for y in range(22, 32):                                                                                   # the sleeve, fading out
    for x in range(9, 24):
        if (x + y) % 2 == 0 and y < 29 or (y >= 29 and (x * 3 + y) % 4 == 0): s.px(x, y, 'v' if x < 17 else 'V')
s.rect(10, 22, 13, 2, 'v'); s.rect(10, 22, 13, 1, 'd')
s.poly([(11, 13), (22, 13), (22, 23), (11, 23)], 'Hhd')                                                  # the palm
s.poly([(11, 14), (8, 9), (10, 5), (13, 7), (14, 13)], 'Hhd'); s.px(10, 5, 'n'); s.px(11, 5, 'n')      # the thumb
s.poly([(14, 13), (13, 6), (14, 2), (17, 3), (17, 13)], 'Hhd'); s.rect(14, 2, 3, 1, 'n')               # the finger, bent to it
s.line(13, 7, 14, 3, 'd'); s.line(14, 9, 16, 9, 'd'); s.line(11, 10, 12, 11, 'd')
for i, x in enumerate((18, 21)): s.poly([(x - 1, 14), (x - 1, 9 + i), (x + 2, 9 + i), (x + 2, 14)], 'hdd'); s.rect(x - 1, 11 + i, 3, 3, 'h')   # two curled
s.rect(23, 12, 2, 4, 'd')
s.line(12, 17, 20, 18, 'd'); s.line(13, 20, 19, 20, 'd')
s.px(12, 4, 's'); s.px(11, 2, 's'); s.px(13, 1, 's'); s.px(9, 3, 's')                                   # a spark where they meet
out['hand'] = s.out()

# ==== the batch of 11-19 (night): the sixteen that were 16-pixel frames doubled ====

def fig(pal, wide=0, tall=0):
    """A standing figure: coat C/c/d, face f/F, dark k. Head at (16, 8), feet at 30."""
    s = Spr(pal)
    s.poly([(10 - wide, 13), (22 + wide, 13), (23 + wide, 29), (9 - wide, 29)], 'Ccd')
    s.ell(16, 14, 6.8 + wide, 3, 'Ccd')
    s.ell(16, 8 - tall, 4.8, 5.4, 'fFF')
    s.px(14, 8 - tall, 'k'); s.px(18, 8 - tall, 'k')
    s.ell(8.5 - wide, 19, 2.2, 5.5, 'Ccd'); s.ell(23.5 + wide, 19, 2.2, 5.5, 'cdd')
    s.rect(7 - wide, 24, 3, 2, 'F'); s.rect(22 + wide, 24, 3, 2, 'F')
    s.rect(10, 29, 5, 3, 'k'); s.rect(17, 29, 5, 3, 'k')
    return s

# ---- usher: a red tunic with gold frogging, a pillbox cap, one white glove showing you the way
s = fig({'C': '#b83a48', 'c': '#8b1f2d', 'd': '#5a1420', 'f': '#e9d5b8', 'F': '#c7b094', 'k': '#1a1612', 'G': '#f0d79a', 'g': '#d9b57a', 'w': '#f6f6f2', 'W': '#cfcfc8'})
s.rect(11, 1, 10, 3, 'c'); s.rect(11, 1, 10, 1, 'C'); s.rect(11, 3, 10, 1, 'g'); s.px(16, 0, 'g')
s.line(13, 11, 19, 11, 'k'); s.px(16, 10, 'F')
for y in (16, 19, 22, 25): s.line(13, y, 19, y, 'g'); s.px(13, y, 'G'); s.px(19, y, 'G')
s.line(16, 15, 16, 28, 'd')
s.rect(10, 13, 3, 2, 'g'); s.rect(19, 13, 3, 2, 'g')
s.ell(23.5, 19, 2.2, 5.5, '.'); s.rect(22, 24, 3, 2, '.')
s.poly([(22, 14), (29, 10), (30, 13), (24, 18)], 'Ccd'); s.rect(28, 7, 3, 4, 'w'); s.px(31, 6, 'w'); s.px(30, 6, 'W'); s.line(31, 5, 31, 3, 'w')   # the arm out, a finger
s.rect(7, 24, 3, 2, 'w')
out['usher'] = s.out()

# ---- censor: a black coat to the floor, a lamp-red stare, a rubber stamp the size of a hand
s = fig({'C': '#4a4a5c', 'c': '#2d2d3a', 'd': '#17171f', 'f': '#c9c0b0', 'F': '#a39a8a', 'k': '#0c0c12', 'r': '#ff5a5a', 'R': '#ffb3b3', 'g': '#8a8aa0', 'W': '#8a6f4a', 'w': '#5c4a30', 'i': '#a8452f'}, wide=1)
s.rect(11, 2, 10, 4, 'c'); s.rect(10, 5, 12, 1, 'd'); s.rect(11, 2, 10, 1, 'C')
s.rect(13, 7, 3, 2, 'r'); s.rect(17, 7, 3, 2, 'r'); s.px(13, 7, 'R'); s.px(17, 7, 'R')
s.line(14, 11, 18, 11, 'k')
s.line(16, 14, 16, 28, 'd'); s.px(12, 15, 'g'); s.px(20, 15, 'g')
for y in (17, 20, 23): s.px(17, y, 'g')
s.ell(24.5, 19, 2.2, 5.5, '.'); s.rect(23, 24, 3, 2, '.')
s.poly([(22, 14), (27, 15), (28, 20), (24, 21)], 'cdd')
s.rect(26, 16, 3, 5, 'W'); s.rect(24, 21, 7, 3, 'w'); s.rect(24, 21, 7, 1, 'W'); s.rect(24, 24, 7, 1, 'i')     # the stamp, inked
for (x, y) in ((25, 27), (28, 28), (30, 26)): s.px(x, y, 'i')
out['censor'] = s.out()

# ---- cataloguer: violet, a gold visor, a drawer of cards held open in both arms
s = fig({'C': '#6a5c82', 'c': '#4a3f5c', 'd': '#2b2436', 'f': '#d8ccb8', 'F': '#b3a690', 'k': '#1a1612', 'G': '#f0d79a', 'g': '#d9b57a', 'P': '#f6f0e0', 'p': '#d6cdb6', 'W': '#8a6f4a', 'w': '#5c4a30'})
s.rect(11, 3, 10, 2, 'g'); s.rect(10, 5, 12, 1, 'G'); s.ell(16, 3, 4, 2, 'c')
s.rect(13, 7, 3, 2, 'P'); s.rect(17, 7, 3, 2, 'P'); s.px(14, 8, 'k'); s.px(18, 8, 'k'); s.px(16, 7, 'g')       # spectacles
s.line(14, 11, 18, 11, 'k')
s.rect(6, 19, 20, 6, 'W'); s.rect(6, 19, 20, 1, 'g'); s.rect(6, 24, 20, 1, 'w'); s.rect(15, 21, 2, 2, 'g')     # the drawer
for x in range(8, 24, 2): s.rect(x, 16 - (x % 4 == 0), 2, 3 + (x % 4 == 0), 'P' if x % 4 else 'p'); s.px(x, 16 - (x % 4 == 0), 'p')
s.rect(12, 14, 2, 3, 'G')                                                                                      # a tab card
s.rect(5, 20, 3, 3, 'F'); s.rect(24, 20, 3, 3, 'F')
out['cataloguer'] = s.out()

# ---- engrosser: a clerk in olive black, a quill behind one ear, a page with one line on it
s = fig({'C': '#5a5846', 'c': '#3b3a2e', 'd': '#24231c', 'f': '#e4d8c4', 'F': '#bfb29c', 'k': '#1a1612', 'Q': '#f6f1e2', 'q': '#d6cfba', 'i': '#2a2f55', 'g': '#d9b57a', 'w': '#ece5d3'})
s.ell(16, 3.5, 5, 2.4, 'c'); s.rect(11, 4, 2, 4, 'c'); s.rect(19, 4, 2, 4, 'c')                                # hair, flat
s.line(21, 6, 27, 0, 'Q'); s.line(22, 6, 28, 1, 'q'); s.px(21, 7, 'k')                                         # the quill
s.line(14, 11, 18, 11, 'k')
s.poly([(13, 13), (19, 13), (16, 18)], 'w'); s.line(16, 14, 16, 28, 'd')
s.ell(8.5, 19, 2.2, 5.5, '.'); s.rect(7, 24, 3, 2, '.')
s.poly([(10, 14), (5, 17), (6, 21), (10, 19)], 'Ccd')
s.poly([(0, 14), (9, 13), (10, 27), (1, 28)], 'Qqq')                                                           # the page, held up
s.line(2, 19, 8, 18, 'i'); s.line(2, 20, 8, 19, 'i'); s.px(8, 21, 'i'); s.px(8, 23, 'i')                       # one line, large, still wet
s.rect(8, 20, 3, 2, 'F')
s.px(24, 26, 'i'); s.px(23, 24, 'F')
out['engrosser'] = s.out()

# ---- tapster: sleeves rolled, a long apron, a heavy glass and a cloth
s = fig({'C': '#8a5c40', 'c': '#5a3a2a', 'd': '#3a2418', 'f': '#e9d5b8', 'F': '#c7b094', 'k': '#1a1612', 'A': '#f6f0e0', 'a': '#d6cdb6', 'q': '#a89f8a', 'G': '#dff4fa', 'g': '#9fd3e6', 'B': '#4f8fa8', 'm': '#6a4a36'}, wide=1)
s.ell(16, 3.2, 4.6, 2, 'm'); s.rect(13, 10, 6, 2, 'm'); s.px(15, 12, 'm'); s.px(17, 12, 'm')                    # a moustache
s.poly([(12, 14), (20, 14), (21, 29), (11, 29)], 'Aaq'); s.line(12, 17, 20, 17, 'q'); s.rect(15, 20, 3, 3, 'a'); s.rect(15, 20, 3, 1, 'q')
s.line(12, 14, 10, 12, 'a'); s.line(20, 14, 22, 12, 'a')
s.rect(6, 22, 3, 3, 'F'); s.rect(23, 22, 3, 3, 'F')                                                            # bare forearms
s.rect(25, 17, 5, 7, 'g'); s.rect(25, 17, 5, 1, 'G'); s.rect(26, 18, 1, 5, 'G'); s.rect(25, 23, 5, 1, 'B'); s.rect(29, 18, 1, 5, 'B')   # the glass
s.poly([(2, 21), (8, 22), (7, 28), (3, 27)], 'Aaq'); s.px(4, 24, 'q'); s.px(6, 26, 'q')                        # the cloth
out['tapster'] = s.out()

# ---- shopkeep: a plum waistcoat over shirtsleeves, round spectacles, a feather duster, a ticket in the other hand
s = fig({'C': '#f1ece0', 'c': '#d6cdb6', 'd': '#a89f8a', 'f': '#e9d5b8', 'F': '#c7b094', 'k': '#1a1612', 'P': '#8a5a7a', 'p': '#6a3f5c', 'q': '#4a2a40', 'g': '#e3bf78', 'G': '#fff2c8', 'h': '#9a9488', 'w': '#b08a5a', 'r': '#c9824a', 'R': '#e8a86a', 't': '#f6f0e0'})
s.poly([(11, 14), (21, 14), (22, 28), (10, 28)], 'Ppq'); s.line(16, 14, 16, 28, 'q')                       # the waistcoat
for y in (17, 20, 23, 26): s.px(17, y, 'g')
s.line(12, 14, 15, 18, 'q'); s.line(20, 14, 17, 18, 'q'); s.px(16, 15, 't')
s.ell(16, 3.6, 4.8, 1.6, 'h'); s.px(11, 5, 'h'); s.px(21, 5, 'h')                                             # thin grey hair
s.rect(12, 7, 3, 3, 'g'); s.rect(17, 7, 3, 3, 'g'); s.px(13, 8, 'G'); s.px(18, 8, 'G'); s.px(15, 8, 'g'); s.px(16, 8, 'g')   # spectacles
s.px(13, 9, 'k'); s.px(18, 9, 'k'); s.line(14, 11, 18, 11, 'F')
s.line(24, 23, 28, 12, 'w'); s.ell(28.5, 9, 2.6, 3.6, 'Rrr'); s.px(27, 7, 'R'); s.px(30, 11, 'r'); s.px(29, 6, 'R')   # the feather duster
s.rect(4, 22, 4, 5, 't'); s.rect(5, 23, 2, 1, 'q'); s.rect(5, 25, 2, 1, 'q'); s.px(6, 21, 'h')                 # a ticket on its string
out['shopkeep'] = s.out()

# ---- boarder: one of the Kept House's own, in a dressing gown with a candle (each wears its own color, by tint)
s = fig({'C': '#f1ece0', 'c': '#d6cdb6', 'd': '#a89f8a', 'f': '#e9d5b8', 'F': '#c7b094', 'k': '#1a1612', 'G': '#e8e2d2', 'g': '#c9c2b0', 'q': '#8f8878', 'h': '#7a6a58', 'w': '#f6f0e0', 'y': '#fff2c8', 'o': '#ffcf7a', 'b': '#b08a5a'})
s.poly([(10, 14), (22, 14), (24, 29), (8, 29)], 'Ggq'); s.line(16, 14, 14, 29, 'q')                           # the gown, wrapped over
s.rect(9, 21, 14, 2, 'q'); s.px(15, 23, 'q'); s.px(15, 24, 'q'); s.px(17, 23, 'q')                             # the cord, tied
s.poly([(11, 14), (16, 18), (21, 14)], 'gqq')
s.ell(16, 3.8, 5.0, 2.0, 'h'); s.rect(11, 4, 1, 3, 'h'); s.rect(21, 4, 1, 3, 'h')                               # hair, flattened by a pillow
s.px(13, 9, 'k'); s.px(18, 9, 'k'); s.line(14, 11, 17, 11, 'F')
s.rect(5, 20, 2, 6, 'w'); s.px(5, 19, 'o'); s.px(6, 18, 'y'); s.px(5, 18, 'o'); s.px(6, 17, 'o'); s.rect(4, 26, 4, 1, 'b')   # the candle in its dish
s.rect(10, 30, 4, 1, 'q'); s.rect(18, 30, 4, 1, 'q')                                                           # slippers
out['boarder'] = s.out()

# ---- traveler: a long coat the color of the road, a hat pulled down, a pack and a stick
s = fig({'C': '#f1ece0', 'c': '#d6cdb6', 'd': '#a89f8a', 'f': '#e9d5b8', 'F': '#c7b094', 'k': '#1a1612', 'T': '#9a927e', 't': '#7a7362', 'u': '#565040', 'h': '#4a4234', 'H': '#6a604c', 'p': '#8a6a42', 'P': '#b08a5a', 'w': '#7a5c38', 'r': '#b5483c'})
s.poly([(10, 14), (22, 14), (25, 30), (7, 30)], 'Ttu'); s.line(16, 14, 16, 30, 'u')                           # the coat, to the boots
s.rect(8, 5, 16, 2, 'h'); s.ell(16, 3.6, 4.6, 2.6, 'Hhh'); s.rect(11, 6, 10, 1, 'u')                             # the hat, brim down
s.px(13, 9, 'k'); s.px(18, 9, 'k'); s.rect(12, 12, 8, 2, 'r'); s.px(20, 14, 'r'); s.px(21, 15, 'r')            # a scarf
s.ell(24, 17, 3.4, 4.4, 'Ppw'); s.line(21, 14, 23, 13, 'w'); s.rect(23, 16, 3, 1, 'w')                         # the pack, behind a shoulder
s.line(5, 30, 7, 10, 'w'); s.px(7, 9, 'w'); s.px(6, 20, 'P')                                                    # the stick
out['traveler'] = s.out()

# ---- sweater: stooped, a leather bag that is not empty, coin dust on its hands
s = fig({'C': '#a08a5a', 'c': '#7a6a4a', 'd': '#4a3e2a', 'f': '#d8c8a8', 'F': '#b3a384', 'k': '#1a1612', 'B': '#8a5c40', 'b': '#5a3a2a', 'x': '#3a2418', 'G': '#ffe9a8', 'g': '#ffcf7a', 'o': '#c98a2b'})
s.rect(10, 2, 12, 3, 'd'); s.rect(8, 5, 16, 1, 'k'); s.rect(10, 2, 12, 1, 'c')                                 # a soft cap pulled down
s.px(14, 8, 'k'); s.px(18, 8, 'k'); s.line(14, 11, 17, 12, 'k')
s.line(16, 15, 16, 28, 'd')
s.ell(23.5, 19, 2.2, 5.5, '.'); s.rect(22, 24, 3, 2, '.')
s.poly([(22, 14), (27, 16), (27, 19), (23, 19)], 'cdd')
s.ell(27, 24, 4.6, 5.4, 'Bbx'); s.poly([(24, 19), (30, 19), (28, 21), (26, 21)], 'bxx'); s.line(25, 20, 29, 20, 'G')    # the bag, tied
s.px(26, 24, 'g'); s.px(28, 26, 'g'); s.px(27, 22, 'G')
for (x, y) in ((30, 30), (25, 30), (28, 31), (22, 31)): s.px(x, y, 'g')                                        # what leaks out of it
s.rect(7, 24, 3, 2, 'g'); s.px(7, 24, 'G')
out['sweater'] = s.out()

# ---- assayer: grey, a skullcap, a loupe in one eye, the balance under its glass bell
s = fig({'C': '#a8a896', 'c': '#8a8a7a', 'd': '#5a5a4a', 'f': '#e4d8c4', 'F': '#bfb29c', 'k': '#1a1612', 'n': '#2a2a3a', 'G': '#eef8fa', 'g': '#cfe8ee', 'b': '#8fb8c4', 'Y': '#f0d79a', 'y': '#c9a86a', 'o': '#8a6f3a'})
s.ell(16, 4, 4.8, 2.6, 'n'); s.rect(11, 4, 10, 1, 'n')
s.ell(18.5, 8, 2, 2, 'y'); s.px(18, 8, 'G'); s.px(19, 8, 'b')                                                  # the loupe
s.line(14, 11, 17, 11, 'k')
s.line(16, 15, 16, 28, 'd'); s.rect(13, 13, 6, 1, 'G')
s.ell(8.5, 19, 2.2, 5.5, '.'); s.rect(7, 24, 3, 2, '.')
s.poly([(10, 14), (6, 17), (7, 20), (10, 19)], 'Ccd'); s.rect(6, 19, 3, 2, 'F')
s.rect(0, 28, 11, 2, 'o'); s.rect(0, 28, 11, 1, 'Y')                                                           # the base
s.line(5, 16, 5, 27, 'y'); s.line(1, 18, 9, 18, 'Y'); s.line(1, 18, 1, 22, 'o'); s.line(9, 18, 9, 23, 'o')
s.rect(0, 22, 3, 1, 'y'); s.rect(8, 23, 3, 1, 'y'); s.px(9, 22, 'Y')                                           # the pans, one lower
for y in range(13, 28):                                                                                         # the bell
    for x in range(0, 11):
        dx = (x - 5) / 5.6; edge = y == 13 and 2 <= x <= 8 or (abs(dx) > 0.86 and y > 14) or (y == 14 and (x in (1, 9)))
        if edge and s.get(x, y) == '.': s.px(x, y, 'g')
s.px(5, 12, 'G'); s.px(1, 16, 'G'); s.px(1, 17, 'G')
out['assayer'] = s.out()

# ---- warden: red and broad, a closed helm, a halberd, and it does not move from the plinth
s = fig({'C': '#b04040', 'c': '#8a2a2a', 'd': '#4a1414', 'f': '#3a3a44', 'F': '#2a2a33', 'k': '#101014', 'M': '#8a8a96', 'm': '#5a5a66', 'n': '#3a3a44', 'S': '#e4e4ea', 's': '#b0b0b8', 'W': '#8a6f4a', 'g': '#d9b57a', 'e': '#ffcf7a'}, wide=2)
s.ell(16, 7.5, 6, 6.4, 'Mmn'); s.rect(10, 7, 12, 2, 'k'); s.rect(15, 5, 2, 8, 'm'); s.px(13, 8, 'e'); s.px(19, 8, 'e')    # the helm, a slit, two lights
s.line(16, 0, 16, 2, 'c'); s.px(17, 0, 'C'); s.px(15, 1, 'C')
s.rect(8, 12, 16, 2, 'm'); s.rect(8, 12, 16, 1, 'M')                                                           # a gorget
s.rect(12, 16, 8, 8, 'd'); s.rect(15, 17, 2, 6, 'g'); s.rect(13, 19, 6, 2, 'g')                                # a tabard, a key on it
s.rect(5, 24, 3, 2, 'm'); s.rect(24, 24, 3, 2, 'm')
s.line(28, 2, 28, 31, 'W'); s.poly([(28, 1), (31, 3), (31, 8), (28, 10)], 'Sss'); s.poly([(28, 4), (25, 6), (28, 8)], 'sss'); s.px(28, 0, 'S')   # the halberd
s.rect(9, 29, 6, 3, 'n'); s.rect(17, 29, 6, 3, 'n')
out['warden'] = s.out()

# ---- tipstaff: navy, brass buttons, a tall hat, and the staff with its crown
s = fig({'C': '#34455c', 'c': '#1f2a3a', 'd': '#121a25', 'f': '#e9d5b8', 'F': '#c7b094', 'k': '#0c1118', 'G': '#f0d79a', 'g': '#d9b57a', 's': '#b08a3a', 'o': '#7a5c20', 'w': '#f2f2f2'}, tall=-1)
s.rect(12, 0, 8, 5, 'c'); s.rect(12, 0, 8, 1, 'C'); s.rect(10, 5, 12, 1, 'k'); s.rect(12, 4, 8, 1, 'g')        # the hat
s.line(14, 12, 18, 12, 'k'); s.rect(12, 10, 2, 3, 'F')                                                          # side whiskers
s.rect(19, 10, 2, 3, 'F')
s.line(16, 15, 16, 28, 'd')
for y in (16, 19, 22, 25): s.px(14, y, 'g'); s.px(18, y, 'g')
s.rect(13, 13, 6, 1, 'w'); s.rect(9, 20, 14, 1, 'k'); s.rect(15, 20, 2, 1, 'g')                                 # a belt
s.line(27, 5, 27, 31, 's'); s.line(28, 5, 28, 31, 'o'); s.rect(25, 1, 5, 4, 'g'); s.rect(25, 1, 5, 1, 'G'); s.px(25, 0, 'G'); s.px(27, 0, 'G'); s.px(29, 0, 'G')   # the staff
s.ell(23.5, 19, 2.2, 5.5, '.'); s.rect(22, 24, 3, 2, '.')
s.poly([(22, 14), (27, 16), (27, 19), (23, 19)], 'cdd'); s.rect(26, 17, 3, 3, 'F')
out['tipstaff'] = s.out()

# ---- footpad: a hood, a red kerchief to the eyes, a cosh, light on its feet
s = fig({'C': '#8a7654', 'c': '#6a5a40', 'd': '#3e3426', 'f': '#d8c0a0', 'F': '#b39c80', 'k': '#16120c', 'R': '#b84040', 'r': '#8a2a2a', 'W': '#5c4a30', 'w': '#3a2c1c', 'e': '#f6f0dc'})
s.ell(16, 7, 6.4, 6.6, 'Ccd'); s.ell(16, 8.5, 4, 4, 'fFF'); s.poly([(13, 1), (16, 0), (19, 2)], 'c')          # the hood
s.rect(12, 9, 8, 4, 'r'); s.rect(12, 9, 8, 1, 'R'); s.poly([(19, 12), (22, 15), (20, 15)], 'r')               # the kerchief
s.rect(13, 7, 2, 2, 'e'); s.rect(17, 7, 2, 2, 'e'); s.px(14, 8, 'k'); s.px(18, 8, 'k'); s.line(12, 6, 15, 6, 'k'); s.line(17, 6, 20, 6, 'k')
s.line(16, 15, 16, 22, 'd'); s.rect(9, 21, 14, 1, 'k')
s.ell(8.5, 19, 2.2, 5.5, '.'); s.rect(7, 24, 3, 2, '.')
s.poly([(10, 14), (5, 12), (4, 15), (9, 19)], 'Ccd'); s.rect(3, 11, 3, 3, 'F')
s.line(4, 11, 1, 2, 'W'); s.line(5, 11, 2, 2, 'w'); s.ell(1.5, 2.5, 2, 2.6, 'Www')                              # the cosh, up
s.rect(10, 29, 5, 3, '.'); s.rect(17, 29, 5, 3, '.')
s.poly([(10, 26), (15, 26), (13, 31), (8, 31)], 'cdd'); s.poly([(17, 26), (22, 26), (25, 30), (20, 31)], 'cdd')   # a crouch
s.rect(7, 30, 5, 2, 'k'); s.rect(21, 30, 5, 2, 'k')
out['footpad'] = s.out()

# ---- gardener: a straw hat wide as its shoulders, green smock, shears, the sun stuck behind it
s = fig({'C': '#8aa862', 'c': '#6a8a4a', 'd': '#3e5a2e', 'f': '#e4d8c4', 'F': '#bfb29c', 'k': '#1a1612', 'H': '#f0dc90', 'h': '#d8c070', 'o': '#a08a3a', 'S': '#e4e8ec', 's': '#b0b8c0', 'n': '#6f7780', 'g': '#4a7a3a', 'r': '#c96a6a', 'W': '#8a6f4a'})
s.ell(16, 5, 11, 2.6, 'Hho'); s.ell(16, 3, 5, 3, 'Hho'); s.rect(11, 4, 10, 1, 'g')                             # the hat
s.rect(10, 6, 12, 1, 'F')                                                                                       # its shadow on the face
s.line(14, 11, 18, 11, 'k')
s.rect(12, 16, 8, 9, 'd'); s.rect(13, 17, 6, 1, 'g'); s.rect(14, 21, 4, 3, 'c')                                 # an apron, a pocket
s.px(15, 20, 'r'); s.px(17, 20, 'g')
s.ell(23.5, 19, 2.2, 5.5, '.'); s.rect(22, 24, 3, 2, '.')
s.poly([(22, 14), (27, 12), (28, 15), (24, 19)], 'cdd'); s.rect(26, 10, 3, 3, 'F')
s.line(27, 10, 24, 1, 'S'); s.line(28, 10, 31, 1, 's'); s.line(25, 2, 26, 6, 'n'); s.px(27, 8, 'k'); s.ell(26, 12, 1.6, 1.6, 'W'); s.ell(29.5, 12, 1.6, 1.6, 'W')   # the shears, open
for (x, y) in ((3, 29), (5, 27), (4, 30), (28, 29), (30, 28)): s.px(x, y, 'g'); s.px(x, y - 1, 'C')            # the border it was at
s.px(5, 26, 'r'); s.px(30, 27, 'r')
out['gardener'] = s.out()

# ---- lamprey: an eel of the wet, a ring of teeth where a face would be
s = Spr({'A': '#86a894', 'a': '#5c7a6a', 'b': '#2f4a3d', 'k': '#12201a', 'e': '#f0e0a0', 'T': '#ffffff', 't': '#d8d8d8', 'm': '#3a1f28', 'M': '#6a2a3a', 'w': '#9fd3e6', 'W': '#4f8fa8'})
pts = [(11, 9, 6.4), (14, 14, 5.6), (18, 18, 5), (22, 21, 4.4), (25, 24, 3.6), (26.5, 27.5, 2.8), (25, 30, 2)]
for (x, y, r) in pts[::-1]: s.ell(x, y, r, r * 0.9, 'Aab')
s.ell(10, 9, 5.2, 5, 'Mmm'); s.ell(10, 9, 2.6, 2.4, 'k')                                                        # the mouth
import math as _m2
for i in range(10):
    a = i * _m2.pi / 5; s.px(10 + _m2.cos(a) * 4.2 - 0.5, 9 + _m2.sin(a) * 4 - 0.5, 'T'); s.px(10 + _m2.cos(a + 0.3) * 3 - 0.5, 9 + _m2.sin(a + 0.3) * 2.8 - 0.5, 't')
s.rect(15, 5, 2, 2, 'e'); s.px(15, 5, 'k')
for i in range(7): s.px(16 + i * 0.4, 10 + i * 0.9, 'k')                                                        # the gill holes
for (x, y) in ((19, 13), (22, 16), (25, 19), (28, 22)): s.px(x, y, 'A')
s.poly([(22, 15), (27, 14), (30, 20), (27, 21)], 'abb')                                                         # a fin
s.rect(2, 30, 20, 2, 'W'); s.rect(4, 30, 10, 1, 'w'); s.px(3, 27, 'w'); s.px(20, 28, 'w'); s.px(6, 20, 'w')
out['lamprey'] = s.out()

# ---- clipper: small, steel-grey, on the die racks, a pair of shears taller than it is
s = Spr({'C': '#b4bec8', 'c': '#8a949d', 'd': '#4f585f', 'k': '#1c2126', 'e': '#f0e0a0', 'S': '#f2f5f8', 's': '#c4ccd4', 'n': '#7d8690', 'g': '#ffe9a8', 'r': '#6a747d', 'W': '#5c4a30'})
s.ell(13, 21, 8, 7, 'Ccd')
s.ell(13, 12, 6, 5.4, 'Ccd')
s.rect(9, 10, 3, 3, 'e'); s.rect(14, 10, 3, 3, 'e'); s.px(10, 11, 'k'); s.px(15, 11, 'k')
s.line(11, 15, 15, 15, 'k')
s.poly([(8, 7), (6, 2), (10, 6)], 'cdd'); s.poly([(18, 7), (20, 2), (16, 6)], 'cdd')                              # two small horns of swarf
s.ell(5.5, 22, 2, 4, 'Ccd'); s.ell(20.5, 20, 2, 4, 'cdd'); s.rect(20, 17, 3, 3, 'c')
s.line(22, 18, 30, 1, 'S'); s.line(23, 18, 31, 2, 's'); s.line(22, 18, 24, 0, 's'); s.line(23, 18, 25, 0, 'n')  # the shears
s.px(23, 14, 'k'); s.ell(21, 21, 2, 2, 'W'); s.ell(25, 21, 2, 2, 'W'); s.px(21, 21, '.'); s.px(25, 21, '.')
s.rect(8, 28, 4, 3, 'd'); s.rect(15, 28, 4, 3, 'd'); s.rect(8, 30, 4, 1, 'k'); s.rect(15, 30, 4, 1, 'k')
for (x, y) in ((27, 26), (29, 29), (26, 30), (2, 29), (30, 24), (4, 27)): s.px(x, y, 'g')                       # filings
out['clipper'] = s.out()

# ---- sigh: grass that has gathered itself up, two pale eyes, a mouth about to say a name
s = Spr({'A': '#e6ecc4', 'a': '#c8d0a0', 'b': '#9aa878', 'd': '#6c7a52', 'k': '#3a4428', 'e': '#fbf8e6', 'g': '#7f9a5a', 'G': '#5a7240', 'w': '#f4f0d8'})
for (x, h, c) in ((2, 5, 'G'), (5, 8, 'g'), (8, 5, 'G'), (23, 6, 'g'), (26, 9, 'G'), (29, 5, 'g')): s.line(x, 31, x + (1 if x < 16 else -1), 31 - h, c)
s.poly([(9, 30), (11, 16), (21, 16), (23, 30)], 'Aab')
s.ell(16, 12, 7.4, 8, 'Aab')
for (x0, x1, y1) in ((10, 6, 3), (13, 11, 0), (16, 16, 1), (19, 21, 0), (22, 26, 4)): s.line(x0, 8, x1, y1, 'a'); s.px(x1, y1, 'A')   # it is made of blades
s.rect(12, 10, 3, 3, 'e'); s.rect(18, 10, 3, 3, 'e'); s.px(13, 11, 'k'); s.px(19, 11, 'k')
s.ell(16.5, 16.5, 1.6, 2, 'k')                                                                                  # the mouth
for y in (20, 24, 28): s.line(11, y, 14, y + 2, 'b'); s.line(18, y + 1, 21, y - 1, 'b')
s.line(9, 19, 4, 14, 'a'); s.line(23, 19, 28, 15, 'b'); s.px(3, 13, 'A'); s.px(29, 14, 'a')
for (x, y) in ((6, 9), (27, 8), (25, 4), (4, 20)): s.px(x, y, 'w')                                             # seed on the air
out['sigh'] = s.out()

# ---- longsight: a body on the hill, and a neck as long as the hill, looking at you from very close
s = Spr({'C': '#bccde0', 'c': '#9ab0c8', 'd': '#5a7088', 'k': '#1a2430', 'Y': '#fff3c0', 'y': '#f0e0a0', 'B': '#f0c670', 'b': '#d8b060', 'e': '#1a1612', 'g': '#7f9a5a', 'G': '#5a7240'})
s.ell(22, 30, 10, 2, 'gGG')
s.ell(22, 23, 7.4, 5.6, 'Ccd')                                                                                  # the body, far off
s.line(19, 28, 18, 31, 'd'); s.line(25, 28, 26, 31, 'd'); s.rect(16, 31, 3, 1, 'b'); s.rect(25, 31, 3, 1, 'b')
s.poly([(27, 22), (31, 19), (31, 25)], 'cdd')
for i in range(15):                                                                                             # the neck
    t = i / 14; x = 19 - t * 9 + _m.sin(t * 3.1) * 3; y = 19 - t * 12
    s.ell(x, y, 2.6 - t * 0.5, 2.2, 'Ccd')
s.ell(8, 6.5, 5.6, 5, 'Ccd')                                                                                    # the head, near
s.ell(6.5, 5.5, 3, 3, 'Yyy'); s.ell(6.5, 5.5, 1.4, 1.6, 'e'); s.px(6, 5, 'Y')                                   # one great eye
s.poly([(3, 8), (0, 10), (4, 11)], 'Bbb'); s.line(1, 10, 4, 10, 'k')
for (x, y) in ((13, 9), (15, 12), (17, 15)): s.px(x, y, 'd')
s.line(11, 2, 13, 0, 'c'); s.line(9, 2, 10, 0, 'c')
out['longsight'] = s.out()

# ---- picnicker: nobody, in a straw hat and white gloves, setting down the teapot
s = Spr({'H': '#f0d2a8', 'h': '#d8b890', 'b': '#8a6a4a', 'k': '#3a3a44', 'G': '#ffffff', 'g': '#f2ecdc', 'q': '#c9c2b0', 'T': '#c9eaf4', 't': '#9fd3e6', 'd': '#5f97ac', 'r': '#c96a6a', 'R': '#e8a0a0', 'w': '#f6f6f2', 's': '#fff6d6'})
s.ell(16, 6, 11.6, 2.8, 'Hhb'); s.ell(16, 3.5, 6, 3.4, 'Hhb'); s.rect(10, 5, 12, 1, 'r'); s.rect(21, 5, 3, 3, 'r'); s.px(22, 8, 'R')   # the hat, a ribbon
s.rect(12, 10, 2, 3, 'k'); s.rect(18, 10, 2, 3, 'k'); s.px(12, 10, 'G'); s.px(18, 10, 'G')                     # eyes, and nothing round them
s.line(14, 15, 18, 15, 'k'); s.px(13, 14, 'k'); s.px(19, 14, 'k')                                               # a polite smile
s.ell(5, 21, 3.6, 3, 'Ggq'); s.rect(4, 17, 3, 3, 'g'); s.line(2, 20, 2, 22, 'G')                                # the gloves
s.ell(27, 19, 3.4, 3, 'ggq'); s.rect(26, 21, 3, 2, 'q')
s.ell(17, 23, 7, 5.6, 'Ttd'); s.rect(13, 17, 8, 2, 't'); s.rect(15, 15, 4, 2, 'd'); s.px(16, 15, 'T')          # the teapot
s.poly([(9, 21), (4, 19), (5, 22), (10, 25)], 'Ttd')                                                            # its spout
s.line(24, 20, 27, 22, 'd'); s.line(27, 22, 24, 26, 'd')                                                        # its handle
s.px(14, 21, 'G'); s.px(14, 22, 'G'); s.rect(15, 24, 5, 1, 'r')
s.line(3, 16, 2, 13, 's'); s.line(5, 15, 6, 12, 's'); s.px(4, 11, 's')                                          # steam: it is still warm
s.rect(2, 29, 28, 3, 'w'); s.rect(2, 29, 28, 1, 'G')
for x in range(2, 30, 6): s.rect(x, 29, 3, 3, 'r'); s.rect(x, 29, 3, 1, 'R')                                    # the checked blanket
out['picnicker'] = s.out()

# ==== X6, 11-20: the Court's officers ====

# ---- bailiff: broad, slate blue, arms folded, an inkpot on a chain at the belt
s = fig({'C': '#6a7a9a', 'c': '#4a5a78', 'd': '#2a3448', 'f': '#d8c8b0', 'F': '#b3a48c', 'k': '#10141c', 'G': '#f0d79a', 'g': '#c9a86a', 'i': '#1c2040', 'I': '#3a4488', 'n': '#3a4458'}, wide=2)
s.rect(10, 1, 12, 3, 'd'); s.rect(9, 4, 14, 1, 'k'); s.rect(15, 1, 2, 2, 'g')                                   # a flat cap, a badge
s.line(13, 6, 15, 7, 'k'); s.line(19, 6, 17, 7, 'k'); s.line(14, 11, 18, 11, 'k'); s.rect(13, 12, 6, 1, 'F')   # brows, a set jaw
s.ell(6.5, 19, 2.2, 5.5, '.'); s.ell(25.5, 19, 2.2, 5.5, '.'); s.rect(5, 24, 3, 2, '.'); s.rect(24, 24, 3, 2, '.')
s.poly([(8, 15), (24, 15), (25, 20), (7, 20)], 'Ccd'); s.line(8, 17, 24, 17, 'd'); s.rect(7, 16, 3, 3, 'F'); s.rect(22, 18, 3, 2, 'F')   # the arms, folded
s.rect(8, 22, 16, 1, 'k'); s.rect(15, 22, 2, 1, 'g')
s.line(20, 23, 22, 26, 'g'); s.rect(21, 26, 4, 4, 'i'); s.rect(21, 26, 4, 1, 'g'); s.px(22, 28, 'I')            # the inkpot
s.px(13, 14, 'g'); s.px(19, 14, 'g')
out['bailiff'] = s.out()

# ---- clerk: behind a brass grille, spectacles, a stamp raised, sleeve garters
s = fig({'C': '#b8b09a', 'c': '#8f876f', 'd': '#5a5444', 'f': '#e4d8c4', 'F': '#bfb29c', 'k': '#1a1612', 'G': '#f0d79a', 'g': '#b08a3a', 'o': '#7a5c20', 'e': '#f6f6ee', 'W': '#8a6f4a', 'w': '#5c4a30', 'r': '#a8452f', 'v': '#3f8a56'})
s.ell(16, 3.4, 4.4, 1.8, 'd'); s.rect(11, 5, 10, 1, 'v'); s.rect(10, 6, 12, 1, 'v')                             # thin hair, an eyeshade
s.rect(12, 7, 3, 3, 'e'); s.rect(17, 7, 3, 3, 'e'); s.px(13, 8, 'k'); s.px(18, 8, 'k'); s.px(15, 8, 'g'); s.px(16, 8, 'g')
s.line(14, 12, 18, 12, 'k')
s.rect(13, 13, 6, 1, 'e'); s.line(16, 14, 16, 22, 'd'); s.rect(7, 17, 4, 1, 'r'); s.rect(21, 17, 4, 1, 'r')    # garters
s.ell(23.5, 19, 2.2, 5.5, '.'); s.rect(22, 24, 3, 2, '.')
s.poly([(22, 14), (27, 12), (28, 15), (24, 19)], 'cdd'); s.rect(26, 9, 3, 3, 'F')
s.rect(26, 4, 3, 5, 'W'); s.rect(24, 2, 7, 3, 'w'); s.rect(24, 2, 7, 1, 'W')                                    # the stamp, raised
s.rect(0, 22, 32, 2, 'g'); s.rect(0, 22, 32, 1, 'G'); s.rect(0, 24, 32, 8, 'W'); s.rect(0, 24, 32, 1, 'o')      # the counter
for x in range(1, 32, 4): s.line(x, 12, x, 21, 'g'); s.px(x, 12, 'G')                                           # the grille
s.rect(0, 11, 32, 1, 'g')
s.rect(12, 26, 8, 4, 'e'); s.line(13, 27, 18, 27, 'd'); s.line(13, 29, 16, 29, 'd')                             # a form, pushed under
out['clerk'] = s.out()

# ---- crier: a long coat with forms in it, a handbell, the List on a pole, mouth open
s = fig({'C': '#c8b080', 'c': '#a08a58', 'd': '#665634', 'f': '#e9d5b8', 'F': '#c7b094', 'k': '#1a1612', 'G': '#f0d79a', 'g': '#c9a030', 'o': '#8a6a18', 'P': '#f6f1e2', 'p': '#d6cfba', 'W': '#6a5236', 'r': '#b84040'})
s.rect(9, 3, 14, 2, 'd'); s.poly([(11, 3), (21, 3), (19, 0), (13, 0)], 'ddd'); s.rect(11, 2, 10, 1, 'r'); s.px(22, 1, 'P'); s.px(23, 0, 'P')   # a tricorn, a feather
s.ell(16, 11.5, 1.6, 1.8, 'k')                                                                                  # mouth open, mid-cry
s.poly([(10, 13), (14, 13), (12, 29), (9, 29)], 'cdd'); s.poly([(18, 13), (22, 13), (23, 29), (20, 29)], 'cdd') # the coat, open
for y in (17, 20, 23): s.rect(13, y, 2, 2, 'P'); s.px(13, y, 'p')                                               # forms in the lining
s.ell(8.5, 19, 2.2, 5.5, '.'); s.rect(7, 24, 3, 2, '.')
s.poly([(10, 14), (5, 16), (6, 20), (10, 19)], 'Ccd'); s.rect(4, 19, 3, 2, 'F')
s.line(5, 6, 5, 31, 'W'); s.poly([(0, 6), (10, 6), (10, 16), (0, 16)], 'Ppp'); s.rect(0, 6, 10, 1, 'W')         # the List
for y in (8, 10, 12, 14): s.line(1, y, 8, y, 'd')
s.ell(27, 23, 3, 3.4, 'Ggo'); s.rect(26, 18, 2, 3, 'W'); s.px(27, 27, 'o'); s.px(27, 28, 'k')                   # the handbell
s.px(31, 21, 'G'); s.px(30, 18, 'G'); s.px(31, 25, 'G')
out['crier'] = s.out()

# ---- the walk-ons (11-23): they were the last at 16 pixels ----
# ---- lamplighter: slate coat, a flat cap, a long pole with the lit wick at its top
s = fig({'C': '#6a7a8a', 'c': '#4a5a6a', 'd': '#2e3a46', 'f': '#e9d5b8', 'F': '#c7b094', 'k': '#1a1612', 'p': '#8a6a44', 'P': '#5c4630', 'g': '#ffcf7a', 'G': '#fff2c8', 'o': '#d9824a'})
s.rect(11, 2, 10, 2, 'd'); s.rect(10, 4, 13, 1, 'k'); s.rect(12, 1, 8, 1, 'c')                                # the cap
s.line(27, 6, 27, 31, 'p'); s.line(28, 6, 28, 31, 'P')                                                         # the pole
s.ell(23.5, 19, 2.2, 5.5, '.'); s.rect(22, 24, 3, 2, '.')
s.poly([(22, 14), (26, 16), (27, 20), (23, 19)], 'cdd'); s.rect(26, 18, 3, 2, 'F')                             # the arm out to it
s.ell(27.5, 3.5, 3.4, 3.6, 'o'); s.ell(27.5, 3.5, 2.4, 2.6, 'g'); s.ell(27.5, 3.8, 1.2, 1.4, 'G'); s.px(27, 0, 'g'); s.px(31, 6, 'g'); s.px(23, 1, 'g')
s.line(16, 15, 16, 28, 'd'); s.px(16, 18, 'g'); s.px(16, 22, 'g')
out['lamplighter'] = s.out()

# ---- runner: tan, mid-stride, a satchel with a letter in it, a scarf out behind
s = fig({'C': '#dbb884', 'c': '#c9a36a', 'd': '#8a6a3c', 'f': '#e9d5b8', 'F': '#c7b094', 'k': '#1a1612', 'b': '#6b4a2e', 'B': '#4a3220', 'w': '#f6f1e2', 'r': '#b84040', 'R': '#8a2a2a'})
s.rect(10, 29, 5, 3, '.'); s.rect(17, 29, 5, 3, '.')
s.poly([(10, 27), (14, 27), (9, 31), (5, 31)], 'ddd'); s.rect(4, 30, 5, 2, 'k')                                # the back leg, kicked out
s.poly([(18, 27), (22, 27), (25, 30), (21, 30)], 'ddd'); s.rect(22, 30, 5, 2, 'k')                             # the front leg, reaching
s.rect(11, 12, 10, 2, 'r'); s.line(11, 13, 4, 11, 'r'); s.line(10, 14, 3, 13, 'R'); s.px(2, 11, 'r')            # the scarf
s.line(11, 14, 22, 22, 'B')                                                                                     # the strap
s.rect(19, 21, 7, 6, 'b'); s.rect(19, 21, 7, 1, 'B'); s.rect(21, 19, 4, 3, 'w'); s.px(22, 20, 'r')              # the satchel, a letter
s.rect(12, 3, 8, 2, 'd'); s.rect(18, 4, 5, 1, 'd')                                                              # a cap, its peak forward
out['runner'] = s.out()

# ---- tinker: a pack frame over the shoulders, pots and pans swinging off it
s = fig({'C': '#b0a07e', 'c': '#9a8a6a', 'd': '#5e5238', 'f': '#d8c8a8', 'F': '#b3a384', 'k': '#1a1612', 'm': '#8e9aa6', 'M': '#c8d2dc', 'n': '#5a646e', 'p': '#8a6a44', 'P': '#5c4630', 'o': '#c9824a'})
s.rect(3, 1, 26, 2, 'p'); s.rect(3, 2, 26, 1, 'P'); s.rect(10, 3, 2, 10, 'P'); s.rect(20, 3, 2, 10, 'P')        # the frame
s.line(5, 3, 5, 7, 'n'); s.ell(5, 10, 3.4, 3.2, 'Mmn'); s.ell(5, 10, 1.6, 1.4, 'n')                             # a pan
s.line(27, 3, 27, 6, 'n'); s.rect(24, 6, 6, 5, 'm'); s.rect(24, 6, 6, 1, 'M'); s.rect(29, 7, 1, 4, 'n'); s.rect(23, 8, 1, 2, 'n')   # a pot
s.line(15, 3, 15, 4, 'n'); s.px(16, 4, 'o')
s.rect(12, 4, 8, 1, 'd'); s.rect(13, 3, 6, 1, 'd'); s.px(14, 8, 'k'); s.px(18, 8, 'k')                           # a soft hat
s.rect(12, 16, 8, 1, 'P'); s.line(13, 14, 19, 22, 'P'); s.line(19, 14, 13, 22, 'P')                              # the straps crossed
s.rect(25, 22, 3, 5, 'M'); s.rect(26, 20, 1, 2, 'n'); s.px(25, 22, 'm')                                          # a ladle in the hand
out['tinker'] = s.out()

# ---- census taker: grey, a wide dark hat, a board of names and a pencil over it
s = fig({'C': '#a8b2bc', 'c': '#8e9aa6', 'd': '#56606a', 'f': '#e9d5b8', 'F': '#c7b094', 'k': '#1a1612', 'h': '#3a3028', 'H': '#5a4c40', 'w': '#f6f1e2', 'W': '#d6cfba', 'y': '#e0b85a', 'b': '#8a6a44', 'i': '#7a6f5f'})
s.rect(7, 4, 18, 2, 'h'); s.rect(7, 4, 18, 1, 'H'); s.rect(11, 0, 10, 4, 'h'); s.rect(11, 3, 10, 1, 'y')        # the hat, a band
s.ell(23.5, 19, 2.2, 5.5, '.'); s.rect(22, 24, 3, 2, '.')
s.poly([(21, 15), (25, 17), (24, 22), (21, 21)], 'cdd')
s.rect(19, 16, 11, 12, 'b'); s.rect(20, 17, 9, 10, 'w'); s.rect(20, 26, 9, 1, 'W'); s.rect(22, 15, 5, 2, 'i')    # the board, its clip
for y in (19, 21, 23): s.line(21, y, 27, y, 'i')
s.px(21, 25, 'i'); s.px(22, 25, 'i')
s.rect(18, 22, 3, 2, 'F'); s.line(26, 12, 21, 21, 'y'); s.px(27, 11, 'h'); s.px(21, 22, 'k')                     # the pencil
s.rect(7, 24, 3, 2, 'F'); s.line(16, 15, 16, 28, 'd')
out['census taker'] = s.out()

# ---- glowworm: a pale grub, eleven rings of it, the last three lit
s = Spr({'G': '#f6ffb8', 'g': '#d8e08a', 'm': '#a8b45a', 'd': '#6a7a3a', 'k': '#2a3214', 'e': '#1a1612', 'y': '#ffffe6', 'w': '#f0f4d8'})
import math as _mg
pts = [(26 - t * 21 / 10.0, 22 - _mg.sin(t / 10.0 * _mg.pi) * 9 + (t > 7) * (t - 7) * 1.5) for t in range(11)]
for i, (x, y) in enumerate(pts): s.ell(x, y, 4.2 if i < 9 else 3.4, 4.0 if i < 9 else 3.2, 'k')
for i, (x, y) in enumerate(pts):
    lit = i <= 2
    s.ell(x, y, 3.3 if i < 9 else 2.5, 3.1 if i < 9 else 2.3, 'yGg' if lit else 'gmd')
for i, (x, y) in enumerate(pts[3:], 3): s.line(x - 1, y - 3, x - 1, y + 2, 'd')
hx, hy = pts[-1]; s.rect(hx - 2, hy - 1, 1, 2, 'e'); s.rect(hx + 1, hy - 1, 1, 2, 'e'); s.px(hx - 2, hy - 1, 'w'); s.px(hx - 2, hy - 4, 'd'); s.px(hx + 2, hy - 4, 'd'); s.px(hx, hy + 1, 'd')
for x, y in ((29, 13), (30, 18), (23, 12), (27, 10), (31, 24)): s.px(x, y, 'G')
out['glowworm'] = s.out()

# ---- mender: an apron with its pocket full, a needle as long as a forearm, red thread
s = fig({'C': '#cdb090', 'c': '#b89a7a', 'd': '#7a5e44', 'f': '#e9d5b8', 'F': '#c7b094', 'k': '#1a1612', 'a': '#ece5d3', 'A': '#c9c0b0', 'n': '#dfe6ea', 'N': '#8a929a', 't': '#d0735f', 'T': '#a8452f', 'h': '#6a5038'})
s.ell(16, 3.6, 5, 2.4, 'h'); s.rect(11, 4, 10, 2, 'h'); s.ell(16, 1.6, 2, 1.6, 'h')                              # hair up in a knot
s.poly([(12, 15), (20, 15), (21, 28), (11, 28)], 'aAA'); s.rect(13, 21, 6, 4, 'A'); s.rect(13, 21, 6, 1, 'a'); s.px(15, 20, 't'); s.px(17, 20, 'N')   # the apron, its pocket
s.rect(12, 13, 1, 3, 'A'); s.rect(19, 13, 1, 3, 'A')
s.line(26, 6, 24, 25, 'N'); s.line(27, 6, 25, 24, 'n'); s.px(26, 5, 'N'); s.px(27, 4, 'N')                       # the needle
s.line(27, 6, 30, 10, 't'); s.line(30, 10, 28, 15, 't'); s.line(28, 15, 31, 20, 'T'); s.line(31, 20, 29, 26, 't')
s.rect(8, 17, 3, 3, 't'); s.px(8, 17, 'T'); s.px(10, 19, 'T')                                                    # a patch on the sleeve
out['mender'] = s.out()

# ---- porter: bent a little under a trunk, a strap across the forehead
s = fig({'C': '#a08e6a', 'c': '#8a7a5a', 'd': '#5a4c34', 'f': '#d8c8a8', 'F': '#b3a384', 'k': '#1a1612', 'b': '#8a6240', 'B': '#6b4a2e', 'n': '#4a3220', 's': '#d9b57a', 'S': '#8a6a2f'})
s.rect(19, 2, 13, 11, 'n'); s.rect(20, 3, 11, 9, 'B'); s.rect(20, 3, 11, 2, 'b'); s.rect(20, 11, 11, 1, 'n')      # the trunk, on the shoulder
s.rect(22, 3, 2, 9, 'S'); s.rect(28, 3, 2, 9, 'S'); s.rect(22, 3, 2, 1, 's'); s.rect(28, 3, 2, 1, 's'); s.rect(24, 6, 3, 2, 's'); s.px(25, 7, 'k')
s.rect(11, 4, 9, 1, 'S'); s.rect(11, 3, 3, 1, 'S')                                                                                         # the strap
s.ell(23.5, 19, 2.2, 5.5, '.'); s.rect(22, 24, 3, 2, '.')
s.poly([(21, 15), (25, 16), (28, 14), (25, 13)], 'cdd'); s.rect(26, 12, 3, 2, 'F')                               # an arm up to steady it
s.line(16, 15, 16, 28, 'd'); s.rect(11, 20, 10, 1, 'd'); s.px(16, 20, 's')
out['porter'] = s.out()

order = ['shopkeep', 'boarder', 'traveler', 'citizen', 'docent', 'wisp', 'hollow', 'mothwing', 'inkling', 'ragpicker', 'revenant', 'sitter', 'drowned', 'hourhand', 'keeper', 'lanternfly', 'bookworm', 'proofreader', 'lampmoth', 'shusher', 'erratum', 'redactor', 'starfish', 'anemone', 'axolotl', 'hermit', 'mudskipper', 'newtling', 'custodian', 'dust devil', 'shelf mouse', 'silverfish', 'hand', 'usher', 'censor', 'cataloguer', 'engrosser', 'tapster', 'sweater', 'assayer', 'warden', 'tipstaff', 'footpad', 'gardener', 'lamprey', 'clipper', 'sigh', 'longsight', 'picnicker', 'bailiff', 'clerk', 'crier', 'lamplighter', 'runner', 'tinker', 'census taker', 'glowworm', 'mender', 'porter']
# ---- the frames: each drawing moved a little (animate.py says how, kind by kind)
from animate import animate, frames
for n in order: out[n]['a'] = animate(n, out[n]['r'])
json.dump(out, open(sys.argv[1] + '/atlas32.json', 'w'))
sheet(out, sys.argv[1] + '/sheet32.png', 6, order=order)
sheet(out, sys.argv[1] + '/sheet32-small.png', 2, bg=(92, 98, 104), order=order)
for n in order:
    assert all(len(r) == N for r in out[n]['r']) and len(out[n]['r']) == N, n
    used = set(''.join(out[n]['r'])) - {'.'}
    assert used <= set(out[n]['p']), (n, used - set(out[n]['p']))
print('ok', len(out))

# A sheet of every frame, a kind a row, to look the motion over.
def frame_sheet(path, names, scale=4):
    from PIL import Image
    most = max(len(frames(out[n])) for n in names)
    img = Image.new('RGB', (most * (N * scale + 6) + 6, len(names) * (N * scale + 6) + 6), (70, 76, 84))
    for j, n in enumerate(names):
        f = out[n]
        for k, rows in enumerate(frames(f)):
            for y in range(N):
                for x in range(N):
                    ch = rows[y][x]
                    if ch == '.': continue
                    c = f['p'][ch].lstrip('#'); rgb = tuple(int(c[i:i + 2], 16) for i in (0, 2, 4))
                    for dy in range(scale):
                        for dx in range(scale): img.putpixel((6 + k * (N * scale + 6) + x * scale + dx, 6 + j * (N * scale + 6) + y * scale + dy), rgb)
    img.save(path)
for i in range(0, len(order), 13): frame_sheet(sys.argv[1] + '/frames-%d.png' % (i // 13), order[i:i + 13])
print('frames', sum(len(out[n]['a']['f']) for n in order), 'bytes', len(json.dumps({n: out[n]['a'] for n in order})))
