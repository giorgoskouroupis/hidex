# pudb on the terminal's own colours: "default" is its fg/bg and the names are its ANSI palette. Only the
# base styles are set; pudb derives the rest. Selection is reverse video.
D = "default"
rev = add_setting(D, "standout")
palette.update({
    "background": (D, D),
    "selectable": (D, D),
    "focused selectable": (rev, D),
    "highlighted": (add_setting(D, "bold"), D),
    "hotkey": (add_setting("dark blue", "underline"), D),
    "input": (D, D),
    "warning": (add_setting("dark red", "bold"), D),
    "header": (add_setting("dark blue", "bold"), D),
    "source": (D, D),
    "current source": (rev, D),
    "line number": ("dark gray", D),
    "breakpoint marker": (add_setting("dark red", "bold"), D),
    "keyword": ("dark magenta", D),
    "function": ("dark blue", D),
    "builtin": ("dark cyan", D),
    "string": ("dark green", D),
    "literal": ("brown", D),
    "comment": ("dark gray", D),
})
