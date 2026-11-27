#include "gravity/ai/gail.hpp"
#include <cctype>
#include <limits>
#include <string>
namespace gravity::ai {
namespace {
class Parser {
public:
 explicit Parser(std::string_view s):s_(s){}
 ArithmeticResult run(){try{skip();const auto v=expression();skip();if(pos_!=s_.size())return {false,0,"unexpected character"};if(v<std::numeric_limits<std::int64_t>::min()||v>std::numeric_limits<std::int64_t>::max())return {false,0,"integer overflow"};return {true,static_cast<std::int64_t>(v),{}};}catch(const char* e){return {false,0,e};}}
private:
 using Wide=__int128_t;
 void skip(){while(pos_<s_.size()&&std::isspace(static_cast<unsigned char>(s_[pos_])))++pos_;}
 bool take(char c){skip();if(pos_<s_.size()&&s_[pos_]==c){++pos_;return true;}return false;}
 Wide expression(){Wide v=term();for(;;){if(take('+'))v=checked(v+term());else if(take('-'))v=checked(v-term());else return v;}}
 Wide term(){Wide v=factor();for(;;){if(take('*'))v=checked(v*factor());else if(take('/')||take('\u00f7')){Wide d=factor();if(d==0)throw "division by zero";if(v==static_cast<Wide>(std::numeric_limits<std::int64_t>::min())&&d==-1)throw "integer overflow";v/=d;}else return v;}}
 Wide factor(){skip();if(take('+'))return factor();if(take('-'))return checked(-factor());if(take('(')){Wide v=expression();if(!take(')'))throw "missing closing parenthesis";return v;}skip();if(pos_>=s_.size()||!std::isdigit(static_cast<unsigned char>(s_[pos_])))throw "expected number";Wide v=0;while(pos_<s_.size()&&std::isdigit(static_cast<unsigned char>(s_[pos_]))){v=v*10+(s_[pos_++]-'0');if(v>static_cast<Wide>(std::numeric_limits<std::int64_t>::max())+1)throw "integer overflow";}return v;}
 Wide checked(Wide v){const Wide lo=static_cast<Wide>(std::numeric_limits<std::int64_t>::min()),hi=static_cast<Wide>(std::numeric_limits<std::int64_t>::max());if(v<lo||v>hi)throw "integer overflow";return v;}
 std::string_view s_;std::size_t pos_{};
};
std::string trim(std::string_view s){std::size_t a=0,b=s.size();while(a<b&&std::isspace(static_cast<unsigned char>(s[a])))++a;while(b>a&&std::isspace(static_cast<unsigned char>(s[b-1])))--b;return std::string(s.substr(a,b-a));}
}
ArithmeticResult ArithmeticEngine::evaluate(std::string_view e)const{return Parser(e).run();}
std::string SentenceEngine::compose(std::string_view question,std::int64_t answer)const{return "Hasil pengiraan bagi "+trim(question)+" ialah "+std::to_string(answer)+".";}
bool GailRuntime::load(std::string_view source,std::string* error){
 const bool start=source.find("@start")!=std::string_view::npos;
 const bool ai=source.find("@ai")!=std::string_view::npos;
 const bool native=source.find("mode native;")!=std::string_view::npos;
 const bool precision=source.find("precision int8;")!=std::string_view::npos;
 const bool end=source.find("code_end;")!=std::string_view::npos;
 if(!(start&&ai&&native&&precision&&end)){loaded_=false;if(error)*error="GAIL program requires @start, native mode, precision int8, @ai and code_end;";return false;}
 loaded_=true;return true;
}
GailResult GailRuntime::ask(std::string_view question)const{
 if(!loaded_)return {false,0,{},"GAIL program is not loaded"};
 auto r=arithmetic_.evaluate(question);if(!r.ok)return {false,0,{},r.error};
 return {true,r.value,sentence_.compose(question,r.value),{}};
}
} // namespace gravity::ai
