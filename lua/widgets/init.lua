---@class Widget
---@field [1] any Data
---@field [2]? WidgetProps Properties
local Widget = {}

function Widget:new(o)
	o = o or {}
	setmetatable(o, self)
	self.__index = self
	return o
end

---@param _ Canvas
function Widget:draw(_) end

---@class WidgetProps
---@field height? number
---@field width? number
---@field draw? function
---@field on_click? function
---@field on_mouse_enter? function
---@field on_mouse_leave? function
---@field on_mouse_move? function
local WidgetProps = {}

return {
	widget = Widget,
	widget_props = WidgetProps,
}
