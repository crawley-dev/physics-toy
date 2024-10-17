struct Uniforms {
    _padding: f32,
    time: f32,
    texture_size: vec2<f32>,
}

@group(0) @binding(0) var texture_sampler: sampler;
@group(0) @binding(1) var texture_data: texture_2d<f32>;
@group(0) @binding(2) var<uniform> uniforms: Uniforms;


@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0), // Bottom-left
        vec2<f32>( 1.0, -1.0), // Bottom-right
        vec2<f32>(-1.0,  1.0), // Top-left
        vec2<f32>( 1.0, -1.0), // Bottom-right
        vec2<f32>( 1.0,  1.0), // Top-right
        vec2<f32>(-1.0,  1.0)  // Top-left
    );
    return vec4<f32>(positions[vertex_index], 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) pixelCoord: vec4<f32>) -> @location(0) vec4<f32> {
    // Normalise coordinate to [0, 1] based on position in texture
    let uv = pixelCoord.xy / uniforms.texture_size; 

    let colour = random_colour(uv, uniforms.time);
    // let colour = textureSample(texture_data, texture_sampler, uv).xyz; // gets the color from the texture (44,44,44)
    return vec4<f32>(colour, 1.0);
}

// A simple hash function with global uniqueness based on position and a global seed
fn hash(uv: vec2<f32>, seed: f32) -> f32 {
    let dot_product = dot(uv, vec2<f32>(12.9898, 78.233)) + seed;
    return fract(sin(dot_product) * 43758.5453);
}

// Converts a float value in the range [0, 1] to a random color
fn random_colour(uv: vec2<f32>, seed: f32) -> vec3<f32> {
    let random_value = hash(uv, seed);
    
    // Generate random color based on the hashed value
    let r = fract(random_value * 1.0);
    let g = fract(random_value * 2.0);
    let b = fract(random_value * 3.0);
    
    return vec3<f32>(r, g, b);
}