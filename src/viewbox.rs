use crate::Command;

#[derive(Debug, Clone, Copy)]
pub struct ViewBox {
    pub min_x: f32,
    pub min_y: f32,

    pub width: f32,
    pub height: f32,
}

impl ViewBox {
    pub fn new(min_x: f32, min_y: f32, width: f32, height: f32) -> Self {
        Self {
            min_x,
            min_y,
            width,
            height,
        }
    }

    #[inline(always)]
    fn scale_x(&self, x: f32, w: f32) -> f32 {
        (x - self.min_x) * w / self.width
    }

    #[inline(always)]
    fn scale_y(&self, y: f32, h: f32) -> f32 {
        (y - self.min_y) * h / self.height
    }

    #[inline(always)]
    fn scale_cmd(&self, cmd: &Command, w: f32, h: f32) -> Command {
        match cmd {
            Command::MoveTo { x, y } => Command::MoveTo {
                x: self.scale_x(*x, w),
                y: self.scale_y(*y, h),
            },
            Command::LineTo { x, y } => Command::LineTo {
                x: self.scale_x(*x, w),
                y: self.scale_y(*y, h),
            },
            Command::CurveTo {
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            } => Command::CurveTo {
                x1: self.scale_x(*x1, w),
                y1: self.scale_y(*y1, h),
                x2: self.scale_x(*x2, w),
                y2: self.scale_y(*y2, h),
                x: self.scale_x(*x, w),
                y: self.scale_y(*y, h),
            },
            Command::ClosePath => Command::ClosePath,
            Command::SmoothCurveTo {
                cx,
                cy,
                x2,
                y2,
                x,
                y,
            } => Command::SmoothCurveTo {
                cx: self.scale_x(*cx, w),
                cy: self.scale_y(*cy, h),
                x2: self.scale_x(*x2, w),
                y2: self.scale_y(*y2, h),
                x: self.scale_x(*x, w),
                y: self.scale_y(*y, h),
            },
            Command::QuadraticBezierCurveTo { x1, y1, x, y } => Command::QuadraticBezierCurveTo {
                x1: self.scale_x(*x1, w),
                y1: self.scale_y(*y1, h),
                x: self.scale_x(*x, w),
                y: self.scale_y(*y, h),
            },
            Command::SmoothQuadraticBezierCurveTo { cx, cy, x, y } => {
                Command::SmoothQuadraticBezierCurveTo {
                    cx: self.scale_x(*cx, w),
                    cy: self.scale_y(*cy, h),
                    x: self.scale_x(*x, w),
                    y: self.scale_y(*y, h),
                }
            }
        }
    }

    #[inline(always)]
    pub fn scale_path(&self, path: &[Command]) -> Vec<Command> {
        let (w, h) = estimate_dimensions(path);

        path.iter().map(|cmd| self.scale_cmd(cmd, w, h)).collect()
    }

    #[inline(always)]
    pub fn scale_path_mut(&self, path: &mut [Command]) {
        let (w, h) = estimate_dimensions(path);

        for cmd in path.iter_mut() {
            *cmd = self.scale_cmd(cmd, w, h);
        }
    }

    pub fn scale_iter<'a>(&'a self, path: &'a [Command]) -> ScaledIterator {
        let (w, h) = estimate_dimensions(path);
        ScaledIterator::new(self, path.iter(), (w, h))
    }
}

pub struct ScaledIterator<'a> {
    view_box: &'a ViewBox,
    dims: (f32, f32),
    iter: std::slice::Iter<'a, Command>,
}

impl<'a> ScaledIterator<'a> {
    fn new(view_box: &'a ViewBox, iter: std::slice::Iter<'a, Command>, dims: (f32, f32)) -> Self {
        Self {
            view_box,
            iter,
            dims,
        }
    }
}

impl<'a> Iterator for ScaledIterator<'a> {
    type Item = Command;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|cmd| self.view_box.scale_cmd(cmd, self.dims.0, self.dims.1))
    }
}

pub fn estimate_dimensions(path: &[Command]) -> (f32, f32) {
    let mut min_x = 0.0f32;
    let mut min_y = 0.0f32;
    let mut max_x = 0.0f32;
    let mut max_y = 0.0f32;

    for cmd in path {
        match cmd {
            Command::MoveTo { x, y } => {
                min_x = min_x.min(*x);
                min_y = min_y.min(*y);
                max_x = max_x.max(*x);
                max_y = max_y.max(*y);
            }
            Command::LineTo { x, y } => {
                min_x = min_x.min(*x);
                min_y = min_y.min(*y);
                max_x = max_x.max(*x);
                max_y = max_y.max(*y);
            }
            Command::CurveTo {
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            } => {
                min_x = min_x.min(*x1);
                min_y = min_y.min(*y1);
                max_x = max_x.max(*x1);
                max_y = max_y.max(*y1);

                min_x = min_x.min(*x2);
                min_y = min_y.min(*y2);
                max_x = max_x.max(*x2);
                max_y = max_y.max(*y2);

                min_x = min_x.min(*x);
                min_y = min_y.min(*y);
                max_x = max_x.max(*x);
                max_y = max_y.max(*y);
            }
            Command::ClosePath => {}
            Command::SmoothCurveTo {
                cx,
                cy,
                x2,
                y2,
                x,
                y,
            } => {
                min_x = min_x.min(*cx);
                min_y = min_y.min(*cy);
                max_x = max_x.max(*cx);
                max_y = max_y.max(*cy);

                min_x = min_x.min(*x2);
                min_y = min_y.min(*y2);
                max_x = max_x.max(*x2);
                max_y = max_y.max(*y2);

                min_x = min_x.min(*x);
                min_y = min_y.min(*y);
                max_x = max_x.max(*x);
                max_y = max_y.max(*y);
            }
            Command::SmoothQuadraticBezierCurveTo { x, y, cx, cy } => {
                min_x = min_x.min(*cx);
                min_y = min_y.min(*cy);
                max_x = max_x.max(*cx);
                max_y = max_y.max(*cy);

                min_x = min_x.min(*x);
                min_y = min_y.min(*y);
                max_x = max_x.max(*x);
                max_y = max_y.max(*y);
            }

            Command::QuadraticBezierCurveTo { x1, y1, x, y } => {
                min_x = min_x.min(*x1);
                min_y = min_y.min(*y1);
                max_x = max_x.max(*x1);
                max_y = max_y.max(*y1);

                min_x = min_x.min(*x);
                min_y = min_y.min(*y);
                max_x = max_x.max(*x);
                max_y = max_y.max(*y);
            }
        }
    }

    (max_x - min_x, max_y - min_y)
}
