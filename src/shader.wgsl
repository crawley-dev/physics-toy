@group(0) @binding(0) var texture_sampler: sampler;
@group(0) @binding(1) var texture_data: texture_2d<f32>;
@group(0) @binding(2) var<uniform> time: f32; // Time data from cpu


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

    // var tex_coords = array<vec2<f32>, 6>(
    //     vec2<f32>(0.0, 1.0),   // Bottom-left
    //     vec2<f32>(1.0, 1.0),   // Bottom-right
    //     vec2<f32>(0.0, 0.0),   // Top-left
    //     vec2<f32>(1.0, 1.0),   // Bottom-right
    //     vec2<f32>(1.0, 0.0),   // Top-right
    //     vec2<f32>(0.0, 0.0)    // Top-left
    // );

    return vec4<f32>(positions[vertex_index], 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) fragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    let resolution = vec2<f32>(800.0, 600.0); // Update to the window resolution dynamically if needed

    // Normalize the fragment coordinates to [0, 1]
    let uv = fragCoord.xy / resolution; 

    // Sample the texture using normalized coordinates
    return textureSample(texture_data, texture_sampler, ); // This returns a vec4<f32>
}
    
// @fragment
// fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
//     let tex_coords = frag_coord.xy / vec2<f32>(800.0, 600.0);  // Assuming window size
//     return textureSample(texture_data, texture_sampler, tex_coords);
// }

// @fragment
// fn fs_main(@builtin(position) fragCoord: vec4<f32>) -> @location(0) vec4<f32> {
//     // Set the resolution here (adjust as needed)
//     let resolution = vec2<f32>(800.0, 600.0);
//     let texture_resolution = vec2<f32>(80.0,60.0); 

//     let uv = fragCoord.xy / resolution;       // Get normalized coordinates (0 to 1)
//     let uv_scaled = uv * (texture_resolution / resolution);

//     // Use pixel coordinates to generate a pseudo-random color
//     let pixel_coord = fragCoord.xy;
    
//     // Simple hash function to create a pseudo-random value based on coordinates
//     let r = fract(sin(dot(pixel_coord, vec2<f32>(12.9898, 78.233))) * 43758.5453 + time);
//     let g = fract(sin(dot(pixel_coord, vec2<f32>(63.9898, 78.233))) * 43758.5453 + time);
//     let b = fract(sin(dot(pixel_coord, vec2<f32>(30.9898, 78.233))) * 43758.5453 + time);
    
//     // Construct a color vector from the random values
//     // let color = vec3<f32>(r, g, b);
//     let color = textureSample(texture_data, texture_sampler, uv_scaled);

//     return color;
// }
