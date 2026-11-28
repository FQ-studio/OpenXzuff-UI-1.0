#include "gravity/ai/knowledge.hpp"
#include <cassert>
#include <cmath>
#include <filesystem>
#include <iostream>
int main(){
 using namespace gravity::ai;KnowledgeStore store;std::string error;
 assert(store.execute("CREATE TABLE knowledge (id INTEGER PRIMARY KEY, title TEXT, body TEXT)",&error));
 assert(store.insert({1,"RV64 kernel","OpenXzuff uses a RISC-V 64-bit kernel with supervisor mode and Sv39 virtual memory."},&error));
 assert(store.insert({2,"Gravity AI","Gravity AI combines a custom knowledge database, RAG retrieval and native C++ inference."},&error));
 assert(!store.insert({1,"duplicate","duplicate id"},&error));
 RagEngine rag(store);const auto hits=rag.search("C++ RAG inference",2);assert(!hits.empty()&&hits.front().document.id==2);
 const auto path=std::filesystem::temp_directory_path()/"gravity-ai-test.gxk";assert(store.save(path,&error));KnowledgeStore restored;assert(restored.load(path,&error));assert(restored.size()==2);std::filesystem::remove(path);
 QuantizedLinear layer(2,3,{1,0,0,1,-1,1},{0.0F,0.0F,0.0F},0.5F);assert(layer.valid());NativeBrain brain(layer);const auto logits=brain.infer_logits({2.0F,1.0F});assert(logits.size()==3);for(float v:logits)assert(std::isfinite(v));assert(NativeBrain::greedy_token(logits)==0);assert(NativeBrain::greedy_token({})==0);
 std::cout<<"Gravity AI native SQL/RAG/inference primitives: PASS\n";return 0;
}
