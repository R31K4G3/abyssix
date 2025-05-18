# abyssix
A procedural structured programming language.

# Examples
## Hello world
```
func main {
  params 0;
  alloc 0;

  putc 72;
  putc 101;
  putc 108;
  putc 108;
  putc 111;
  putc 32;
  putc 119;
  putc 111;
  putc 114;
  putc 108;
  putc 100;
  putc 10;
}
```
## FizzBuzz
```
func main {
  params 0;
  alloc 1;

  set.0 = 1;
  while  get.0  int.<=  100: {
    if  get.0  int.%  15  int.==  0: {
      putc 70; putc 105; putc 122; putc 122; putc 66; putc 117; putc 122; putc 122;
    } else if  get.0  int.%  3  int.==  0: {
      putc 70; putc 105; putc 122; putc 122;
    } else if  get.0  int.%  5  int.==  0: {
      putc 66; putc 117; putc 122; putc 122;
    } else {
      printInt(get.0);
    }
    putc 10;
    set.0  =  1  int.+  get.0;
  }
}

func printInt {
  params 1;

  alloc 24;
  // 0: target to print
  // 1-21: char buffer
  // 22: buffer length
  // 23: iteration variable

  set.0  =  param.0;
  if  get.0  int.==  0: {
    putc 48; // print '0'
    return 0;
  } else if  get.0  int.<  0: {
    set.0  =  int.-  get.0;
    putc 45; // print '-'
  } else {
    // do nothing
  }
  while  get.0  int.>  0: {
    set[1  int.+  get.22]  =  get.0  int.%  10;
    set.0  =  get.0  int./  10;
    set.22  =  get.22  int.+  1;
  }
  set.23  =  get.22  int.-  1;
  while  get.23  int.>=  0: {
    putc(48  int.+  get[1  int.+  get.23]);
    set.23  =  get.23  int.-  1;
  }
}
```

# Syntaxes
## Function definition
```
func functionName {
  params 0;
  alloc 0;

  statement0;
  statement1;
  statement2;
  ...
}
```
A function name must match the regex `/^[A-Za-z_]\w+$/`.  
The first statement must be a parameters declaration.  
It specify the number of parameters.  
e.g. `params 30;`  
The second statement must be a stack allocation statement.  
It specify the number of local variables.  
e.g. `alloc 20;`  

## param: access to parameters
```
param.0
param.1
```

## if: selection
```
if condition: whenTrue;
else whenFalse;
```
```
if condition: {
  whenTrue;
  ...
} else {
  whenFalse;
  ...
}
```
The `else` branch isn't able to be omitted.  
Not only `if` and `else` but also `else if` is allowed.  

## while: repeatition
```
while condition: body;
```
```
while condition: {
  body;
  ...
}
```
`while` statement repeats while the condition expression is true.

## get, set: variable access
```
get.0
get.1
get[0]
get[1]
set.0 = value0;
set.1 = value1;
set[0] = value2;
set[1] = value3;
```
Assign a value to a local variable.  
The below example means: copies the value of variable 0 to variable 1.
```
set.1 = get.0;
```

## getc, putc: standard input/output
```
set.0 = getc;
putc 72;  // H
putc 101; // e
putc 108; // l
putc 108; // l
putc 111; // o
```
`getc` reads a byte from standard input.  
`putc` writes a byte to standard output.  
When the value passed to `putc` is greater than 255, the remainder of it divided by 256 is written.

## return: exit from a function
```
return 1;
```
`return` statement sets the return value of the function call and exit.  

## funcName(): function call
```
funcName(1, 2, 3);
```
A name of a callee must match the regex `/^[A-Za-z_]\w+$/`.  
The number of arguments must be equal to the number of parameters.

## Arithmetic operators
```
1 int.+ 1
1 int.- 1
1 int.* 1
1 int./ 1
1 int.% 1
int.- 1
1.0 float.+ 1.0
1.0 float.- 1.0
1.0 float.* 1.0
1.0 float./ 1.0
1.0 float.% 1.0
float.- 1
```
The addition, subtraction, multiplication, division, remainder and negation is supported.

## Comparison operators
```
1 int.== 1
1 int.!= 1
1 int.<  1
1 int.<= 1
1 int.>  1
1 int.>= 1
1.0 float.== 1.0
1.0 float.!= 1.0
1.0 float.<  1.0
1.0 float.<= 1.0
1.0 float.>  1.0
1.0 float.>= 1.0
```
The equality, inequality, less-than, less-than-or-equal-to, greater-than and greater-than-or-equal-to is supported.  
The operators return 0 or 1.

## Bitwise operators
```
1 & 1
1 | 1
1 ^ 1
~1
```
Bitwise and, bitwise or, bitwise xor, bitwise not  
is supported.

## Bit-shift operators
```
1 << 1
1 >> 1
```
Signed left shift and signed right shift is supported.

## Logical not operator
```
!1
```
The operator returns 0 when the passed value is not 0.
The operator returns 1 when the passed value is 0.

## Comment
```
// This is comment.
// This doesn't affect the behavior.
```
