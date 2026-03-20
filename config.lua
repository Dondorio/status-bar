---@module "stubs"
---@diagnostic disable: lowercase-global
--#selene: allow(unused_variable)

local bit = require("bit")
local extras = require("lua.extras")
local container = require("lua.widgets.container")
local colour = extras.Colour
local anchor = extras.Anchor

local rect_offset = 0

---@type Widget
local child = container:new({
	draw = function(self, canvas)
		canvas:set_paint_colour(colour.GREEN)
		canvas:draw_circle(300.0, 50.0, 10.0)
		extras.debug_print(self)
	end,
}, {}, {
	width = 10,
	height = 100,
	on_click = function(_, x, y)
		print(string.format("widget child clicked at local $d, %d\n", x, y))
		rect_offset = rect_offset - 20
	end,
})

local nested = container:new(nil, { child })

---@type Widget[]
local wid = {
	container:new(nil, { nested }, {
		width = 10,
		height = 100,
		on_click = function(_, x, y)
			print(string.format("widget clicked at local $d, %d\n", x, y))
			rect_offset = rect_offset - 20
		end,
	}),
}

---@param canvas Canvas
local smiley_face = function(canvas)
	canvas:set_paint_colour(colour.YELLOW)
	canvas:draw_circle(500.0, 50.0, 20.0)
	canvas:set_paint_colour(colour.BLACK)
	canvas:draw_line(495.0, 45.0, 495.0, 55.0)
	canvas:draw_line(505.0, 45.0, 505.0, 55.0)

	canvas:path_begin_from(495.0, 60.0)
	canvas:path_bezier_curve_to(498.0, 61.0, 502.0, 61.0, 505.0, 60.0)
	canvas:draw_path_stroke()
	canvas:set_paint_style("fill")
end

function opts()
	---@type WindowOpts
	return {
		width = 1000,
		height = 100,
		layer = "overlay",
		-- exclusive_zone = 100,
		-- margin = { top = 20 },
		-- namespace = "hello world",
		anchor = bit.bor(anchor.TOP),
		widgets = wid,
	}
end

---@param canvas Canvas
function draw(canvas)
	smiley_face(canvas)

	rect_offset = rect_offset + 1
	rect_offset = rect_offset % 500

	canvas:set_paint_colour(colour.BLUE)
	canvas:draw_rect(rect_offset, 50.0, 150.0, 20.0)

	for _, w in ipairs(wid) do
		w:draw(canvas)
	end
end
