local widget = require("lua.widgets.init")
local extras = require("lua.extras")
local colour = extras.Colour

---@class Container: Widget
---@field [1] Widget[] Children
local Container = widget.widget:new()
Container.__index = Container

---@param children Widget[]?
---@param props WidgetProps?
function Container:new(o, children, props)
	o = o or {}
	setmetatable(o, self)
	o[1] = children or {}
	o[2] = props
	return o
end

---@param canvas Canvas
function Container:draw(canvas)
	canvas:set_paint_colour(colour.RED)
	canvas:draw_circle(300.0, 50.0, 20.0)

	for _, w in ipairs(self[1]) do
		w:draw(canvas)
	end
end

return Container
