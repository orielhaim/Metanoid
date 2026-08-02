#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::globals,
}

// Procedural atmospheric sky: vertical gradient + fbm nebula + stars + aurora.
// Bindings mirror the SkyMaterial Rust struct. Animation uses globals.time so
// the material never has to be mutated per frame.
struct SkyMaterial {
    top: vec4<f32>,
    bottom: vec4<f32>,
    nebula: vec4<f32>,
    params: vec4<f32>, // x = nebula_strength, y = star_density, z = aurora
    seed: vec4<f32>,   // x = motion scale, y = drift phase seed
};

@group(2) @binding(0) var<uniform> sky_top: vec4<f32>;
@group(2) @binding(1) var<uniform> sky_bottom: vec4<f32>;
@group(2) @binding(2) var<uniform> sky_nebula: vec4<f32>;
@group(2) @binding(3) var<uniform> sky_params: vec4<f32>;
@group(2) @binding(4) var<uniform> sky_seed: vec4<f32>;

fn hash21(p: vec2<f32>) -> f32 {
    var n = dot(p, vec2<f32>(127.1, 311.7));
    n = sin(n) * 43758.5453;
    return fract(n);
}

fn vnoise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let a = hash21(i);
    let b = hash21(i + vec2<f32>(1.0, 0.0));
    let c = hash21(i + vec2<f32>(0.0, 1.0));
    let d = hash21(i + vec2<f32>(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn fbm(p: vec2<f32>) -> f32 {
    var val = 0.0;
    var amp = 0.5;
    var freq = 1.0;
    for (var i = 0; i < 4; i = i + 1) {
        val += amp * vnoise(p * freq);
        amp *= 0.5;
        freq *= 2.0;
    }
    return val;
}

fn star_layer(uv: vec2<f32>, density: f32, scale: f32, t: f32, phase: f32) -> f32 {
    let cell = floor(uv * scale);
    let fcell = fract(uv * scale) - 0.5;
    let rnd = hash21(cell);
    let star = step(1.0 - density, rnd);
    let tw = 0.55 + 0.45 * sin(t + rnd * 6.2831 + phase);
    let dist = length(fcell);
    let glow = smoothstep(0.35, 0.0, dist) * star;
    return glow * tw;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Normalize world position to 0..1 over the stage.
    let uv = (in.world_position.xy + vec2<f32>(640.0, 360.0)) / vec2<f32>(1280.0, 720.0);
    let uv_y = clamp(uv.y, 0.0, 1.0);

    let t = globals.time * sky_seed.x; // 0 with reduced-motion
    let drift_x = sky_seed.y + t * 0.006;
    let drift_y = 0.31 + sky_seed.y * 0.5 + t * 0.003;

    // Vertical gradient: horizon (bottom) -> sky (top).
    var col = mix(sky_bottom.rgb, sky_top.rgb, smoothstep(0.15, 0.85, uv_y));

    // Slow drifting nebula.
    let neb_noise = fbm(vec2<f32>(uv.x * 2.5 + drift_x, uv_y * 2.0 + drift_y));
    let neb_mask = smoothstep(0.42, 0.95, neb_noise) * sky_params.x;
    col = mix(col, sky_nebula.rgb, neb_mask);

    // Aurora bands near the top.
    if (sky_params.z > 0.02) {
        let aur = vnoise(vec2<f32>(uv.x * 3.0 + drift_x, uv_y * 8.0));
        let band = smoothstep(0.55, 0.95, uv_y) * (1.0 - abs(uv_y - 0.78) * 6.0);
        let aurora_c = mix(sky_nebula.rgb, vec3<f32>(0.3, 1.0, 0.7), 0.6);
        col = mix(col, aurora_c, band * aur * sky_params.z * 0.6);
    }

    // Stars (two layers).
    if (sky_params.y > 0.02) {
        col += vec3<f32>(1.0, 1.0, 0.9) * star_layer(uv, sky_params.y * 0.5, 70.0, t * 0.5, drift_x) * 0.55;
        col += vec3<f32>(0.9, 0.95, 1.0) * star_layer(uv, sky_params.y * 0.35, 23.0, t * 0.35, drift_x * 2.0) * 0.8;
    }

    return vec4<f32>(col, 1.0);
}
