-- OpenXzuff UI desktop shell example.
-- Requires the host API documented in docs/platform-api.md.

local UI = require("openxzuff.ui")

local Desktop = {}
Desktop.__index = Desktop

local COLORS = {
  background = { 15, 20, 31, 255 },
  panel = { 24, 31, 45, 255 },
  accent = { 20, 126, 214, 255 },
  text = { 238, 244, 255, 255 },
  muted = { 164, 180, 200, 255 },
}

function Desktop.new(platform, options)
  options = options or {}
  local width = options.width or 800
  local height = options.height or 600
  assert(type(width) == "number" and width > 0, "width must be positive")
  assert(type(height) == "number" and height > 0, "height must be positive")

  local self = setmetatable({
    platform = platform,
    width = width,
    height = height,
    menu_open = false,
    status = "OpenXzuff UI 1.0 · Lua UI foundation",
    ui = UI.new(platform),
  }, Desktop)

  self.ui:add_button("menu", { x = 12, y = 8, w = 108, h = 36 },
    "OpenXzuff", function()
      self.menu_open = not self.menu_open
    end, {
      background = COLORS.accent,
      pressed_background = { 15, 100, 176, 255 },
    })

  self.ui:add_button("about", { x = 136, y = 8, w = 86, h = 36 },
    "About", function()
      self.status = "OpenXzuff UI is open source; platform adapter is host supplied."
    end)

  return self
end

function Desktop:resize(width, height)
  assert(type(width) == "number" and width > 0, "width must be positive")
  assert(type(height) == "number" and height > 0, "height must be positive")
  self.width, self.height = width, height
end

function Desktop:pointer_pressed(x, y, button)
  if self.menu_open and button == 1 and x >= 16 and x < 256
      and y >= 56 and y < 208 then
    local choice = math.floor((y - 56) / 38)
    if choice == 0 then
      self.status = "OpenXzuff UI 1.0 · Lua-native UI core"
    elseif choice == 1 then
      self.status = "No application is bundled in this foundation build."
    elseif choice == 2 then
      self.status = "Kernel and framebuffer integration are not wired yet."
    elseif choice == 3 then
      self.menu_open = false
    end
    return true
  end
  return self.ui:pointer_pressed(x, y, button)
end

function Desktop:pointer_released(x, y, button)
  return self.ui:pointer_released(x, y, button)
end

function Desktop:draw()
  local p = self.platform
  p.fill_rect(0, 0, self.width, self.height, COLORS.background)

  -- A restrained desktop grid gives a useful test pattern without assets.
  for x = 0, self.width, 40 do
    p.fill_rect(x, 48, 1, math.max(0, self.height - 48), { 25, 33, 47, 255 })
  end
  for y = 48, self.height, 40 do
    p.fill_rect(0, y, self.width, 1, { 25, 33, 47, 255 })
  end

  p.fill_rect(0, 0, self.width, 52, COLORS.panel)
  self.ui:draw()

  p.draw_text("OpenXzuff UI", 24, 94, COLORS.text)
  p.draw_text("Lua-native desktop foundation", 24, 124, COLORS.muted)
  p.draw_text(self.status, 16, math.max(58, self.height - 28), COLORS.muted)

  if self.menu_open then
    p.fill_rect(16, 56, 240, 152, { 28, 38, 55, 255 })
    p.fill_rect(16, 56, 3, 152, COLORS.accent)
    local items = { "About OpenXzuff", "Applications", "Platform status", "Close menu" }
    for i, label in ipairs(items) do
      p.draw_text(label, 30, 66 + (i - 1) * 38, COLORS.text)
    end
  end
end

return Desktop
