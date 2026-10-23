#include <assert.h>
#include <stdio.h>
#include <string.h>
#include "../tools/oxz-install.c"
int main(void){assert(safe("main.luau"));assert(safe("assets/icon.png"));assert(!safe("../escape"));assert(!safe("/absolute"));assert(!safe("assets/../../escape"));assert(!safe("C:/escape"));{const char*a="{\"format\":\"LAP\",\"language\":\"Luau\",\"entry\":\"main.luau\",\"id\":\"org.example.test\",\"version\":\"1.0.0\"}";const char*b="{\"format\":\"LUAP\",\"language\":\"Lua\",\"entry\":\"main.lua\",\"id\":\"org.example.test\",\"version\":\"1.0.0\"}";assert(manifest((const uint8_t*)a,strlen(a),".lap"));assert(manifest((const uint8_t*)b,strlen(b),".luap"));assert(!manifest((const uint8_t*)a,strlen(a),".luap"));}puts("native installer tests: PASS");return 0;}
