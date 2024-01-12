use crate::Command;

pub(crate) fn calculate_ellipse_parameters(
    x0: f32,
    y0: f32, // Start point
    x: f32,
    y: f32, // End point
    mut rx: f32,
    mut ry: f32, // Radii
    phi: f32,    // X-axis rotation in degrees
    large_arc_flag: bool,
    sweep_flag: bool,
) -> Option<(f32, f32, f32, f32)> {
    // Ensure radii are positive
    rx = rx.abs();
    ry = ry.abs();

    // If radii are zero, return None (invalid arc)
    if rx == 0.0 || ry == 0.0 {
        return None;
    }

    // Convert rotation angle from degrees to radians
    let phi_rad = phi.to_radians();
    let cos_phi = phi_rad.cos();
    let sin_phi = phi_rad.sin();

    // Step 1: Compute (x1', y1') - the transformed start point
    let dx2 = (x0 - x) / 2.0;
    let dy2 = (y0 - y) / 2.0;
    let x1p = cos_phi * dx2 + sin_phi * dy2;
    let y1p = -sin_phi * dx2 + cos_phi * dy2;

    // Ensure radii are large enough
    let x1p_sq = x1p * x1p;
    let y1p_sq = y1p * y1p;
    let rx_sq = rx * rx;
    let ry_sq = ry * ry;

    // Correct out of range radii
    let radii_check = x1p_sq / rx_sq + y1p_sq / ry_sq;
    if radii_check > 1.0 {
        rx *= radii_check.sqrt();
        ry *= radii_check.sqrt();
    }

    // Step 2: Compute (cx', cy') - the transformed center point
    let sign = if large_arc_flag == sweep_flag {
        -1.0
    } else {
        1.0
    };
    let sq = ((rx_sq * ry_sq) - (rx_sq * y1p_sq) - (ry_sq * x1p_sq))
        / ((rx_sq * y1p_sq) + (ry_sq * x1p_sq));
    let coef = sign * sq.max(0.0).sqrt();
    let cxp = coef * ((rx * y1p) / ry);
    let cyp = coef * -((ry * x1p) / rx);

    // Step 3: Compute (cx, cy) from (cx', cy')
    let cx = cos_phi * cxp - sin_phi * cyp + (x0 + x) / 2.0;
    let cy = sin_phi * cxp + cos_phi * cyp + (y0 + y) / 2.0;

    // Step 4: Compute start_angle and delta_angle
    let ux = (x1p - cxp) / rx;
    let uy = (y1p - cyp) / ry;
    let vx = (-x1p - cxp) / rx;
    let vy = (-y1p - cyp) / ry;

    let start_angle = calculate_angle(1.0, 0.0, ux, uy);
    let mut delta_angle = calculate_angle(ux, uy, vx, vy);

    if !sweep_flag && delta_angle > 0.0 {
        delta_angle -= 2.0 * std::f32::consts::PI;
    } else if sweep_flag && delta_angle < 0.0 {
        delta_angle += 2.0 * std::f32::consts::PI;
    }

    Some((cx, cy, start_angle, delta_angle))
}

fn calculate_angle(ux: f32, uy: f32, vx: f32, vy: f32) -> f32 {
    let dot = ux * vx + uy * vy;
    let len = ((ux * ux + uy * uy) * (vx * vx + vy * vy)).sqrt();
    let angle = (dot / len).acos();
    if ux * vy - uy * vx < 0.0 {
        -angle
    } else {
        angle
    }
}

fn rotate_point(px: f32, py: f32, cx: f32, cy: f32, cos_rad: f32, sin_rad: f32) -> (f32, f32) {
    let dx = px - cx;
    let dy = py - cy;
    (
        cx + dx * cos_rad - dy * sin_rad,
        cy + dx * sin_rad + dy * cos_rad,
    )
}

pub(crate) fn push_eliptical_cmds(
    cmds: &mut Vec<Command>,
    x: f32,
    y: f32,
    rx: f32,
    ry: f32,
    angle1: f32,
    angle2: f32,
    x_axis_rotation: f32,
    steps: i32,
) {
    let rad = x_axis_rotation.to_radians();
    let cos_rad = rad.cos();
    let sin_rad = rad.sin();

    let step_f = steps as f32;
    for i in 0..steps as i32 {
        let p1 = i as f32 / step_f;
        let p2 = (i + 1) as f32 / step_f;
        let a1 = angle1 + (angle2 - angle1) * p1;
        let a2 = angle1 + (angle2 - angle1) * p2;

        let (x0, y0) = rotate_point(x + a1.cos() * rx, y + a1.sin() * ry, x, y, cos_rad, sin_rad);
        let (x1, y1) = rotate_point(
            x + ((a1 + a2) * 0.5).cos() * rx,
            y + ((a1 + a2) * 0.5).sin() * ry,
            x,
            y,
            cos_rad,
            sin_rad,
        );
        let (x2, y2) = rotate_point(x + a2.cos() * rx, y + a2.sin() * ry, x, y, cos_rad, sin_rad);

        let cx = 2.0 * x1 - x0 / 2.0 - x2 / 2.0;
        let cy = 2.0 * y1 - y0 / 2.0 - y2 / 2.0;

        if i == 0 {
            cmds.push(Command::LineTo { x: x0, y: y0 });
        }

        cmds.push(Command::SmoothQuadraticBezierCurveTo {
            cx,
            cy,
            x: x2,
            y: y2,
        });
    }
}
