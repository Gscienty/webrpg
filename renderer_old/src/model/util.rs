pub fn animation_ratio(anim: &str) -> Option<f32> {
    let (_, ratio) = anim.rsplit_once('@')?;
    let ratio = ratio.parse::<f32>().ok()?;

    ratio.is_finite().then(|| ratio.clamp(0., 1.))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_last_finite_animation_ratio_and_clamps_it() {
        assert_eq!(animation_ratio("Stand@0.25"), Some(0.25));
        assert_eq!(animation_ratio("Stand@ignored@2"), Some(1.));
        assert_eq!(animation_ratio("Stand@-1"), Some(0.));
        assert_eq!(animation_ratio("Stand@NaN"), None);
        assert_eq!(animation_ratio("Stand@0.5x"), None);
        assert_eq!(animation_ratio("Stand"), None);
    }
}
