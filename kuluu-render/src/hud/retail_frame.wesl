import bevy_ui_render::ui_vertex_output::UiVertexOutput;

struct FramePart {
    uv: vec4<f32>,
    native_size: vec4<f32>,
    colors: array<vec4<f32>, 4>,
    blend: vec4<u32>,
}

struct FrameUniform {
    metrics: vec4<f32>,
    logical_size: vec4<f32>,
    parts: array<FramePart, 10>,
}

@group(1) @binding(0) var<uniform> frame: FrameUniform;
@group(1) @binding(1) var background: texture_2d<f32>;
@group(1) @binding(2) var background_sampler: sampler;
@group(1) @binding(3) var corner: texture_2d<f32>;
@group(1) @binding(4) var corner_sampler: sampler;
@group(1) @binding(5) var horizontal: texture_2d<f32>;
@group(1) @binding(6) var horizontal_sampler: sampler;
@group(1) @binding(7) var vertical: texture_2d<f32>;
@group(1) @binding(8) var vertical_sampler: sampler;

fn part_rect(index: u32, size: vec2<f32>) -> vec4<f32> {
    let c = frame.metrics.xy;
    let between = max(size - 2.0 * c, vec2<f32>(0.0));
    switch index {
        case 1u: { return vec4<f32>(0.0, 0.0, c); }
        case 2u: { return vec4<f32>(c.x, 0.0, between.x, frame.metrics.z); }
        case 3u: { return vec4<f32>(size.x - c.x, 0.0, c); }
        case 4u: { return vec4<f32>(0.0, c.y, frame.metrics.w, between.y); }
        case 5u: { return vec4<f32>(size.x - frame.metrics.w, c.y, frame.metrics.w, between.y); }
        case 6u: { return vec4<f32>(0.0, size.y - c.y, c); }
        case 7u: { return vec4<f32>(c.x, size.y - frame.metrics.z, between.x, frame.metrics.z); }
        case 8u: { return vec4<f32>(size - c, c); }
        default: { return vec4<f32>(vec2<f32>(0.0), size); }
    }
}

fn vertex_color(part: FramePart, uv: vec2<f32>) -> vec4<f32> {
    if uv.x + uv.y <= 1.0 {
        return part.colors[0] * (1.0 - uv.x - uv.y) + part.colors[1] * uv.x + part.colors[2] * uv.y;
    }
    return part.colors[3] * (uv.x + uv.y - 1.0) + part.colors[1] * (1.0 - uv.y) + part.colors[2] * (1.0 - uv.x);
}

fn sample_part(index: u32, uv: vec2<f32>) -> vec4<f32> {
    switch index {
        case 0u, 9u: { return textureSampleLevel(background, background_sampler, uv / vec2<f32>(textureDimensions(background)), 0.0); }
        case 2u, 7u: { return textureSampleLevel(horizontal, horizontal_sampler, uv / vec2<f32>(textureDimensions(horizontal)), 0.0); }
        case 4u, 5u: { return textureSampleLevel(vertical, vertical_sampler, uv / vec2<f32>(textureDimensions(vertical)), 0.0); }
        default: { return textureSampleLevel(corner, corner_sampler, uv / vec2<f32>(textureDimensions(corner)), 0.0); }
    }
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let size = frame.logical_size.xy;
    let point = in.uv * size;
    var result = vec4<f32>(0.0);
    for (var index = 0u; index < 9u; index += 1u) {
        let part = frame.parts[index];
        let rect = part_rect(index, size);
        let offset = point - rect.xy;
        if any(offset < vec2<f32>(0.0)) || any(offset >= rect.zw) {
            continue;
        }
        let sample_uv = part.uv.xy + offset * part.uv.zw / part.native_size.xy;
        var color = sample_part(index, sample_uv) * vertex_color(part, offset / rect.zw);
        color.a = min(color.a * part.native_size.z, 1.0);
        if part.blend.x == 0u {
            result = vec4<f32>(color.rgb, 1.0);
        } else if part.blend.z == 2u {
            result = vec4<f32>(result.rgb + color.rgb * color.a, result.a);
        } else {
            result = vec4<f32>(color.rgb * color.a + result.rgb * (1.0 - color.a), color.a + result.a * (1.0 - color.a));
        }
    }
    return result;
}
