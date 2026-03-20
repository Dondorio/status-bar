return {
  --stylua: ignore
	Colour = {
		BLACK   = 0xFF000000,
		BLUE    = 0xFF0000FF,
		CYAN    = 0xFF00FFFF,
		GREEN   = 0xFF00FF00,
		MAGENTA = 0xFFFF00FF,
		RED     = 0xFFFF0000,
		WHITE   = 0xFFFFFFFF,
		YELLOW  = 0xFFFFFF00,
	},

  --stylua: ignore
	---@enum
	Anchor = {
		TOP    = 1,
		BOTTOM = 2,
		LEFT   = 4,
		RIGHT  = 8,
	},

	debug_print = function(data)
		local loop
		loop = function(o)
			if type(o) == "table" then
				local res = {}
				for k, v in pairs(o) do
					if type(k) == "number" then
						k = "[" .. k .. "]"
					end

					local pair = k .. " = " .. loop(v)
					table.insert(res, pair)
				end
				local str = table.concat(res, ", ")

				return "{ " .. str .. " }"
			elseif type(o) == "string" then
				local escaped = string.gsub(o, "\n", "\\n")

				return '"' .. escaped .. '"'
			end

			return tostring(o)
		end

		print(loop(data))
	end,
}
