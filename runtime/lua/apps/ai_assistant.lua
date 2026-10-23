-- OpenXzuff AI Assistant app shell.
-- Inference is unavailable until the native AI service is implemented.
local App = {}
App.__index = App

function App.new(platform)
  assert(type(platform) == "table", "platform API is required")
  assert(type(platform.fill_rect) == "function", "platform.fill_rect is required")
  assert(type(platform.draw_text) == "function", "platform.draw_text is required")
  return setmetatable({
    platform = platform,
    prompt = "",
    answer = "AI runtime is not configured yet.",
    status = "Offline foundation · no model loaded",
  }, App)
end

function App:set_prompt(prompt)
  assert(type(prompt) == "string", "prompt must be a string")
  self.prompt = prompt
end

function App:submit()
  if self.prompt == "" then
    self.answer = "Enter a question first."
    return false
  end
  self.answer = "Inference is not available yet. No answer was generated."
  self.status = "Waiting for LiteRT backend and Qwen model integration"
  return false
end

function App:draw(width, height)
  local p = self.platform
  p.fill_rect(0, 0, width, height, { 15, 20, 31, 255 })
  p.fill_rect(0, 0, width, 52, { 24, 31, 45, 255 })
  p.draw_text("OpenXzuff AI Assistant", 18, 16, { 238, 244, 255, 255 })
  p.draw_text("Question:", 20, 82, { 20, 160, 255, 255 })
  p.draw_text(self.prompt ~= "" and self.prompt or "(no prompt)", 20, 110,
    { 238, 244, 255, 255 })
  p.draw_text("Response:", 20, 160, { 20, 160, 255, 255 })
  p.draw_text(self.answer, 20, 188, { 238, 244, 255, 255 })
  p.draw_text(self.status, 20, math.max(220, height - 30),
    { 164, 180, 200, 255 })
end

return App
