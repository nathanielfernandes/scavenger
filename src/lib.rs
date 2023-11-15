use logos::{Lexer, Logos};

#[derive(Clone, Copy, Debug, PartialEq)]
enum Cmd {
    M,
    L,
    H,
    V,
    C,
    S,
    Q,
    T,
    A,
    Z,
}

impl Cmd {
    #[inline]
    fn map(c: char) -> Option<(Self, bool)> {
        return Some(match c {
            'M' => (Cmd::M, false),
            'm' => (Cmd::M, true),
            'L' => (Cmd::L, false),
            'l' => (Cmd::L, true),
            'H' => (Cmd::H, false),
            'h' => (Cmd::H, true),
            'V' => (Cmd::V, false),
            'v' => (Cmd::V, true),
            'C' => (Cmd::C, false),
            'c' => (Cmd::C, true),
            'S' => (Cmd::S, false),
            's' => (Cmd::S, true),
            'Q' => (Cmd::Q, false),
            'q' => (Cmd::Q, true),
            'T' => (Cmd::T, false),
            't' => (Cmd::T, true),
            'A' => (Cmd::A, false),
            'a' => (Cmd::A, true),
            'Z' => (Cmd::Z, false),
            'z' => (Cmd::Z, true),
            _ => return None,
        });
    }
}

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ ,\t\r\n\f]+")]
enum Token {
    // any single character
    #[regex(r"[a-zA-Z]", |lex| Cmd::map(lex.slice().chars().next().expect("char")))]
    Command((Cmd, bool)),

    // any floating point number
    #[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?", |lex| lex.slice().parse::<f32>().unwrap_or(0.0))]
    Number(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    // M x y
    MoveTo {
        x: f32,
        y: f32,
    },
    // L x y
    LineTo {
        x: f32,
        y: f32,
    },
    // H x
    HorizontalLineTo {
        x: f32,
    },
    // V y
    VerticalLineTo {
        y: f32,
    },
    // C x1 y1 x2 y2 x y
    CurveTo {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x: f32,
        y: f32,
    },
    // Z
    ClosePath,
    // S x2 y2 x y
    SmoothCurveTo {
        x2: f32,
        y2: f32,
        x: f32,
        y: f32,
    },
    // Q x1 y1 x y
    QuadraticBezierCurveTo {
        x1: f32,
        y1: f32,
        x: f32,
        y: f32,
    },
    // T x y
    SmoothQuadraticBezierCurveTo {
        x: f32,
        y: f32,
    },
    // A rx ry x-axis-rotation large-arc-flag sweep-flag x y
    EllipticalArc {
        rx: f32,
        ry: f32,
        x_axis_rotation: f32,
        large_arc_flag: bool,
        sweep_flag: bool,
        x: f32,
        y: f32,
    },
}

#[inline]
fn number(lexer: &mut Lexer<Token>) -> Result<f32, &'static str> {
    match lexer.next() {
        Some(Ok(Token::Number(n))) => Ok(n),
        _ => Err("expected number"),
    }
}

#[inline]
fn parse_path<'src>(lexer: &mut Lexer<'src, Token>) -> Result<Vec<Command>, &'static str> {
    let mut commands = Vec::new();

    // current point
    let mut px = 0.0;
    let mut py = 0.0;

    // current control point
    let mut cx = 0.0;
    let mut cy = 0.0;

    // current subpath starting point
    let mut sx = 0.0;
    let mut sy = 0.0;

    while let Some(Ok(token)) = lexer.next() {
        match token {
            Token::Command((Cmd::M, relative)) => {
                let x = number(lexer)?;
                let y = number(lexer)?;

                if relative {
                    px += x;
                    py += y;
                } else {
                    px = x;
                    py = y;
                }

                sx = px;
                sy = py;

                commands.push(Command::MoveTo { x: px, y: py });
            }

            Token::Command((Cmd::L, relative)) => {
                let x = number(lexer)?;
                let y = number(lexer)?;

                if relative {
                    px += x;
                    py += y;
                } else {
                    px = x;
                    py = y;
                }

                commands.push(Command::LineTo { x: px, y: py });
            }

            Token::Command((Cmd::H, relative)) => {
                let x = number(lexer)?;

                if relative {
                    px += x;
                } else {
                    px = x;
                }

                commands.push(Command::HorizontalLineTo { x: px });
            }

            Token::Command((Cmd::V, relative)) => {
                let y = number(lexer)?;

                if relative {
                    py += y;
                } else {
                    py = y;
                }

                commands.push(Command::VerticalLineTo { y: py });
            }

            Token::Command((Cmd::C, relative)) => {
                let _x1 = number(lexer)?;
                let _y1 = number(lexer)?;
                let x2 = number(lexer)?;
                let y2 = number(lexer)?;
                let x = number(lexer)?;
                let y = number(lexer)?;

                if relative {
                    cx = px + x2;
                    cy = py + y2;
                    px += x;
                    py += y;
                } else {
                    cx = x2;
                    cy = y2;
                    px = x;
                    py = y;
                }

                commands.push(Command::CurveTo {
                    x1: cx,
                    y1: cy,
                    x2: cx,
                    y2: cy,
                    x: px,
                    y: py,
                });
            }

            Token::Command((Cmd::S, relative)) => {
                let x2 = number(lexer)?;
                let y2 = number(lexer)?;
                let x = number(lexer)?;
                let y = number(lexer)?;

                if relative {
                    cx = px + x2;
                    cy = py + y2;
                    px += x;
                    py += y;
                } else {
                    cx = x2;
                    cy = y2;
                    px = x;
                    py = y;
                }

                commands.push(Command::SmoothCurveTo {
                    x2: cx,
                    y2: cy,
                    x: px,
                    y: py,
                });
            }

            Token::Command((Cmd::Q, relative)) => {
                let x1 = number(lexer)?;
                let y1 = number(lexer)?;
                let x = number(lexer)?;
                let y = number(lexer)?;

                if relative {
                    cx = px + x1;
                    cy = py + y1;
                    px += x;
                    py += y;
                } else {
                    cx = x1;
                    cy = y1;
                    px = x;
                    py = y;
                }

                commands.push(Command::QuadraticBezierCurveTo {
                    x1: cx,
                    y1: cy,
                    x: px,
                    y: py,
                });
            }

            Token::Command((Cmd::T, relative)) => {
                let x = number(lexer)?;
                let y = number(lexer)?;

                if relative {
                    cx = px + (px - cx);
                    cy = py + (py - cy);
                    px += x;
                    py += y;
                } else {
                    cx = px + (px - cx);
                    cy = py + (py - cy);
                    px = x;
                    py = y;
                }

                commands.push(Command::SmoothQuadraticBezierCurveTo { x: px, y: py });
            }

            Token::Command((Cmd::A, relative)) => {
                let rx = number(lexer)?;
                let ry = number(lexer)?;
                let x_axis_rotation = number(lexer)?;
                let large_arc_flag = if number(lexer)? != 0.0 { true } else { false };
                let sweep_flag = if number(lexer)? != 0.0 { true } else { false };
                let x = number(lexer)?;
                let y = number(lexer)?;

                if relative {
                    px += x;
                    py += y;
                } else {
                    px = x;
                    py = y;
                }

                commands.push(Command::EllipticalArc {
                    rx,
                    ry,
                    x_axis_rotation,
                    large_arc_flag,
                    sweep_flag,
                    x: px,
                    y: py,
                });
            }

            Token::Command((Cmd::Z, _)) => {
                px = sx;
                py = sy;

                commands.push(Command::ClosePath);
            }
            Token::Number(_) => return Err("expected command"),
        }
    }

    Ok(commands)
}

pub fn parse_path_str(path: &str) -> Result<Vec<Command>, &'static str> {
    let mut lexer = Token::lexer(path);
    parse_path(&mut lexer)
}
