-- tIDEx's Neovim (editors/nvim run: nvim -u this file, NVIM_APPNAME=tidex-nvim, so your own Neovim config and
-- state are never read): plain Neovim, the language servers of lua/tidex/servers.lua, files kept in step with
-- the disk, the terminal's 16 colours (so it follows /theme).
local here = vim.fs.dirname(debug.getinfo(1, "S").source:sub(2))
vim.opt.runtimepath:prepend(here)

vim.g.mapleader = " "
local o = vim.opt
o.number = true
o.cursorline = true
o.signcolumn = "yes"
o.mouse = "a"
o.termguicolors = false
o.undofile = true
o.list = true
o.listchars = { tab = "→ ", trail = "·", nbsp = "␣" }
o.updatetime = 1000

-- Files changed on disk (Claude, git, a guest editor) come back by themselves: an untouched buffer is reloaded
-- within a second; one with changes asks (W12: keep yours, or load the disk's) when you come back to it or stop
-- typing, never in the middle of it. Writing over a file changed on disk asks too.
o.autoread = true
vim.uv.new_timer():start(1000, 1000, vim.schedule_wrap(function()
  if vim.fn.mode():match("^[cr]") or vim.fn.getcmdwintype() ~= "" then return end
  for _, b in ipairs(vim.api.nvim_list_bufs()) do
    if vim.api.nvim_buf_is_loaded(b) and vim.bo[b].buftype == "" and not vim.bo[b].modified
        and vim.api.nvim_buf_get_name(b) ~= "" then
      vim.cmd("silent! checktime " .. b)
    end
  end
end))
vim.api.nvim_create_autocmd({ "FocusGained", "BufEnter", "CursorHold" }, { command = "silent! checktime" })

-- Language servers: those found on the PATH; completion as you type; diagnostics in full on the cursor line
for name, config in pairs(require("tidex.servers")) do
  vim.lsp.config(name, config)
  if vim.fn.executable(config.cmd[1]) == 1 then vim.lsp.enable(name) end
end
o.completeopt = { "menuone", "noselect", "popup" }
vim.api.nvim_create_autocmd("LspAttach", {
  callback = function(args)
    local client = vim.lsp.get_client_by_id(args.data.client_id)
    if client and client:supports_method("textDocument/completion") then
      vim.lsp.completion.enable(true, client.id, args.buf, { autotrigger = true })
    end
  end,
})
vim.diagnostic.config({ virtual_text = { current_line = true }, severity_sort = true })

-- mini.nvim (tools/nvim-pack, shipped with tIDEx, pinned): pickers, git markers, and the keys that follow Space
-- or g shown as you type them. The keys are Helix's space menu, so both editors answer the same.
local pack = vim.fs.normalize(here .. "/../../../tools/nvim-pack")
vim.opt.packpath:prepend(pack)
-- the tree-sitter parsers and queries built for tIDEx (nvim-parsers.txt): colours and structure for those files
vim.opt.runtimepath:prepend(pack .. "/site")
vim.api.nvim_create_autocmd("FileType", { callback = function(args) pcall(vim.treesitter.start, args.buf) end })
if pcall(vim.cmd.packadd, "mini.nvim") then
  require("mini.pick").setup()
  require("mini.extra").setup()
  require("mini.diff").setup()
  local clue = require("mini.clue")
  clue.setup({
    triggers = { { mode = "n", keys = "<Leader>" }, { mode = "x", keys = "<Leader>" }, { mode = "n", keys = "g" } },
    clues = { clue.gen_clues.g() },
    window = { delay = 300 },
  })
  local pick, extra, lsp = MiniPick.builtin, MiniExtra.pickers, vim.lsp.buf
  local function key(keys, run, what) vim.keymap.set("n", keys, run, { desc = what }) end
  key("<Leader>f", pick.files, "files")
  key("<Leader>b", pick.buffers, "open files")
  key("<Leader>/", pick.grep_live, "search the project")
  key("<Leader>'", pick.resume, "the last list again")
  key("<Leader>s", function() extra.lsp({ scope = "document_symbol" }) end, "symbols here")
  key("<Leader>S", function() extra.lsp({ scope = "workspace_symbol" }) end, "symbols in the project")
  key("<Leader>d", function() extra.diagnostic({ scope = "current" }) end, "diagnostics here")
  key("<Leader>D", function() extra.diagnostic() end, "diagnostics in the project")
  key("<Leader>g", function() extra.git_files({ scope = "modified" }) end, "changed files")
  key("<Leader>k", lsp.hover, "documentation")
  key("<Leader>r", lsp.rename, "rename")
  key("<Leader>a", lsp.code_action, "code actions")
  vim.keymap.set("n", "<Leader>c", "gcc", { remap = true, desc = "comment the line" })
  key("<Leader>?", function() extra.keymaps() end, "every key")
  key("gd", lsp.definition, "definition")
  key("gr", function() extra.lsp({ scope = "references" }) end, "references")
  key("gi", lsp.implementation, "implementation")
end
