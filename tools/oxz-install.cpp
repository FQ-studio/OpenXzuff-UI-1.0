// OpenXzuff native LAP/LUAP installer. C++17; ZIP_STORED only.
#include <algorithm>
#include <cctype>
#include <cstdint>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <limits>
#include <string>
#include <vector>
#if !defined(_WIN32)
#include <unistd.h>
#endif
namespace fs = std::filesystem;
namespace {
constexpr std::size_t kMaxArchive=64u*1024u*1024u, kMaxFiles=2048, kMaxEntry=32u*1024u*1024u, kMaxTotal=128u*1024u*1024u;
struct Entry { std::string name; std::uint32_t crc{}, compressed{}, size{}, local{}; std::uint16_t method{}, flags{}; };
std::uint16_t u16(const std::vector<std::uint8_t>&b,std::size_t p){return std::uint16_t(b[p])|std::uint16_t(b[p+1])<<8;}
std::uint32_t u32(const std::vector<std::uint8_t>&b,std::size_t p){return std::uint32_t(b[p])|std::uint32_t(b[p+1])<<8|std::uint32_t(b[p+2])<<16|std::uint32_t(b[p+3])<<24;}
std::uint32_t crc32(const std::uint8_t*p,std::size_t n){std::uint32_t c=0xffffffffu;for(std::size_t i=0;i<n;i++){c^=p[i];for(int k=0;k<8;k++)c=(c>>1)^(0xedb88320u&std::uint32_t(-std::int32_t(c&1u)));}return ~c;}
bool safePath(const std::string&s){
 if(s.empty()||s[0]=='/'||s.find('\\')!=s.npos||s.find(':')!=s.npos)return false;
 fs::path p(s); if(p.is_absolute())return false;
 for(const auto&part:p) if(part==".."||part=="."||part.empty())return false;
 return true;
}
bool jsonString(const std::string&j,const std::string&key,std::string&out){
 const std::string needle="\""+key+"\"";auto p=j.find(needle);
 if(p==j.npos||j.find(needle,p+needle.size())!=j.npos)return false;
 p=j.find(':',p+needle.size());if(p==j.npos)return false;++p;
 while(p<j.size()&&std::isspace(static_cast<unsigned char>(j[p])))++p;
 if(p==j.size()||j[p++]!='"')return false;auto end=p;
 while(end<j.size()&&j[end]!='"'){if(j[end]=='\\'||static_cast<unsigned char>(j[end])<0x20)return false;++end;}
 if(end==j.size())return false;out=j.substr(p,end-p);return true;
}
bool validManifest(const std::vector<std::uint8_t>&b,std::size_t off,std::size_t n,const std::string&ext,std::string&id){
 if(n==0||n>65536||off>b.size()||n>b.size()-off)return false;
 std::string j(reinterpret_cast<const char*>(b.data()+off),n),format,language,entry,version;
 if(j.find('\0')!=j.npos||!jsonString(j,"format",format)||!jsonString(j,"language",language)||!jsonString(j,"entry",entry)||!jsonString(j,"id",id)||!jsonString(j,"version",version))return false;
 if(format!=(ext==".lap"?"LAP":"LUAP")||language!=(ext==".lap"?"Luau":"Lua")||!safePath(entry))return false;
 if(ext==".lap"&&(entry.size()<5||entry.substr(entry.size()-5)!=".luau"))return false;
 if(ext==".luap"&&(entry.size()<4||entry.substr(entry.size()-4)!=".lua"))return false;
 if(id.find('.')==id.npos||id.find('/')!=id.npos||id.find(' ')!=id.npos)return false;
 if(version.empty()||!std::isdigit(static_cast<unsigned char>(version[0]))||version.find('.')==version.npos)return false;
 return true;
}
bool readArchive(const fs::path&path,std::vector<std::uint8_t>&b){
 std::error_code ec;auto sz=fs::file_size(path,ec);if(ec||sz>kMaxArchive)return false;
 std::ifstream f(path,std::ios::binary);if(!f)return false;b.resize(static_cast<std::size_t>(sz));
 if(!b.empty())f.read(reinterpret_cast<char*>(b.data()),static_cast<std::streamsize>(b.size()));
 return f.good() || (f.eof() && static_cast<std::size_t>(f.gcount())==b.size());
}
bool extractData(const std::vector<std::uint8_t>&b,const Entry&e,std::size_t&off){
 if(e.local>b.size()||b.size()-e.local<30||u32(b,e.local)!=0x04034b50u)return false;
 off=std::size_t(e.local)+30+u16(b,e.local+26)+u16(b,e.local+28);
 return off<=b.size()&&e.size<=b.size()-off&&e.compressed==e.size&&crc32(b.data()+off,e.size)==e.crc;
}
bool install(const fs::path&package,const fs::path&root){
 const auto ext=package.extension().string();if(ext!=".lap"&&ext!=".luap"){std::cerr<<"ERROR: expected .lap or .luap\n";return false;}
 std::vector<std::uint8_t>b;if(!readArchive(package,b)||b.size()<22){std::cerr<<"ERROR: unreadable or oversized ZIP\n";return false;}
 std::size_t eocd=std::string::npos;const std::size_t begin=b.size()>65557?b.size()-65557:0;
 for(std::size_t p=b.size()-22;;--p){if(u32(b,p)==0x06054b50u){eocd=p;break;}if(p==begin)break;}
 if(eocd==std::string::npos||u16(b,eocd+4)||u16(b,eocd+6)||u16(b,eocd+8)!=u16(b,eocd+10)){std::cerr<<"ERROR: malformed/multi-disk ZIP\n";return false;}
 const auto count=u16(b,eocd+10),cdSize=u32(b,eocd+12),cdOff=u32(b,eocd+16);
 if(!count||count>kMaxFiles||std::uint64_t(cdOff)+cdSize>b.size()){std::cerr<<"ERROR: invalid ZIP directory\n";return false;}
 std::vector<Entry> entries;entries.reserve(count);std::size_t p=cdOff,total=0,manifestIndex=std::numeric_limits<std::size_t>::max();
 for(std::size_t i=0;i<count;i++){
  if(p+46>b.size()||u32(b,p)!=0x02014b50u){std::cerr<<"ERROR: malformed ZIP entry\n";return false;}
  Entry e;e.flags=u16(b,p+8);e.method=u16(b,p+10);e.crc=u32(b,p+16);e.compressed=u32(b,p+20);e.size=u32(b,p+24);auto nl=u16(b,p+28),xl=u16(b,p+30),cl=u16(b,p+32);e.local=u32(b,p+42);
  if(!nl||p+46u+nl+xl+cl>b.size()){std::cerr<<"ERROR: invalid entry name\n";return false;}
  e.name.assign(reinterpret_cast<const char*>(b.data()+p+46),nl);if(!e.name.empty()&&e.name.back()=='/')e.name.pop_back();
  if(!safePath(e.name)||std::any_of(entries.begin(),entries.end(),[&](const Entry&x){return x.name==e.name;})){std::cerr<<"ERROR: unsafe or duplicate path\n";return false;}
  if((e.flags&1u)||e.method!=0||e.compressed!=e.size||e.size>kMaxEntry||total>kMaxTotal-e.size){std::cerr<<"ERROR: unsupported compression/encryption or size limit exceeded (ZIP_STORED only)\n";return false;}
  total+=e.size;if(e.name=="manifest.json")manifestIndex=entries.size();entries.push_back(std::move(e));p+=46u+nl+xl+cl;
 }
 if(manifestIndex==std::numeric_limits<std::size_t>::max()){std::cerr<<"ERROR: manifest.json missing\n";return false;}
 std::size_t mo;const auto&me=entries[manifestIndex];std::string id;
 if(!extractData(b,me,mo)||!validManifest(b,mo,me.size,ext,id)){std::cerr<<"ERROR: invalid manifest or CRC\n";return false;}
 // Entry script must be present in the package.
 std::string manifestText(reinterpret_cast<const char*>(b.data()+mo),me.size),entryName;
 if(!jsonString(manifestText,"entry",entryName)||std::none_of(entries.begin(),entries.end(),[&](const Entry&e){return e.name==entryName;})){std::cerr<<"ERROR: entry script not found\n";return false;}
 std::error_code ec;fs::create_directories(root,ec);if(ec){std::cerr<<"ERROR: cannot create app root\n";return false;}
 const fs::path dest=root/id;if(fs::exists(dest)){std::cerr<<"ERROR: app already installed; refusing overwrite\n";return false;}
 const fs::path stage=root/(".staging-"+std::to_string(
#if defined(_WIN32)
  1
#else
  static_cast<long long>(::getpid())
#endif
 ));
 if(fs::exists(stage)){std::cerr<<"ERROR: staging path already exists\n";return false;}
 if(!fs::create_directory(stage,ec)||ec){std::cerr<<"ERROR: cannot create staging directory\n";return false;}
 bool ok=true;
 for(const auto&e:entries){std::size_t off;if(!extractData(b,e,off)){ok=false;break;}const fs::path target=stage/fs::path(e.name);auto parent=target.parent_path();fs::create_directories(parent,ec);if(ec){ok=false;break;}std::ofstream out(target,std::ios::binary|std::ios::trunc);if(!out){ok=false;break;}out.write(reinterpret_cast<const char*>(b.data()+off),static_cast<std::streamsize>(e.size));out.close();if(!out){ok=false;break;}}
 if(!ok){fs::remove_all(stage,ec);std::cerr<<"ERROR: extraction failed; staging removed\n";return false;}
 fs::rename(stage,dest,ec);if(ec){fs::remove_all(stage,ec);std::cerr<<"ERROR: cannot activate install (existing target or filesystem error)\n";return false;}
 std::cout<<"INSTALLED: "<<id<<"\nPATH: "<<dest.string()<<"\nScripts were not executed; a sandboxed runtime is required.\n";return true;
}
}
int main(int argc,char**argv){
 if(argc==4&&std::string(argv[1])=="install")return install(argv[2],argv[3])?0:2;
 std::cerr<<"Usage: "<<argv[0]<<" install PACKAGE.lap|PACKAGE.luap APP_ROOT\n";return 2;
}
