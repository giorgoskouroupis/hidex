-- The language servers tIDEx's Neovim uses, one table: the ones hIDEx brings (Python, and the files most
-- projects have), and others found on the PATH when there (e.g. from the project's dev shell). init.lua enables
-- those found; editors/nvim reads it for /langs. Named after their programs, as Helix names them.
return {
  -- "standard", not basedpyright's default "recommended", which flags every unknown or partial type;
  -- hidex-pyright: basedpyright on the project's Python (hidex-python, /langs python)
  basedpyright = {
    cmd = { "hidex-pyright", "--stdio" },
    filetypes = { "python" },
    root_markers = { "pyproject.toml", "setup.py", "setup.cfg", "requirements.txt", ".git" },
    settings = { basedpyright = { analysis = { typeCheckingMode = "standard" } } },
  },
  ruff = { cmd = { "ruff", "server" }, filetypes = { "python" }, root_markers = { "pyproject.toml", "ruff.toml", ".git" } },
  ["bash-language-server"] = { cmd = { "bash-language-server", "start" }, filetypes = { "sh", "bash" }, root_markers = { ".git" } },
  ["markdown-oxide"] = { cmd = { "markdown-oxide" }, filetypes = { "markdown" }, root_markers = { ".git" } },
  taplo = { cmd = { "taplo", "lsp", "stdio" }, filetypes = { "toml" }, root_markers = { ".git" } },
  ["yaml-language-server"] = { cmd = { "yaml-language-server", "--stdio" }, filetypes = { "yaml" }, root_markers = { ".git" } },
  ["vscode-json-language-server"] = { cmd = { "vscode-json-language-server", "--stdio" }, filetypes = { "json", "jsonc" }, root_markers = { ".git" } },
  ["vscode-html-language-server"] = { cmd = { "vscode-html-language-server", "--stdio" }, filetypes = { "html" }, root_markers = { ".git" } },
  ["vscode-css-language-server"] = { cmd = { "vscode-css-language-server", "--stdio" }, filetypes = { "css", "scss", "less" }, root_markers = { ".git" } },
  ["typescript-language-server"] = {
    cmd = { "typescript-language-server", "--stdio" },
    filetypes = { "javascript", "javascriptreact", "typescript", "typescriptreact" },
    root_markers = { "package.json", "tsconfig.json", ".git" },
  },
  ["docker-langserver"] = { cmd = { "docker-langserver", "--stdio" }, filetypes = { "dockerfile" }, root_markers = { ".git" } },
  ["rust-analyzer"] = { cmd = { "rust-analyzer" }, filetypes = { "rust" }, root_markers = { "Cargo.toml", ".git" } },
  gopls = { cmd = { "gopls" }, filetypes = { "go", "gomod" }, root_markers = { "go.mod", ".git" } },
}
