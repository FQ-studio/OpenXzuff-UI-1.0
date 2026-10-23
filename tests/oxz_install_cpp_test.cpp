#include <cassert>
#include <cstdint>
#include <iostream>
#include <string>
#define main oxz_installer_program_main
#include "../tools/oxz-install.cpp"
#undef main
int main(){
 assert(safePath("main.luau"));assert(safePath("assets/icon.png"));
 assert(!safePath("../escape"));assert(!safePath("/absolute"));assert(!safePath("assets/../../escape"));assert(!safePath("C:/escape"));
 std::string id;
 const std::string lap="{\"format\":\"LAP\",\"language\":\"Luau\",\"entry\":\"main.luau\",\"id\":\"org.example.test\",\"version\":\"1.0.0\"}";
 const std::string luap="{\"format\":\"LUAP\",\"language\":\"Lua\",\"entry\":\"main.lua\",\"id\":\"org.example.test\",\"version\":\"1.0.0\"}";
 std::vector<std::uint8_t>a(lap.begin(),lap.end()),b(luap.begin(),luap.end());
 assert(validManifest(a,0,a.size(),".lap",id));assert(validManifest(b,0,b.size(),".luap",id));assert(!validManifest(a,0,a.size(),".luap",id));
 std::cout<<"native C++ installer tests: PASS\n";
}
