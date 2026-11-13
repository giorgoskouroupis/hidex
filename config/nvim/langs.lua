-- nvim -l langs.lua LANG (editors/nvim servers): that language's servers in lua/tidex/servers.lua, "✓ found
-- ✘ missing", or "none". LANG as /langs names it (python, bash, javascript…), also Neovim's filetype.
local servers = dofile(vim.fs.dirname(arg[0]) .. "/lua/tidex/servers.lua")
local lang = ({ bash = "sh", jsx = "javascriptreact", tsx = "typescriptreact" })[arg[1]] or arg[1]
local out = {}
for name, config in vim.spairs(servers) do
  if vim.tbl_contains(config.filetypes, lang) or vim.tbl_contains(config.filetypes, arg[1]) then
    out[#out + 1] = (vim.fn.executable(config.cmd[1]) == 1 and "✓ " or "✘ ") .. name
  end
end
io.write(#out > 0 and table.concat(out, "  ") or "none")
