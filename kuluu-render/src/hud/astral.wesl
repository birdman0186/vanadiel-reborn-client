import bevy_ui_render::ui_vertex_output::UiVertexOutput;

@group(1) @binding(0) var<uniform> effect: vec4<f32>;

fn ink(distance: f32, width: f32) -> f32 {
    return 1.0 - smoothstep(width - 0.45, width + 0.45, abs(distance));
}

fn star(p: vec2<f32>, radius: f32) -> f32 {
    let a = abs(p);
    return max(ink(a.x + 3.0 * a.y, radius), ink(3.0 * a.x + a.y, radius));
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let size = in.size;
    let p = in.uv * size;
    let q = min(p, size - p);
    let silver = vec3<f32>(0.39, 0.49, 0.63);
    let gold = vec3<f32>(0.88, 0.63, 0.28);
    let pearl = vec3<f32>(1.0, 0.90, 0.66);
    if effect.x > 0.5 {
        let edge = min(min(q.x - max(0.0, 4.0 - q.y), q.y), q.x);
        let bevel = 1.0 - smoothstep(0.0, 0.65, in.uv.y);
        let focus = effect.x > 1.5;
        var c = mix(vec3<f32>(0.008,0.017,0.050),vec3<f32>(0.072,0.13,0.26),bevel);
        if focus { c = mix(vec3<f32>(0.018,0.055,0.15),vec3<f32>(0.09,0.24,0.49),bevel); }
        c = mix(c, select(silver * 0.55, mix(gold, silver, in.uv.x), focus), ink(edge-1.0,0.55));
        if focus {
            let t = effect.y;
            let sweep = (1.0 - pow(1.0 - t, 3.0)) * size.x;
            c = mix(c, pearl, ink(p.y-1.3, 0.7) * (1.0-smoothstep(0.0, 13.0, abs(p.x-sweep))) * sin(t*3.14159));
            let arrow = abs(p.y - 0.5 * size.y) * 1.25;
            let tip = 8.0 - p.x - arrow;
            if p.x > 1.5 && tip > 0. && p.x + arrow * 0.5 > 3.5 {
                c = select(gold, pearl, p.y < size.y * 0.5);
            }
        }
        return vec4<f32>(c, smoothstep(-0.5,0.5,edge) * 0.97);
    }
    let edge = min(q.x, q.y);
    let deep = vec3<f32>(0.004,0.010,0.029);
    let cobalt = vec3<f32>(0.025,0.059,0.132);
    var c = mix(deep, cobalt, 0.18 + 0.67 * (1.0-in.uv.y) * (1.0-in.uv.x*0.65));
    c = mix(c, silver, ink(edge-0.8,0.52)*0.87);
    c = mix(c, silver, ink(edge-2.7,0.42)*0.32);
    let corner = 1.0-smoothstep(4.0,16.0,max(q.x,q.y));
    c = mix(c, gold, corner*ink(edge-1.5,0.73));
    c = mix(c, pearl, star(q-vec2<f32>(4.0),1.65) * corner);
    if effect.z < 1.0 {
        let t = effect.z;
        let pos = vec2<f32>(size.x - p.x, p.y);
        let fade = smoothstep(0.0,0.12,t)*(1.0-smoothstep(0.62,1.0,t));
        let a = star(pos-vec2<f32>(30.0,10.0), 2.8) * smoothstep(0.0,0.17,t);
        let b = star(pos-vec2<f32>(19.0,6.7), 2.2) * smoothstep(0.17,0.34,t);
        let d = star(pos-vec2<f32>(9.0,5.0), 2.6) * smoothstep(0.34,0.51,t);
        c = mix(c, pearl, max(a,max(b,d))*fade);
    }
    return vec4<f32>(c, mix(0.90, 0.98, 1.0-smoothstep(1.0,4.0,edge)));
}
