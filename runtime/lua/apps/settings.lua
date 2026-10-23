-- Minimal native settings app model for the Lua host.
local Settings = {}
Settings.__index = Settings

function Settings.new()
  return setmetatable({
    values = {
      theme = "Cyber Blue",
      animations = true,
      ai_enabled = false,
    },
  }, Settings)
end

function Settings:get(key)
  return self.values[key]
end

function Settings:set(key, value)
  if key == "theme" then
    assert(type(value) == "string" and #value > 0, "theme must be a non-empty string")
  elseif key == "animations" or key == "ai_enabled" then
    assert(type(value) == "boolean", key .. " must be boolean")
  else
    return nil, "unknown setting"
  end
  self.values[key] = value
  return true
end

return Settings
