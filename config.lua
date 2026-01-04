---@module "stubs"
---@diagnostic disable: lowercase-global
--#selene: allow(unused_variable)

local bit = require("bit")
local extras = require("extras")
local colour = extras.Colour
local anchor = extras.Anchor

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
	}
end

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

local x = 0

---@param canvas Canvas
function draw(canvas)
	smiley_face(canvas)

	x = x + 1
	x = x % 500

	canvas:set_paint_colour(colour.BLUE)
	canvas:draw_rect(x, 50.0, 150.0, 20.0)
end
