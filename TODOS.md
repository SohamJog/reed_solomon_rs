Optimization TODOS:
1. Replace GF256 with 252 bit prime field from LW
2. Figure out math stuff and see if you can just have 1 math library
3 .

Lambdaworks notes:
- First way to do this is by just replacing GFVal with LargeField, but then we don't take advantage of the large prime field
- The second way to do this is by replacing GFVals with largefield, i.e serializing the largefield into a byte array and then using that as the value
