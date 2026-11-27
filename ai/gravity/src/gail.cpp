#include "gravity/ai/gail.hpp"
#include <cctype>
#include <limits>
#include <string>
namespace gravity::ai {
namespace {
class Parser {
public:
 explicit Parser(std::string_view s):s_(s){}
 ArithmeticResult run(){try{skip();auto v=expression();skip();if(pos_!=s_.size())return {false,0,"unexpected character"};return {true,v,{}};}catch(const char* e){return {false,0,e};}}
private:
 static std::int64_t add(std::int64_t a,std::int64_t b){if((b>0&&a>INT64_MAX-b)||(b<0&&a<INT64_MIN-b))throw "integer overflow";return a+b;}
 static std::int64_t sub(std::int64_t a,std::int64_t b){if((b<0&&a>INT64_MAX+b)||(b>0&&a<INT64_MIN+b))throw "integer overflow";return a-b;}
 static std::int64_t mul(std::int64_t a,std::int64_t b){if(a==0||b==0)return 0;if((a==-1&&b==INT64_MIN)||(b==-1&&a==INT64_MIN))throw "integer overflow";if(a>0?(b>0?a>INT64_MAX/b:b<INT64_MIN/a):(b>0?a<INT64_MIN/b:a<INT64_MAX/b))throw "integer overflow";return a*b;}
 void skip(){while(pos_<s_.size()&&std::isspace(static_cast<unsigned char>(s_[pos_])))++pos_;}
 bool take(char c){skip();if(pos_<s_.size()&&s_[pos_]==c){++pos_;return true;}return false;}
 bool takeDivide(){skip();if(pos_<s_.size()&&s_[pos_]=='/'){++pos_;return true;}if(s_.substr(pos_,2)=="\xC3\xB7"){pos_+=2;return true;}return false;}
 std::int64_t expression(){auto v=term();for(;;){if(take('+'))v=add(v,term());else if(take('-'))v=sub(v,term());else return v;}}
 std::int64_t term(){auto v=factor();for(;;){if(take('*'))v=mul(v,factor());else if(takeDivide()){auto d=factor();if(d==0)throw "division by zero";if(v==INT64_MIN&&d==-1)throw "integer overflow";v/=d;}else return v;}}
 std::int64_t factor(){skip();if(take('+'))return factor();if(take('-')){auto v=factor();if(v==INT64_MIN)throw "integer overflow";return -v;}if(take('(')){auto v=expression();if(!take(')'))throw "missing closing parenthesis";return v;}skip();if(pos_>=s_.size()||!std::isdigit(static_cast<unsigned char>(s_[pos_])))throw "expected number";std::int64_t v=0;while(pos_<s_.size()&&std::isdigit(static_cast<unsigned char>(s_[pos_]))){int digit=s_[pos_++]-'0';if(v>(INT64_MAX-digit)/10)throw "integer overflow";v=v*10+digit;}return v;}
 std::string_view s_;std::size_t pos_{};
};
std::string trim(std::string_view s){std::size_t a=0,b=s.size();while(a<b&&std::isspace(static_cast<unsigned char>(s[a])))++a;while(b>a&&std::isspace(static_cast<unsigned char>(s[b-1])))--b;return std::string(s.substr(a,b-a));}
bool has(std::string_view source,std::string_view token){return source.find(token)!=std::string_view::npos;}
}
ArithmeticResult ArithmeticEngine::evaluate(std::string_view e)const{return Parser(e).run();}
std::string SentenceEngine::compose(std::string_view question,std::int64_t answer)const{return "Hasil pengiraan bagi "+trim(question)+" ialah "+std::to_string(answer)+".";}
bool GailRuntime::load(std::string_view source,std::string* error){
 loaded_=false;add_=subtract_=multiply_=divide_=false;
 const bool envelope=has(source,"@start")&&has(source,"@ai")&&has(source,"mode native;")&&has(source,"precision int8;")&&has(source,"code_end;");
 const bool sentence=has(source,"answer + sentence")&&has(source,"sentence by_ai();");
 if(!envelope){if(error)*error="GAIL requires @start, @ai, native mode, int8 precision and code_end;";return false;}
 if(!sentence){if(error)*error="GAIL requires answer + sentence { sentence by_ai(); }";return false;}
 add_=has(source,"quest : have +")&&has(source,"mechanism : plus number with number");
 subtract_=has(source,"quest : have -")&&has(source,"mechanism : subtract number from number");
 multiply_=has(source,"quest : have *")&&has(source,"mechanism : multiply number with number");
 divide_=has(source,"quest : have ÷")&&has(source,"mechanism : divide number by number");
 if(!(add_||subtract_||multiply_||divide_)){if(error)*error="GAIL declares no supported arithmetic mechanisms";return false;}
 loaded_=true;if(error)error->clear();return true;
}
GailResult GailRuntime::ask(std::string_view question)const{
 if(!loaded_)return {false,0,{},"GAIL program is not loaded"};
 const bool usesDivide=question.find('/')!=std::string_view::npos||question.find("÷")!=std::string_view::npos;
 if((question.find('+')!=std::string_view::npos&&!add_)||(question.find('-')!=std::string_view::npos&&!subtract_)||(question.find('*')!=std::string_view::npos&&!multiply_)||(usesDivide&&!divide_))
  return {false,0,{},"the requested operation is not declared in this GAIL program"};
 auto r=arithmetic_.evaluate(question);
 if(!r.ok)return {false,0,{},r.error};
 return {true,r.value,sentence_.compose(question,r.value),{}};
}
} // namespace gravity::ai
