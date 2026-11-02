fn fullbright(renderer: &mut Renderer) {
    if let Some(enabled) = crate::mesher::meteor_horizon::state().fullbright_override() {
        renderer.fullbright = enabled;
    }
}
