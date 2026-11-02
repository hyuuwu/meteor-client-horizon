fn fullbright(renderer: &mut Renderer) {
    renderer.fullbright |= meteor_horizon::config().fullbright;
}
