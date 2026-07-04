# TİLK Dilinin Resmî Sözdizimi Kuralları (EBNF)

TİLK dilinin dilbilgisi yapısı Extended Backus-Naur Form (EBNF) standardında aşağıda resmîleştirilmiştir.

## EBNF Şeması
```ebnf
Program         ::= Statement*
Statement       ::= VarDecl | Assignment | IfStatement | WhileStatement | ForStatement | FnDeclaration | ReturnStatement | ExprStatement
VarDecl         ::= Identifier "=" Expr ";"
Assignment      ::= Identifier "=" Expr ";"
ReturnStatement ::= "döndür" Expr? ";"
IfStatement     ::= Expr ("ise" | "se") Block ( "değilse" Block )?
WhileStatement  ::= Expr "iken" Block
ForStatement    ::= Identifier "," Expr ("dan" | "den" | "tan" | "ten") Expr ("e" | "a" | "ye" | "ya") "dek" ("artarak" | "azalarak")? Block
FnDeclaration   ::= "işlev" Identifier "(" ParamList? ")" Block
Block           ::= "{" Statement* "}"
Expr            ::= LogicalOrExpr
LogicalOrExpr   ::= LogicalAndExpr ( "veya" LogicalAndExpr )*
LogicalAndExpr  ::= EqualityExpr ( "ve" EqualityExpr )*
EqualityExpr    ::= ComparisonExpr ( ( "==" | "!=" ) ComparisonExpr )*
ComparisonExpr  ::= Term ( ( "<" | ">" | "<=" | ">=" ) Term )*
Term            ::= Factor ( ( "+" | "-" ) Factor )*
Factor          ::= Primary ( ( "*" | "/" | "%" ) Primary )*
Primary         ::= Identifier | Number | String | Boolean | "boş" | CallExpr | "(" Expr ")"
CallExpr        ::= Identifier "(" ArgList? ")"
ParamList       ::= Identifier ( "," Identifier )*
ArgList         ::= Expr ( "," Expr )*
Boolean         ::= "doğru" | "yanlış"
Number          ::= [0-9]+ ( "." [0-9]+ )?
String          ::= '"' [^"\\]* '"'
Identifier      ::= [a-zA-ZçğıişöüÇĞIİŞÖÜ_] [a-zA-Z0-9çğıişöüÇĞIİŞÖÜ_]*
```
