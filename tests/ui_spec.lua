package.path = "src/?.lua;src/?/init.lua;" .. package.path

local UI = require("openxzuff.ui")
local Desktop = require("openxzuff.desktop")

local calls = { rects = {}, texts = {} }
local platform = {
  fill_rect = function(x, y, w, h, color)
    calls.rects[#calls.rects + 1] = { x=x, y=y, w=w, h=h, color=color }
  end,
  draw_text = function(text, x, y, color)
    calls.texts[#calls.texts + 1] = { text=text, x=x, y=y, color=color }
  end,
}

local function eq(actual, expected, message)
  assert(actual == expected, (message or "values differ") ..
    ": expected " .. tostring(expected) .. ", got " .. tostring(actual))
end

local ui = UI.new(platform)
local clicked = 0
ui:add_button("ok", { x=10, y=10, w=80, h=30 }, "OK", function()
  clicked = clicked + 1
end)

eq(ui:hit_test(10, 10).id, "ok", "left/top edge is inside")
eq(ui:hit_test(90, 20), nil, "right edge is outside")
eq(ui:hit_test(20, 40), nil, "bottom edge is outside")
eq(ui:pointer_pressed(20, 20, 1), true, "press inside")
eq(ui:pointer_released(20, 20, 1), true, "release inside triggers click")
eq(clicked, 1, "callback invoked once")
eq(ui:pointer_pressed(20, 20, 1), true, "second press")
eq(ui:pointer_released(200, 200, 1), false, "release outside does not click")
eq(clicked, 1, "outside release does not invoke callback")
eq(ui:remove_widget("ok"), true, "remove existing widget")
eq(ui:remove_widget("ok"), false, "missing widget returns false")

local duplicate_ok = pcall(function()
  ui:add_button("dup", { x=0, y=0, w=1, h=1 }, "A")
  ui:add_button("dup", { x=0, y=0, w=1, h=1 }, "B")
end)
eq(duplicate_ok, false, "duplicate IDs rejected")

local invalid_rect_ok = pcall(function()
  ui:add_button("bad", { x=0, y=0, w=-1, h=10 }, "bad")
end)
eq(invalid_rect_ok, false, "negative width rejected")

ui:add_button("under", { x=0, y=0, w=50, h=50 }, "Under")
ui:add_button("over", { x=10, y=10, w=50, h=50 }, "Over")
eq(ui:hit_test(20, 20).id, "over", "last added widget is on top")
ui:draw()
assert(#calls.rects > 0, "draw should call fill_rect")
assert(#calls.texts > 0, "draw should call draw_text")

local desktop = Desktop.new(platform, { width=320, height=240 })
desktop:draw()
assert(#calls.rects > 0, "desktop draw emits rectangles")
assert(#calls.texts > 0, "desktop draw emits text")
eq(desktop:pointer_pressed(20, 20, 1), true, "desktop menu button can be pressed")
eq(desktop:pointer_released(20, 20, 1), true, "desktop menu button can be clicked")
eq(desktop.menu_open, true, "menu toggles open")
desktop:pointer_pressed(40, 65, 1)
desktop:pointer_released(40, 65, 1)
eq(desktop.status, "OpenXzuff UI 1.0 · Lua-native UI core", "menu item updates status")

print("OpenXzuff UI tests passed")
