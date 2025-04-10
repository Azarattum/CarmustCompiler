# Carmust Compiler

Carmust is a C to ARM64 compiler written in Rust. This project is a prototype the primary goal of which was learning Rust. As it stands, the compiler currently supports a subset of C and lacks any real optimizations.

## Showcase

An example program that it can compile:
```c
typedef int i32;
typedef float f32;

i32 globalData = 42;
f32 floatEncoding = 32.0;

int main() {
  // Expressions:
  i32 unaryOperators = 43 + -globalData - 11;
  i32 wildExpressions = 1 << !!!!!!-2 > (6 & 1 ^ (3 % 4)) & 255 == 1;
  i32 booleanAndBinaryOps = 7 || (1 + !5 == 2 - 3) && 1;
  f32 floatingPointMath = (2.5 * 2 + floatEncoding) / 2;

  short shortsAreAlsoAllowed = 1;
  long longsAreSupportedAsWell = 123456;

  // Assignment works
  unaryOperators = unaryOperators + wildExpressions + ' ';

  // Empty statements
  ;
  ;

  // For loops
  for (int i = 0; i < 5 + 1; i = i + 1) {
    unaryOperators = unaryOperators + 1;
  }

  // Expressions in the return statement
  return unaryOperators + floatingPointMath + booleanAndBinaryOps - 5;
}
```

Let's look closely at the compilation process with a more simple program:
```c
float global = 42;

int main() {
  int local = 1337;
  return local % 255 + global;
}
```

At first the following tokens are extracted from the source code: 
<pre>
<span style="color:cyan">Tokens</span>: [Keyword("float"), Identifier("global"), Symbol("="), Data(Integer(42), "42"), Symbol(";"), Keyword("int"), Identifier("main"), Symbol("("), Symbol(")"), Symbol("{"), Keyword("int"), Identifier("local"), Symbol("="), Data(Integer(1337), "1337"), Symbol(";"), Keyword("return"), Identifier("local"), Symbol("%"), Data(Integer(255), "255"), Symbol("+"), Identifier("global"), Symbol(";"), Symbol("}")]
</pre>

Then an abstract syntax tree is generated:
<pre>
<span style="color:magenta">AST</span>: [Variable(Variable { datatype: Type(Compound(Float, 1)), name: "global", assignment: Some(Assignment { name: "global", value: Value(Data(Integer(42))) }) }), Function(Function { datatype: Type(Compound(Int, 1)), name: "main", body: [Variable(Variable { datatype: Type(Compound(Int, 1)), name: "local", assignment: Some(Assignment { name: "local", value: Value(Data(Integer(1337))) }) }), Return(Binary { op: Addition, lhs: Binary { op: Remainder, lhs: Value(Pointer(Identifier("local"))), rhs: Value(Data(Integer(255))) }, rhs: Value(Pointer(Identifier("global"))) })] })]
</pre>

Which can be compiled into an intermediate representation:
<pre>
<span style="color:lime">IR</span>:
globals:
  global_0 = 42

main:
  0) Mov @ 1337
  1) Str 'local_1' @0
  2) Ldr @ 'local_1'
  3) Mov @ 255
  4) Div @2 @3
  5) Mul @3 @4
  6) Sub @2 @5
  7) Ldg @ 'global_0'
  8) SCvtF @6 
  9) Add @8 @7
 10) FCvtZS @9 
 11) Ret @10
</pre>

And finally compiled to ARM64 assembly:
<pre>
<span style="color:yellow">ASM</span>:
.global main
global_0:
  .word 1109917696
main:
  sub sp, sp, 16
  mov w0, 1337
  str w0, [sp, 12]
  ldr w0, [sp, 12]
  mov w1, 255
  sdiv w2, w0, w1
  mul w1, w1, w2
  sub w0, w0, w1
  adrp x3, global_0@GOTPAGE
  ldr x3, [x3, global_0@GOTPAGEOFF]
  ldr s1, [x3]
  scvtf s0, w0
  fadd s0, s0, s1
  fcvtzs w0, s0
  add sp, sp, 16
  ret
</pre>

As you can see, it supports type inference, global/local variables, simple `for` loops, arbitrary expressions (with bitwise and boolean operators) and `return` statement which allows us to observe the result of the program:
<pre>
<span style="color:dodgerblue">Execution Result</span>: 104
</pre>