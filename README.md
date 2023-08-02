# Shared Exponent Formats

**WIP**

| Name     | Bytes | Max value  | Signed | 0.01 Max Δ | 0.1 Max Δ  | 1.0 Max Δ  | 10.0 Max Δ | 100 Max Δ  | 1000 Max Δ |
| -------- | ----- | ---------- | ------ | ---------- | ---------- | ---------- | ---------- | ---------- | -----------|
| xyz8e5   | 4     | 65280      | true   | 0.00005267 | 0.00042162 | 0.00582401 | 0.05401088 | 0.43168700 | 3.45184112 |
| rgb9e5   | 4     | 65408      | false  | 0.00002636 | 0.00021118 | 0.00288069 | 0.02697610 | 0.21562232 | 1.72821546 |
| xyz9e2   | 4     | 1          | true   | 0.00021123 | 0.00021169 | 0.00169017 |
| 3x f16   | 6     | 65504      | true   | 0.00000653 | 0.00005261 | 0.00042066 | 0.00667667 | 0.05379717 | 0.43026507 |
| xyz13e6  | 6     | 4.29e9     | true   | 0.00000165 | 0.00001318 | 0.00017735 | 0.00168843 | 0.01350524 | 0.10790101 |
| xyz14e3  | 6     | 16         | true   | 0.00000659 | 0.00000659 | 0.00005271 | 0.00084312 |
| xyz18e7  | 8     | 1.85e19    | true   | 0.00000005 | 0.00000041 | 0.00000532 | 0.00005286 | 0.00042286 | 0.00338291 |
| 8unorm   | 3     | 1          | false  | 0.00339042 | 0.00338363 | 0.00338270 |

Max Δ is max distance from f32 input 3d coordinate found. Tested with 10,000,000 random coordinates per range.

- All formats reproduce 0.0 and 1.0 exactly.
- INF becomes MAX for the respective format.
- NAN becomes 0.0.
- rgb9e5 layout matches the common [GPU texture format](https://registry.khronos.org/OpenGL/extensions/EXT/EXT_texture_shared_exponent.txt)
- TODO WGSL encoding/decoding implementations.

X is input value random range. Y is distance from f32 input 3d coordinate:
![demo](avg_delta.PNG)
![demo](max_avg_delta.PNG)

Tested against f64 below, including compairson to 3x f32. Note the max representable value varies significantly: 
```
f16:     6.55e4
xyz13e6: 4.29e9
xyz18e7: 1.85e19
f32:     3.40e38
```
![demo](max_avg_delta_f64.PNG)