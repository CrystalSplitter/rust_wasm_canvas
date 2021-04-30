pub type M3x3 = Vec<f32>;
pub type M4x4 = Vec<f32>;

pub fn mult3x3(a: &[f32], b: &[f32]) -> M3x3 {
    vec![
            // Row 1
            a[0  ] * b[0  ] + a[1    ] * b[3  ] + a[2    ] * b[3*2  ],
            a[0  ] * b[1  ] + a[1    ] * b[3+1] + a[2    ] * b[3*2+1],
            a[0  ] * b[2  ] + a[1    ] * b[3+2] + a[2    ] * b[3*2+2],

            // Row 2
            a[3  ] * b[0  ] + a[3  +1] * b[3  ] + a[3+2  ] * b[3*2  ],
            a[3  ] * b[1  ] + a[3  +1] * b[3+1] + a[3+2  ] * b[3*2+1],
            a[3  ] * b[2  ] + a[3  +1] * b[3+2] + a[3+2  ] * b[3*2+2],

            // Row 3
            a[3*2] * b[0  ] + a[3*2+1] * b[3  ] + a[3*2+2] * b[3*2  ],
            a[3*2] * b[1  ] + a[3*2+1] * b[3+1] + a[3*2+2] * b[3*2+1],
            a[3*2] * b[2  ] + a[3*2+1] * b[3+2] + a[3*2+2] * b[3*2+2],
    ]
}

pub fn zero() -> M3x3 {
    (0..9).map(|_| 0.).collect()
}

pub fn translation(dx: f32, dy: f32) -> M3x3 {
    vec![
        1., 0., 0.,
        0., 1., 0.,
        dx, dy, 1.
    ]
}

/// Generate a 3x3 rotation matrix.
///
/// theta: Angle to rotate in radians.
pub fn rotation(thetaRad: f32) -> M3x3 {
    let c: f32 = thetaRad.cos();
    let s: f32 = thetaRad.sin();
    vec![
        c, -s,  0.,
        s,  c,  0.,
        0., 0., 1.
    ]
}

pub fn scale(dx: f32, dy: f32) -> M3x3 {
    vec![
        dx, 0., 0.,
        0., dy, 0.,
        0., 0., 1.
    ]
}

pub fn id() -> M3x3 {
    vec![
        1., 0., 0.,
        0., 1., 0.,
        0., 0., 1.
    ]
}

