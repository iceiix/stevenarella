
use render::glsl;

pub fn add_shaders(reg: &mut glsl::Registry) {
	reg.register("lookup_texture", include_str!("shaders/lookup_texture.glsl"));
	reg.register("get_light", include_str!("shaders/get_light.glsl"));

	reg.register("ui_vertex", include_str!("shaders/ui_vertex.glsl"));
	reg.register("ui_frag", include_str!("shaders/ui_frag.glsl"));
}