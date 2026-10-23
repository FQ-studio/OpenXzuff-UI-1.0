-- Xzuff App Hub GUI for the OpenXzuff Lua host.
-- Host callbacks: hub_list(query), hub_download(app_id), install_package(local_path),
-- and optionally choose_package()/hub_upload(local_path, metadata).
local Hub = {}
Hub.__index = Hub
local C = {
 bg={15,20,31,255}, panel={24,31,45,255}, card={29,40,57,255},
 accent={20,126,214,255}, text={238,244,255,255}, muted={164,180,200,255},
 good={95,220,160,255}, warn={255,190,90,255}
}
function Hub.new(platform, options)
 assert(type(platform)=="table","platform API is required")
 assert(type(platform.fill_rect)=="function","platform.fill_rect is required")
 assert(type(platform.draw_text)=="function","platform.draw_text is required")
 options=options or {}
 return setmetatable({platform=platform,width=options.width or 800,height=options.height or 600,
  apps={},selected=1,query="",status="App Hub is ready. Press Refresh to load catalog.",
  status_color=C.muted,pressed=nil},Hub)
end
function Hub:_buttons()
 local w=self.width
 return {
  {id="refresh",x=20,y=110,w=118,h=40,label="Refresh"},
  {id="download",x=148,y=110,w=140,h=40,label="Download"},
  {id="install",x=298,y=110,w=140,h=40,label="Install"},
  {id="upload",x=448,y=110,w=140,h=40,label="Upload app"},
 }
end
function Hub:_hit(x,y)
 for _,b in ipairs(self:_buttons()) do
  if x>=b.x and x<b.x+b.w and y>=b.y and y<b.y+b.h then return b.id end
 end
 if y>=166 and y<self.height-82 then
  local i=math.floor((y-166)/58)+1
  if i>=1 and i<=#self.apps then self.selected=i; return "row" end
 end
end
function Hub:pointer_pressed(x,y,button)
 if button~=nil and button~=1 then return false end
 self.pressed=self:_hit(x,y); return self.pressed~=nil
end
function Hub:pointer_released(x,y,button)
 if button~=nil and button~=1 then return false end
 local was,now=self.pressed,self:_hit(x,y); self.pressed=nil
 if was=="row" and now=="row" then return true end
 if not was or was~=now then return false end
 if was=="refresh" then return self:refresh() end
 if was=="download" then return self:download_selected(false) end
 if was=="install" then return self:download_selected(true) end
 if was=="upload" then return self:upload() end
 return false
end
function Hub:refresh()
 if type(self.platform.hub_list)~="function" then
  self.status="Hub network adapter unavailable; catalog not loaded."; self.status_color=C.warn; return false
 end
 local ok,result=pcall(self.platform.hub_list,self.query)
 if not ok or type(result)~="table" then
  self.status="Could not load catalog: "..tostring(result); self.status_color=C.warn; return false
 end
 self.apps=result; self.selected=math.min(self.selected,math.max(1,#self.apps))
 self.status="Loaded "..tostring(#self.apps).." app(s) from Xzuff App Hub."
 self.status_color=C.good; return true
end
function Hub:download_selected(install_after)
 local app=self.apps[self.selected]
 if not app or type(app.id)~="string" then
  self.status="Select an app from the catalog first."; self.status_color=C.warn; return false
 end
 if type(self.platform.hub_download)~="function" then
  self.status="Hub download adapter unavailable."; self.status_color=C.warn; return false
 end
 local ok,path,detail=pcall(self.platform.hub_download,app.id)
 if not ok or type(path)~="string" or path=="" then
  self.status="Download failed: "..tostring(detail or path); self.status_color=C.warn; return false
 end
 if not install_after then
  self.status="Downloaded to "..path; self.status_color=C.good; return true
 end
 if type(self.platform.install_package)~="function" then
  self.status="Downloaded, but native installer adapter is unavailable: "..path
  self.status_color=C.warn; return false
 end
 local iok,success,reason=pcall(self.platform.install_package,path)
 if not iok or success~=true then
  self.status="Downloaded; installation failed: "..tostring(reason or success)
  self.status_color=C.warn; return false
 end
 self.status="Installed "..app.title.." successfully."; self.status_color=C.good; return true
end
function Hub:upload()
 if type(self.platform.choose_package)~="function" or type(self.platform.hub_upload)~="function" then
  self.status="Upload requires native file-picker and authenticated hub_upload adapters."
  self.status_color=C.warn; return false
 end
 local ok,path=pcall(self.platform.choose_package,{".lap",".luap"})
 if not ok or type(path)~="string" or path=="" then
  self.status="No package selected for upload."; self.status_color=C.warn; return false
 end
 local uok,result,detail=pcall(self.platform.hub_upload,path)
 if not uok or type(result)~="table" then
  self.status="Upload failed: "..tostring(detail or result); self.status_color=C.warn; return false
 end
 self.status="Uploaded "..tostring(result.title or result.id or path).." to App Hub."
 self.status_color=C.good; return self:refresh()
end
function Hub:draw(width,height)
 if width and height then self.width,self.height=width,height end
 local p,w,h=self.platform,self.width,self.height
 p.fill_rect(0,0,w,h,C.bg); p.fill_rect(0,0,w,58,C.panel)
 p.fill_rect(0,58,4,h-58,C.accent)
 p.draw_text("Xzuff App Hub",22,18,C.text)
 p.draw_text("Browse, download, install and upload LAP/LUAP apps",20,78,C.muted)
 for _,b in ipairs(self:_buttons()) do
  p.fill_rect(b.x,b.y,b.w,b.h,self.pressed==b.id and {15,100,176,255} or C.accent)
  p.draw_text(b.label,b.x+10,b.y+13,C.text)
 end
 local y=166
 if #self.apps==0 then p.draw_text("No apps loaded. Tap Refresh to connect to the catalog.",22,y,C.muted) end
 for i,app in ipairs(self.apps) do
  if y+48<h-78 then
   p.fill_rect(20,y,w-40,50,i==self.selected and C.card or C.panel)
   p.fill_rect(20,y,3,50,i==self.selected and C.accent or C.panel)
   p.draw_text(tostring(app.title or "Untitled").."  "..tostring(app.version or ""),32,y+7,C.text)
   p.draw_text(tostring(app.publisher or "Unknown publisher").." · "..tostring(app.package_type or ""),32,y+29,C.muted)
  end
  y=y+58
 end
 p.fill_rect(20,h-70,w-40,46,C.panel)
 p.draw_text(self.status,30,h-53,self.status_color)
end
return Hub
