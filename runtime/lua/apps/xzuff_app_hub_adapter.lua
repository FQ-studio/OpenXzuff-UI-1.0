-- Host bridge for Xzuff App Hub HTTP API.
-- The platform must implement HTTP, JSON, file download and native installer primitives.
local Adapter = {}
Adapter.__index = Adapter

function Adapter.new(platform, options)
  assert(type(platform)=="table","platform API is required")
  options=options or {}
  local base=(options.base_url or "http://127.0.0.1:8787"):gsub("/+$","")
  return setmetatable({platform=platform,base_url=base},Adapter)
end

function Adapter:_require(name)
  local fn=self.platform[name]
  if type(fn)~="function" then error("Host API missing: "..name,2) end
  return fn
end

function Adapter:hub_list(query)
  local get=self:_require("http_get")
  local decode=self:_require("json_decode")
  local url=self.base_url.."/api/apps"
  if query and query~="" then
    local encode=self:_require("url_encode")
    url=url.."?q="..encode(query)
  end
  local status,body=get(url,{})
  if status~=200 then return nil,"catalog HTTP status "..tostring(status) end
  local records,err=decode(body)
  if type(records)~="table" then return nil,err or "invalid catalog JSON" end
  return records
end

function Adapter:hub_download(app_id, app_record)
  assert(type(app_id)=="string" and app_id:match("^[%w_.%-]+$") and not app_id:find("..",1,true),
    "invalid App Hub ID")
  local download=self:_require("download_to_file")
  local ext=(type(app_record)=="table" and app_record.package_type=="LUAP") and ".luap" or ".lap"
  local path=self:_require("make_download_path")("xzuff-"..app_id..ext)
  local status,actual_path=download(self.base_url.."/api/apps/"..app_id.."/download",path,{})
  if status~=200 or type(actual_path)~="string" or actual_path=="" then
    return nil,"download failed: "..tostring(status)
  end
  return actual_path
end

function Adapter:hub_upload(path, metadata)
  assert(type(path)=="string" and path~="","package path is required")
  assert(type(metadata)=="table","upload metadata is required")
  local post=self:_require("http_post_multipart")
  local token=self:_require("get_secret")("xzuff_hub_upload_token")
  if type(token)~="string" or token=="" then
    return nil,"Set the host secret xzuff_hub_upload_token before uploading."
  end
  local fields={
    title=metadata.title, publisher=metadata.publisher,
    version=metadata.version or "1.0.0", category=metadata.category or "Other",
    description=metadata.description or ""
  }
  local status,body=post(self.base_url.."/api/apps",fields,"package",path,
    {Authorization="Bearer "..token})
  if status~=200 and status~=201 then return nil,"upload HTTP status "..tostring(status) end
  local record,err= self:_require("json_decode")(body)
  if type(record)~="table" then return nil,err or "invalid upload response" end
  return record
end

function Adapter:install_package(path)
  return self:_require("install_package")(path)
end

function Adapter:choose_package(extensions)
  return self:_require("choose_package")(extensions)
end

function Adapter:prompt_text(label, default)
  return self:_require("prompt_text")(label,default)
end

-- Return a drawing platform that forwards UI drawing to the original host and
-- exposes the network/file/native callbacks as plain functions for the GUI.
function Adapter.attach(platform, options)
  local bridge=Adapter.new(platform,options)
  local proxy={
    hub_list=function(query) return bridge:hub_list(query) end,
    hub_download=function(id) return bridge:hub_download(id) end,
    hub_upload=function(path,metadata) return bridge:hub_upload(path,metadata) end,
    install_package=function(path) return bridge:install_package(path) end,
    choose_package=function(exts) return bridge:choose_package(exts) end,
    prompt_text=function(label,default) return bridge:prompt_text(label,default) end,
  }
  return setmetatable(proxy,{__index=platform})
end

return Adapter
