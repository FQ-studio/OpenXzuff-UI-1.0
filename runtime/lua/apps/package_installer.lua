-- OpenXzuff graphical LAP/LUAP installer screen.
-- The host must supply choose_package() and install_package(path) for real installs.
local App = {}
App.__index = App
local C = {
  bg={15,20,31,255}, panel={24,31,45,255}, card={29,40,57,255},
  accent={20,126,214,255}, text={238,244,255,255}, muted={164,180,200,255},
  success={95,220,160,255}, warning={255,190,90,255},
}
function App.new(platform, options)
  assert(type(platform)=="table", "platform API is required")
  assert(type(platform.fill_rect)=="function", "platform.fill_rect is required")
  assert(type(platform.draw_text)=="function", "platform.draw_text is required")
  options=options or {}
  return setmetatable({platform=platform,width=options.width or 800,
    height=options.height or 600,package_path=nil,
    status="Choose a .lap (Luau) or .luap (Lua) package.",
    status_color=C.muted,pressed=nil}, App)
end
function App:resize(width,height)
  assert(type(width)=="number" and width>0, "width must be positive")
  assert(type(height)=="number" and height>0, "height must be positive")
  self.width,self.height=width,height
end
function App:_buttons()
  local margin=24
  local y=math.max(270,math.floor(self.height*0.60))
  local bw=math.min(220,math.floor((self.width-margin*3)/2))
  return {
    {id="browse",x=margin,y=y,w=bw,h=48,label="Choose package"},
    {id="install",x=margin*2+bw,y=y,w=bw,h=48,label="Install app"},
  }
end
function App:_hit(x,y)
  for _,b in ipairs(self:_buttons()) do
    if x>=b.x and x<b.x+b.w and y>=b.y and y<b.y+b.h then return b.id end
  end
end
function App:pointer_pressed(x,y,button)
  if button~=nil and button~=1 then return false end
  self.pressed=self:_hit(x,y)
  return self.pressed~=nil
end
function App:pointer_released(x,y,button)
  if button~=nil and button~=1 then return false end
  local was,now=self.pressed,self:_hit(x,y)
  self.pressed=nil
  if not was or was~=now then return false end
  if was=="browse" then return self:choose_package() end
  if was=="install" then return self:install_selected() end
  return false
end
function App:choose_package()
  if type(self.platform.choose_package)~="function" then
    self.status="File picker unavailable: host must implement choose_package()."
    self.status_color=C.warning
    return false
  end
  local ok,path=pcall(self.platform.choose_package,{".lap",".luap"})
  if not ok then
    self.status="File picker error: "..tostring(path); self.status_color=C.warning; return false
  end
  if type(path)~="string" or path=="" then
    self.status="No package selected."; self.status_color=C.muted; return false
  end
  local ext=path:match("(%.[^./\\]+)$")
  if ext~=".lap" and ext~=".luap" then
    self.status="Unsupported file. Select a .lap or .luap package."
    self.status_color=C.warning; return false
  end
  self.package_path=path
  self.status="Selected: "..path; self.status_color=C.success
  return true
end
function App:install_selected()
  if not self.package_path then
    self.status="Choose a package before installing."
    self.status_color=C.warning; return false
  end
  if type(self.platform.install_package)~="function" then
    self.status="Native installer service unavailable; package not installed."
    self.status_color=C.warning; return false
  end
  local ok,result,detail=pcall(self.platform.install_package,self.package_path)
  if not ok then
    self.status="Installer error: "..tostring(result); self.status_color=C.warning; return false
  end
  if result~=true then
    self.status="Install failed: "..tostring(detail or result or "unknown error")
    self.status_color=C.warning; return false
  end
  self.status="Installation completed: "..self.package_path
  self.status_color=C.success
  return true
end
function App:draw(width,height)
  if width and height then self:resize(width,height) end
  local p,w,h=self.platform,self.width,self.height
  p.fill_rect(0,0,w,h,C.bg)
  p.fill_rect(0,0,w,58,C.panel)
  p.fill_rect(0,58,4,h-58,C.accent)
  p.draw_text("OpenXzuff Package Installer",22,18,C.text)
  p.draw_text("Install Luau and Lua apps",24,88,C.accent)
  p.draw_text("Supported packages",24,126,C.text)
  p.fill_rect(24,150,math.max(220,w-48),62,C.card)
  p.draw_text(".lap  -  Luau App Package",38,163,C.text)
  p.draw_text(".luap  -  Lua App Package",38,187,C.muted)
  if self.package_path then
    p.draw_text("Selected package:",24,226,C.muted)
    p.draw_text(self.package_path,24,249,C.text)
  else
    p.draw_text("No package selected",24,236,C.muted)
  end
  for _,b in ipairs(self:_buttons()) do
    p.fill_rect(b.x,b.y,b.w,b.h,
      self.pressed==b.id and {15,100,176,255} or C.accent)
    p.draw_text(b.label,b.x+14,b.y+16,C.text)
  end
  p.fill_rect(24,h-74,math.max(220,w-48),48,C.panel)
  p.draw_text(self.status,36,h-57,self.status_color)
  p.draw_text("Only install packages you trust.",24,h-18,C.muted)
end
return App
