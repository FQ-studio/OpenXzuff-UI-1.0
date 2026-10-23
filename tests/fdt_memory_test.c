#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include "gravity/fdt.h"

static void put32(uint8_t *p, uint32_t v)
{
    p[0]=(uint8_t)(v>>24); p[1]=(uint8_t)(v>>16);
    p[2]=(uint8_t)(v>>8); p[3]=(uint8_t)v;
}
static void put64(uint8_t *p, uint64_t v)
{
    put32(p,(uint32_t)(v>>32)); put32(p+4,(uint32_t)v);
}

int main(void)
{
    uint8_t blob[256] = {0};
    const char strings[] = "#address-cells\0#size-cells\0device_type\0reg\0";
    size_t off_strings=80, off_struct=(off_strings+sizeof(strings)+3u)&~(size_t)3u, p=off_struct;
    gravity_memory_map_t map;
    size_t parsed_size = 0;

#define TOKEN(v) do { put32(blob+p,(v)); p+=4; } while(0)
#define PROP(len,off) do { put32(blob+p,(len)); put32(blob+p+4,(off)); p+=8; } while(0)
    /* FDT reservation map at byte 40: reserve [0x80000000,0x80001000). */
    put64(blob+40,0x80000000ULL); put64(blob+48,0x1000); put64(blob+56,0); put64(blob+64,0);
    TOKEN(1); blob[p++]=0; while(p%4) blob[p++]=0;
    TOKEN(3); PROP(4,0); put32(blob+p,2); p+=4; while(p%4) blob[p++]=0;
    TOKEN(3); PROP(4,15); put32(blob+p,2); p+=4; while(p%4) blob[p++]=0;
    TOKEN(1); memcpy(blob+p,"memory@80000000",16); p+=16; while(p%4) blob[p++]=0;
    TOKEN(3); PROP(7,27); memcpy(blob+p,"memory",7); p+=7; while(p%4) blob[p++]=0;
    TOKEN(3); PROP(16,39); put64(blob+p,0x80000000ULL); put64(blob+p+8,0x08000000ULL); p+=16;
    TOKEN(2);
    TOKEN(1); memcpy(blob+p,"reserved-memory",16); p+=16; while(p%4) blob[p++]=0;
    TOKEN(1); memcpy(blob+p,"secure@87000000",16); p+=16; while(p%4) blob[p++]=0;
    TOKEN(3); PROP(16,39); put64(blob+p,0x87000000ULL); put64(blob+p+8,0x1000); p+=16;
    TOKEN(2); TOKEN(2); TOKEN(2); TOKEN(9);
#undef TOKEN
#undef PROP
    put32(blob,0xd00dfeed); put32(blob+4,(uint32_t)p);
    put32(blob+8,(uint32_t)off_struct); put32(blob+12,(uint32_t)off_strings);
    put32(blob+16,40); put32(blob+20,17); put32(blob+24,16);
    put32(blob+28,0); put32(blob+32,sizeof(strings)); put32(blob+36,(uint32_t)(p-off_struct));
    gravity_memory_map_init(&map);
    assert(gravity_fdt_add_memory(blob,sizeof(blob),&map,&parsed_size)==0);
    assert(parsed_size == p);
    assert(map.count==1);
    assert(map.regions[0].start==0x80001000u);
    assert(map.regions[0].end==0x88000000u);
    assert(gravity_fdt_add_memory(blob,20,&map,&parsed_size)==-1);
    blob[0]=0;
    assert(gravity_fdt_add_memory(blob,sizeof(blob),&map,&parsed_size)==-1);
    puts("Gravity FDT memory discovery tests: PASS");
    return 0;
}
