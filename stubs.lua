---@meta
---@diagnostic disable: unused-local
---@diagnostic disable: lowercase-global
--#selene: allow(unused_variable)

---@class Canvas
local Canvas = {}

---@param colour number
function Canvas:clear(colour) end

---@param px number
---@param py number
---@param sx number
---@param sy number
function Canvas:draw_rect(px, py, sx, sy) end

---@param px number
---@param py number
---@param radius number
function Canvas:draw_circle(px, py, radius) end

---@param fx number
---@param fy number
---@param tx number
---@param ty number
function Canvas:draw_line(fx, fy, tx, ty) end

---@param px number
---@param py number
---@param str string
function Canvas:draw_text(px, py, str) end

function Canvas:draw_path_stroke() end
function Canvas:draw_path_fill() end

---@param px number
---@param py number
function Canvas:path_begin_from(px, py) end

---@param cp1x number
---@param cp1y number
---@param cp2x number
---@param cp2y number
---@param px number
---@param py number
function Canvas:path_bezier_curve_to(cp1x, cp1y, cp2x, cp2y, px, py) end

---@param colour number
function Canvas:set_paint_colour(colour) end

---@param style "fill" | "stroke"
function Canvas:set_paint_style(style) end

---@param width number
function Canvas:set_stroke_width(width) end
