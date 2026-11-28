#include <cstdint>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <iterator>
#include <stdexcept>
#include <stdexcept>
#include <string>
#include <vector>
namespace {
using B=std::vector<std::uint8_t>;
std::uint16_t r16(const B&b,std::size_t p){if(p>b.size()||b.size()-p<2)throw std::runtime_error("truncated ELF field");return std::uint16_t(b[p])|(std::uint16_t(b[p+1])<<8);}
std::uint32_t r32(const B&b,std::size_t p){if(p>b.size()||b.size()-p<4)throw std::runtime_error("truncated ELF field");std::uint32_t v=0;for(unsigned i=0;i<4;++i)v|=std::uint32_t(b[p+i])<<(8*i);return v;}
std::uint64_t r64(const B&b,std::size_t p){if(p>b.size()||b.size()-p<8)throw std::runtime_error("truncated ELF field");std::uint64_t v=0;for(unsigned i=0;i<8;++i)v|=std::uint64_t(b[p+i])<<(8*i);return v;}
void w(B&b,std::uint64_t v,unsigned n){for(unsigned i=0;i<n;++i)b.push_back(std::uint8_t(v>>(8*i)));}
B read(const std::filesystem::path&p){std::ifstream f(p,std::ios::binary);if(!f)throw std::runtime_error("cannot open input");return B(std::istreambuf_iterator<char>(f),{});}
void write(const std::filesystem::path&p,const B&b){std::ofstream f(p,std::ios::binary|std::ios::trunc);if(!f)throw std::runtime_error("cannot create output");f.write(reinterpret_cast<const char*>(b.data()),std::streamsize(b.size()));if(!f)throw std::runtime_error("write failed");}
std::uint64_t hash(const B&b,std::size_t start){std::uint64_t h=14695981039346656037ull;for(std::size_t i=start;i<b.size();++i){h^=b[i];h*=1099511628211ull;}return h;}
struct S{std::uint64_t va,mem,file,off,align;std::uint32_t flags;};
B convert(const B&e){
 if(e.size()<64||e[0]!=0x7f||e[1]!='E'||e[2]!='L'||e[3]!='F')throw std::runtime_error("input is not ELF");
 if(e[4]!=2||e[5]!=1||e[6]!=1)throw std::runtime_error("requires ELF64 little-endian version 1");
 if(r16(e,16)!=2)throw std::runtime_error("requires static ET_EXEC ELF");
 if(r16(e,18)!=243)throw std::runtime_error("ELF machine is not RISC-V");
 if(r16(e,52)!=64||r16(e,54)!=56)throw std::runtime_error("invalid ELF64 header sizes");
 auto phoff=r64(e,32);auto n=r16(e,56);
 if(!n||phoff>e.size()||std::uint64_t(n)*56>e.size()-phoff)throw std::runtime_error("invalid program-header table");
 std::vector<S> ss;
 for(std::uint16_t i=0;i<n;++i){auto p=std::size_t(phoff)+std::size_t(i)*56;if(r32(e,p)!=1)continue;
  const auto fl=r32(e,p+4); const auto off=r64(e,p+8),va=r64(e,p+16),fs=r64(e,p+32),ms=r64(e,p+40),al=r64(e,p+48);
  if(fs>ms||off>e.size()||fs>e.size()-off)throw std::runtime_error("invalid PT_LOAD bounds");
  if(ms==0) continue;
  if(al>1&&(al&(al-1))) throw std::runtime_error("invalid PT_LOAD alignment");
  ss.push_back({va,ms,fs,off,al,fl});
 }
 if(ss.empty()||ss.size()>64)throw std::runtime_error("requires 1..64 PT_LOAD segments");
 B o;o.insert(o.end(),{'G','N','B','1'});w(o,1,2);w(o,56,2);w(o,1,4);w(o,0,4);w(o,r64(e,24),8);
 w(o,ss.size(),4);w(o,0,4);w(o,56,8);w(o,56+ss.size()*48,8);w(o,0,8);
 std::uint64_t payload=56+ss.size()*48;
 for(auto&s:ss){w(o,s.va,8);w(o,s.mem,8);w(o,s.file,8);w(o,payload,8);w(o,s.align,8);w(o,s.flags,4);w(o,0,4);payload+=s.file;}
 for(auto&s:ss)o.insert(o.end(),e.begin()+std::ptrdiff_t(s.off),e.begin()+std::ptrdiff_t(s.off+s.file));
 auto sum=hash(o,56);for(unsigned i=0;i<8;++i)o[48+i]=std::uint8_t(sum>>(8*i));return o;
}
bool verify(const B&b){
 if(b.size()<56||b[0]!='G'||b[1]!='N'||b[2]!='B'||b[3]!='1'||r16(b,4)!=1||r16(b,6)!=56||r32(b,8)!=1)return false;
 const auto n=r32(b,24); const auto table=r64(b,32),payload=r64(b,40);
 if(!n||n>64||table!=56||payload!=56+std::uint64_t(n)*48||payload>b.size())return false;
 if(hash(b,56)!=r64(b,48))return false;
 for(std::uint32_t i=0;i<n;++i){const auto p=56+std::size_t(i)*48; const auto mem=r64(b,p+8),sz=r64(b,p+16),off=r64(b,p+24);
  if(sz>mem||off<payload||off>b.size()||sz>b.size()-off) return false;
  const auto al=r64(b,p+32);
  if(al>1&&(al&(al-1))) return false;
 }
 return true;
}
}
int main(int argc,char**argv){try{
 if(argc==3&&std::string(argv[1])=="--verify"){auto b=read(argv[2]);if(!verify(b)){std::cerr<<"GNB verification failed\n";return 1;}std::cout<<"GNB v1 verified: RV64 checksum OK\n";return 0;}
 if(argc!=3){std::cerr<<"usage: gravity-elf-to-gnb input.elf output.gnb | --verify image.gnb\n";return 2;}
 auto b=convert(read(argv[1]));write(argv[2],b);std::cout<<"Converted RV64 ELF to GNB v1 ("<<b.size()<<" bytes)\n";return 0;
 }catch(const std::exception&e){std::cerr<<"GNB: "<<e.what()<<"\n";return 1;}}
