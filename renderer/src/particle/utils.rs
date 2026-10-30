use shared::math3;

pub(super) fn generate_random_direction(latitude: f32) -> math3::Vector3 {
    let theta = rand::random::<f32>() * std::f32::consts::TAU;
    let phi = rand::random::<f32>() * latitude;
    let (sin_phi, cos_phi) = phi.sin_cos();
    let (sin_theta, cos_theta) = theta.sin_cos();

    (sin_phi * cos_theta, sin_phi * sin_theta, cos_phi).into()
}
