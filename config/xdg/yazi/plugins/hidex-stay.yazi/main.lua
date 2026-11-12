--- hIDEx: "go to the parent folder", but not above the project folder (HIDEX_YAZI_ROOT, hidex-yazi): yazi runs
--- in a sandbox that sees only the project, and a folder above it is refused
local cwd = ya.sync(function()
	return tostring(cx.active.current.cwd)
end)

return {
	entry = function()
		if cwd() ~= os.getenv("HIDEX_YAZI_ROOT") then
			ya.emit("leave", {})
		end
	end,
}
