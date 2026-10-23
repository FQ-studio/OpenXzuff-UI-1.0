package.path = "src/?.lua;src/?/init.lua;runtime/lua/?.lua;runtime/lua/?/init.lua;" .. package.path
local Installer=require("apps.package_installer")
local calls={rects=0,texts=0,installs={}}
local platform={
 fill_rect=function() calls.rects=calls.rects+1 end,
 draw_text=function() calls.texts=calls.texts+1 end,
 choose_package=function() return "/apps/hello.lap" end,
 install_package=function(path) calls.installs[#calls.installs+1]=path; return true end,
}
local app=Installer.new(platform,{width=640,height=480})
app:draw()
assert(calls.rects>0 and calls.texts>0,"GUI draws controls and labels")
assert(app:pointer_pressed(30,290,1),"choose button accepts press")
assert(app:pointer_released(30,290,1),"choose button selects package")
assert(app.package_path=="/apps/hello.lap","selected package saved")
local install_x=48+math.min(220,math.floor((640-72)/2))+2
assert(app:pointer_pressed(install_x,290,1),"install button accepts press")
assert(app:pointer_released(install_x,290,1),"install button invokes host service")
assert(#calls.installs==1 and calls.installs[1]=="/apps/hello.lap","native service receives path")
local nohost=Installer.new({fill_rect=function() end,draw_text=function() end},{width=640,height=480})
assert(nohost:install_selected()==false,"cannot install without selected package")
assert(nohost.status:find("Choose a package",1,true),"missing selection explained")
print("OpenXzuff graphical package installer tests passed")
