pub fn minimap_point_for_world(
    world_center: shared::math3::Vector2,
    map_size: shared::math3::Vector2,
    world: shared::math3::Vector3,
    screen: shared::math3::Rect,
) -> Option<shared::math3::Vector2> {
    if map_size[0] <= 0. || map_size[1] <= 0. {
        return None;
    }

    let nx = ((world[0] - world_center[0]) / map_size[0]).clamp(0., 1.);
    let ny = ((world[1] - world_center[1]) / map_size[1]).clamp(0., 1.);

    Some((screen.x + nx * screen.w, screen.y + (1. - ny) * screen.h).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_world_coordinates_to_the_minimap() {
        let screen = shared::math3::Rect::new(10., 20., 200., 100.);
        let point = minimap_point_for_world(
            (100., 200.).into(),
            (400., 200.).into(),
            (300., 250., 0.).into(),
            screen,
        );
        assert_eq!(point, Some((110., 95.).into()));

        let clamped = minimap_point_for_world(
            (100., 200.).into(),
            (400., 200.).into(),
            (0., 500., 0.).into(),
            screen,
        );
        assert_eq!(clamped, Some((10., 20.).into()));

        assert_eq!(
            minimap_point_for_world(
                (0., 0.).into(),
                (0., 200.).into(),
                (0., 0., 0.).into(),
                screen,
            ),
            None
        );
    }
}
