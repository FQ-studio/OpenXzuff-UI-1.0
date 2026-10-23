-- OpenXzuff UI 1.0: small host-agnostic UI core.
-- The platform host supplies drawing functions and normalized pointer events.

local UI = {}
UI.__index = UI

local function is_number(value)
  return type(value) == "number" and value == value
    and value ~= math.huge and value ~= -math.huge
end

local function valid_rect(rect)
  return type(rect) == "table"
    and is_number(rect.x) and is_number(rect.y)
    and is_number(rect.w) and is_number(rect.h)
    and rect.w >= 0 and rect.h >= 0
end

function UI.new(platform)
  assert(type(platform) == "table", "platform API table is required")
  assert(type(platform.fill_rect) == "function", "platform.fill_rect is required")
  assert(type(platform.draw_text) == "function", "platform.draw_text is required")

  return setmetatable({
    platform = platform,
    widgets = {},
    focused = nil,
    pointer_down = false,
  }, UI)
end

function UI:add_button(id, rect, label, on_click, style)
  assert(type(id) == "string" and id ~= "", "button id must be a non-empty string")
  assert(valid_rect(rect), "button rect must contain finite x, y, w, h")
  assert(type(label) == "string", "button label must be a string")
  assert(on_click == nil or type(on_click) == "function", "on_click must be a function")
  assert(style == nil or type(style) == "table", "style must be a table")

  for _, widget in ipairs(self.widgets) do
    assert(widget.id ~= id, "duplicate widget id: " .. id)
  end

  local widget = {
    kind = "button",
    id = id,
    rect = { x = rect.x, y = rect.y, w = rect.w, h = rect.h },
    label = label,
    on_click = on_click,
    style = style or {},
    enabled = true,
  }
  self.widgets[#self.widgets + 1] = widget
  return widget
end

function UI:remove_widget(id)
  for i, widget in ipairs(self.widgets) do
    if widget.id == id then
      table.remove(self.widgets, i)
      if self.focused == widget then self.focused = nil end
      return true
    end
  end
  return false
end

local function contains(rect, x, y)
  return x >= rect.x and y >= rect.y
    and x < rect.x + rect.w and y < rect.y + rect.h
end

function UI:hit_test(x, y)
  if not is_number(x) or not is_number(y) then return nil end
  -- Later widgets are visually on top, so test them first.
  for i = #self.widgets, 1, -1 do
    local widget = self.widgets[i]
    if widget.enabled and contains(widget.rect, x, y) then
      return widget
    end
  end
  return nil
end

function UI:pointer_pressed(x, y, button)
  if button ~= nil and button ~= 1 then return false end
  local widget = self:hit_test(x, y)
  self.focused = widget
  self.pointer_down = widget ~= nil
  return widget ~= nil
end

function UI:pointer_released(x, y, button)
  if button ~= nil and button ~= 1 then return false end
  local was_down = self.pointer_down
  self.pointer_down = false
  local widget = self:hit_test(x, y)
  if was_down and widget and widget == self.focused and widget.on_click then
    widget.on_click(widget)
    return true
  end
  return false
end

function UI:draw()
  local p = self.platform
  for _, widget in ipairs(self.widgets) do
    local r, s = widget.rect, widget.style
    local bg = s.background or { 38, 48, 65, 255 }
    local fg = s.foreground or { 240, 245, 255, 255 }
    if not widget.enabled then
      bg = s.disabled_background or { 55, 59, 68, 255 }
      fg = s.disabled_foreground or { 155, 160, 170, 255 }
    elseif self.focused == widget and self.pointer_down then
      bg = s.pressed_background or { 20, 92, 150, 255 }
    end
    p.fill_rect(r.x, r.y, r.w, r.h, bg)
    p.draw_text(widget.label, r.x + 12, r.y + 10, fg)
  end
end

return UI
