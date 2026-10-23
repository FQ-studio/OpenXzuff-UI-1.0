package.path = "src/?.lua;src/?/init.lua;runtime/lua/?.lua;runtime/lua/?/init.lua;" .. package.path
local Hub=require("apps.xzuff_app_hub")
local installed,downloaded,uploaded=nil,nil,nil
local platform={
 fill_rect=function() end, draw_text=function() end,
 hub_list=function() return {{id="demo-1",title="Demo App",version="1.0.0",publisher="FQ Studio",package_type="LAP"}} end,
 hub_download=function(id) downloaded=id; return "/tmp/demo.lap" end,
 install_package=function(path) installed=path; return true end,
 choose_package=function() return "/tmp/new.luap" end,
 hub_upload=function(path) uploaded=path; return {id="new-1",title="New App"} end,
}
local hub=Hub.new(platform,{width=640,height=480})
hub:draw()
assert(hub:refresh() and #hub.apps==1,"catalog refresh loads apps")
assert(hub:download_selected(false) and downloaded=="demo-1","download calls host adapter")
assert(hub:download_selected(true) and installed=="/tmp/demo.lap","install connects to native host adapter")
assert(hub:upload() and uploaded=="/tmp/new.luap","upload calls host adapter and refreshes")
local offline=Hub.new({fill_rect=function() end,draw_text=function() end},{width=640,height=480})
assert(not offline:refresh(),"missing network adapter fails safely")
print("Xzuff App Hub GUI tests passed")
