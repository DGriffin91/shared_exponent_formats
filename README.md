# Shared Exponent Formats

Implementations provided in rust, glsl, and wgsl.

| Name      | Bytes | Signed | Max Val | Epsilon  |
|-----------|-------|--------|---------|----------|
| 3x 8unorm | 3     | false  | 1       | 3.92e-3  |
| xyz8e5    | 4     | true   | 65280   | 1.19e-7  |
| rgb9e5    | 4     | false  | 65408   | 5.96e-8  |
| 3x f16    | 6     | true   | 65504   | 9.77e-4  |
| xyz13e6   | 6     | true   | 4.29e9  | 5.68e-14 |
| xyz18e7   | 8     | true   | 1.84e19 | 4.14e-25 |

| Name      | 0.01 Max Δ | 0.1 Max Δ | 1.0 Max Δ | 10.0 Max Δ | 100 Max Δ | 1000 Max Δ |
|-----------|------------|-----------|-----------|------------|-----------|------------|
| 3x 8unorm | 3.39e-3    | 3.39e-3   | 3.39e-3   |            |           |            |
| rgb8e5    | 5.28e-5    | 4.23e-4   | 5.82e-3   | 5.40e-2    | 4.33e-1   | 3.46       |
| rgb9e5    | 2.64e-5    | 2.11e-4   | 2.91e-3   | 2.70e-2    | 2.16e-1   | 1.73       |
| 3x f16    | 6.58e-6    | 5.27e-5   | 4.22e-4   | 6.74e-3    | 5.39e-2   | 4.32e-1    |
| rgb13e6   | 1.65e-6    | 1.32e-5   | 1.82e-4   | 1.69e-3    | 1.36e-2   | 1.09e-1    |
| rgb18e7   | 5.01e-8    | 4.13e-7   | 5.65e-6   | 5.29e-5    | 4.23e-4   | 3.39e-3    |

Max Δ is max distance from f32 input 3d coordinate found. Tested with 1.0e8 random coordinates per range.

- All formats reproduce 0.0 and 1.0 exactly.
- INF becomes MAX for the respective format.
- NAN becomes 0.0. (rust impl only)
- rgb9e5 layout matches the common [GPU texture format](https://registry.khronos.org/OpenGL/extensions/EXT/EXT_texture_shared_exponent.txt)

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