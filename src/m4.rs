/// 4x4 Identity Matrix.
pub fn id() -> M4x4 {
    let n: usize = 4;
    let mut v = Vec::with_capacity(n);
    for i in 0..n {
        for j in 0..n {
            v.push(if i == j { 1. } else { 0. });
        }
    }
    v
}

pub fn scale(dx: f32, dy: f32) -> M3x3 {
    vec![dx, 0., 0., 0., dy, 0., 0., 0., 1.]
}
