# mode("#rrggbb"): "dark" or "light", by the colour's brightness (the usual weights of red, green and blue);
# the one rule for hidex-bg and hidex-theme's dark and light groups
function mode(hex,   r, g, b) {
  r = strtonum("0x" substr(hex, 2, 2)); g = strtonum("0x" substr(hex, 4, 2)); b = strtonum("0x" substr(hex, 6, 2))
  return (r * 299 + g * 587 + b * 114 < 128000) ? "dark" : "light"
}

# faint("#rrggbb", "dark"|"light"): a colour just off that background, for lines that should almost vanish
# (unfocused pane frames, hidex-config): 16% towards white on a dark one, towards black on a light one
function faint(hex, m,   i, c, out) {
  out = "#"
  for (i = 0; i < 3; i++) {
    c = strtonum("0x" substr(hex, 2 + 2 * i, 2))
    c = (m == "light") ? int(c * 84 / 100) : int(c + (255 - c) * 16 / 100)
    out = out sprintf("%02x", c)
  }
  return out
}
