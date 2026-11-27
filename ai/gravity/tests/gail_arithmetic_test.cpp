#include "gravity/ai/gail.hpp"
#include <cassert>
#include <iostream>
int main(){
 using namespace gravity::ai;
 ArithmeticEngine engine;
 assert(engine.evaluate("1 + 1").value==2);
 assert(engine.evaluate("9 - 14").value==-5);
 assert(engine.evaluate("6 * 7").value==42);
 assert(engine.evaluate("84 / 7").value==12);
 assert(engine.evaluate("84 ÷ 7").value==12);
 assert(engine.evaluate("(2 + 3) * 4").value==20);
 assert(!engine.evaluate("1 / 0").ok);
 assert(!engine.evaluate("9223372036854775807 + 1").ok);
 GailRuntime runtime;std::string error;
 const char* program=R"GAIL(@start { mode native; precision int8; }
@ai {
quest : have *
mechanism : multiply number with number
quest : have +
mechanism : plus number with number
quest : have -
mechanism : subtract number from number
quest : have ÷
mechanism : divide number by number
answer + sentence { sentence by_ai(); }
}
code_end;)GAIL";
 assert(runtime.load(program,&error));
 const auto result=runtime.ask("1 + 1");
 assert(result.ok&&result.answer==2);
 assert(result.sentence=="Hasil pengiraan bagi 1 + 1 ialah 2.");
 assert(runtime.ask("6 * 7").answer==42);
 assert(runtime.ask("20 ÷ 5").answer==4);
 assert(!runtime.ask("2 / 0").ok);
 GailRuntime addOnly;
 assert(addOnly.load("@start { mode native; precision int8; } @ai { quest : have + mechanism : plus number with number answer + sentence { sentence by_ai(); } } code_end;",&error));
 assert(!addOnly.ask("2 * 3").ok);
 assert(!GailRuntime{}.load("@start {}",&error));
 std::cout<<"GAIL runtime + arithmetic + sentence: PASS
";
}
