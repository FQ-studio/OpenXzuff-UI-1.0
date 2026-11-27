#include "gravity/ai/knowledge.hpp"
#include <algorithm>
#include <cctype>
#include <cmath>
#include <fstream>
#include <sstream>
#include <unordered_map>
#include <unordered_set>
#include <utility>
namespace gravity::ai {
namespace {
constexpr char kMagic[] = "GXKDB01";
constexpr std::uint32_t kMaxDocuments = 100000, kMaxStringBytes = 4U * 1024U * 1024U;
std::string normalize(std::string_view value) {
 std::string result; result.reserve(value.size());
 for (unsigned char ch : value) result.push_back((std::isalnum(ch) || ch >= 0x80) ? static_cast<char>(std::tolower(ch)) : ' ');
 return result;
}
std::vector<std::string> tokens(std::string_view value) {
 std::istringstream input(normalize(value)); std::vector<std::string> result; std::string token;
 while (input >> token) if (token.size() > 1) result.push_back(std::move(token));
 return result;
}
bool writeU32(std::ostream& out, std::uint32_t v) { for (unsigned i=0;i<4;++i) out.put(static_cast<char>((v>>(i*8U))&0xffU)); return bool(out); }
bool writeU64(std::ostream& out, std::uint64_t v) { for (unsigned i=0;i<8;++i) out.put(static_cast<char>((v>>(i*8U))&0xffU)); return bool(out); }
bool readU32(std::istream& in, std::uint32_t& v) { v=0; for(unsigned i=0;i<4;++i){int c=in.get();if(c==EOF)return false;v|=std::uint32_t(static_cast<unsigned char>(c))<<(i*8U);}return true; }
bool readU64(std::istream& in, std::uint64_t& v) { v=0; for(unsigned i=0;i<8;++i){int c=in.get();if(c==EOF)return false;v|=std::uint64_t(static_cast<unsigned char>(c))<<(i*8U);}return true; }
bool writeString(std::ostream& out,const std::string& s){if(s.size()>kMaxStringBytes||!writeU32(out,static_cast<std::uint32_t>(s.size())))return false;out.write(s.data(),static_cast<std::streamsize>(s.size()));return bool(out);}
bool readString(std::istream& in,std::string& s){std::uint32_t n=0;if(!readU32(in,n)||n>kMaxStringBytes)return false;s.resize(n);in.read(s.data(),static_cast<std::streamsize>(n));return bool(in);}
}
bool KnowledgeStore::execute(std::string_view sql,std::string* error){
 std::string q(sql);std::transform(q.begin(),q.end(),q.begin(),[](unsigned char c){return static_cast<char>(std::toupper(c));});
 if(q.find("CREATE TABLE KNOWLEDGE")==0){table_created_=true;return true;}
 if(q.find("SELECT ")==0&&q.find(" FROM KNOWLEDGE")!=std::string::npos){if(!table_created_){if(error)*error="knowledge table does not exist";return false;}return true;}
 if (error) *error = "unsupported SQL statement; supported: CREATE TABLE knowledge and SELECT ... FROM knowledge";
 return false;
}
bool KnowledgeStore::insert(KnowledgeDocument d,std::string* error){
 if(!table_created_){if(error)*error="create knowledge table before inserting";return false;}
 if(d.id==0||d.title.empty()||d.body.empty()||d.title.size()>kMaxStringBytes||d.body.size()>kMaxStringBytes){if(error)*error="invalid document or size limit";return false;}
 if(std::find_if(documents_.begin(),documents_.end(),[&](const auto& x){return x.id==d.id;})!=documents_.end()){if(error)*error="duplicate document id";return false;}
 documents_.push_back(std::move(d));return true;
}
std::vector<KnowledgeDocument> KnowledgeStore::documents()const{return documents_;}
std::size_t KnowledgeStore::size()const noexcept{return documents_.size();}
bool KnowledgeStore::save(const std::filesystem::path& p,std::string* error)const{
 std::ofstream out(p,std::ios::binary|std::ios::trunc);if(!out){if(error)*error="cannot create database";return false;}
 out.write(kMagic,sizeof(kMagic)-1);if(!writeU32(out,static_cast<std::uint32_t>(documents_.size()))){if(error)*error="header write failed";return false;}
 for (const auto& d : documents_) {
  if (!writeU64(out, d.id) || !writeString(out, d.title) || !writeString(out, d.body)) {
   if (error) *error = "record write failed";
   return false;
  }
 }
 return true;
}
bool KnowledgeStore::load(const std::filesystem::path& p,std::string* error){
 std::ifstream in(p,std::ios::binary);if(!in){if(error)*error="cannot open database";return false;}
 char magic[sizeof(kMagic)-1]{};in.read(magic,sizeof(magic));std::uint32_t count=0;
 if(!in||std::string_view(magic,sizeof(magic))!=std::string_view(kMagic,sizeof(kMagic)-1)||!readU32(in,count)||count>kMaxDocuments){if(error)*error="invalid database header";return false;}
 std::vector<KnowledgeDocument> loaded;loaded.reserve(count);std::unordered_set<std::uint64_t> ids;
 for(std::uint32_t i=0;i<count;++i){KnowledgeDocument d;if(!readU64(in,d.id)||!readString(in,d.title)||!readString(in,d.body)||d.id==0||d.title.empty()||d.body.empty()||!ids.insert(d.id).second){if(error)*error="corrupt document record";return false;}loaded.push_back(std::move(d));}
 if(in.peek()!=EOF){if(error)*error="unexpected trailing data";return false;}documents_=std::move(loaded);table_created_=true;return true;
}
std::vector<KnowledgeHit> RagEngine::search(std::string_view query,std::size_t limit)const{
 const auto qt=tokens(query); const auto docs=store_.documents(); std::vector<KnowledgeHit> hits; if(qt.empty()||docs.empty()||limit==0)return hits;
 std::unordered_map<std::string,std::size_t> df;std::vector<std::unordered_map<std::string,std::size_t>> tf;std::vector<std::size_t> lengths;std::size_t total=0;
 for(const auto& d:docs){auto words=tokens(d.title+" "+d.body);std::unordered_map<std::string,std::size_t> counts;for(const auto& w:words)++counts[w];for(const auto& e:counts)++df[e.first];total+=words.size();lengths.push_back(words.size());tf.push_back(std::move(counts));}
 const double avg=static_cast<double>(total)/static_cast<double>(docs.size());
 for(std::size_t i=0;i<docs.size();++i){double score=0;for(const auto& w:qt){auto it=tf[i].find(w);if(it==tf[i].end())continue;const double n=static_cast<double>(docs.size()), f=static_cast<double>(df[w]), freq=static_cast<double>(it->second), len=static_cast<double>(lengths[i]);const double idf=std::log(1.0+(n-f+0.5)/(f+0.5));score+=idf*(freq*2.2)/(freq+1.2*(0.25+0.75*len/avg));}if(score>0)hits.push_back({docs[i],score});}
 std::sort(hits.begin(),hits.end(),[](const auto&a,const auto&b){return a.score>b.score;});if(hits.size()>limit)hits.resize(limit);return hits;
}
QuantizedLinear::QuantizedLinear(std::size_t in,std::size_t out,std::vector<std::int8_t>w,std::vector<float>b,float s):input_size_(in),output_size_(out),weights_(std::move(w)),bias_(std::move(b)),weight_scale_(s){}
bool QuantizedLinear::valid()const noexcept{return input_size_>0&&output_size_>0&&weights_.size()==input_size_*output_size_&&bias_.size()==output_size_&&std::isfinite(weight_scale_)&&weight_scale_>0;}
std::vector<float> QuantizedLinear::infer(const std::vector<float>& input)const{
 if (!valid() || input.size() != input_size_) return {};
 std::vector<float> out(output_size_, 0.0F);
 for(std::size_t r=0;r<output_size_;++r){float sum=bias_[r];for(std::size_t c=0;c<input_size_;++c)sum+=input[c]*static_cast<float>(weights_[r*input_size_+c])*weight_scale_;out[r]=sum;}return out;
}
std::vector<float> NativeBrain::infer_logits(const std::vector<float>& features)const{auto v=projection_.infer(features);for(auto& x:v)x=std::tanh(x);return v;}
std::size_t NativeBrain::greedy_token(const std::vector<float>& logits){if(logits.empty())return 0;return static_cast<std::size_t>(std::distance(logits.begin(),std::max_element(logits.begin(),logits.end())));}
} // namespace gravity::ai
