# mode("#rrggbb"): "dark" or "light", by the colour's brightness (the usual weights of red, green and blue);
# the one rule for hidex-bg and hidex-theme's dark and light groups
function mode(hex,   r, g, b) {
  r = strtonum("0x" substr(hex, 2, 2)); g = strtonum("0x" substr(hex, 4, 2)); b = strtonum("0x" substr(hex, 6, 2))
  return (r * 299 + g * 587 + b * 114 < 128000) ? "dark" : "light"
}
