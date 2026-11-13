-- nvim -l nvim-parsers.lua NVIM_TREESITTER DIR LIST: nvim-treesitter's parsers and queries for the languages in
-- LIST (one a line, # comments) built into DIR (parser/, queries/), at the revisions nvim-treesitter pins. Needs
-- tree-sitter (the CLI), a C compiler, curl and tar; fails when a parser is missing afterwards.
local nts, dir, list = arg[1], arg[2], arg[3]
vim.opt.runtimepath:prepend(nts)
local langs = {}
for line in io.lines(list) do
  if line:match("^%a") then langs[#langs + 1] = line end
end
require("nvim-treesitter").setup({ install_dir = dir })
require("nvim-treesitter").install(langs):wait(900000)
local missing = vim.tbl_filter(function(l) return vim.uv.fs_stat(dir .. "/parser/" .. l .. ".so") == nil end, langs)
if #missing > 0 then
  io.stderr:write("parsers missing: " .. table.concat(missing, " ") .. "\n")
  os.exit(1)
end
