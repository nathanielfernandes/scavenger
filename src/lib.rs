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
        px: f32,
        py: f32,

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
#[allow(unused_assignments)]
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

    let mut last_command = None;

    while let Some(Ok(token)) = lexer.next() {
        match token {
            Token::Command((command, relative)) => {
                let (dx, dy) = if relative { (px, py) } else { (0.0, 0.0) };

                match command {
                    Cmd::M => {
                        let x = number(lexer)?;
                        let y = number(lexer)?;

                        px = x + dx;
                        py = y + dy;

                        sx = px;
                        sy = py;

                        commands.push(Command::MoveTo { x: px, y: py });
                    }

                    Cmd::L => {
                        let x = number(lexer)?;
                        let y = number(lexer)?;

                        px = x + dx;
                        py = y + dy;

                        commands.push(Command::LineTo { x: px, y: py });
                    }

                    Cmd::H => {
                        let x = number(lexer)?;

                        px = x + dx;

                        commands.push(Command::LineTo { x: px, y: py });
                    }

                    Cmd::V => {
                        let y = number(lexer)?;

                        py = y + dy;

                        commands.push(Command::LineTo { x: px, y: py });
                    }

                    Cmd::C => {
                        let x1 = number(lexer)?;
                        let y1 = number(lexer)?;
                        let x2 = number(lexer)?;
                        let y2 = number(lexer)?;
                        let x = number(lexer)?;
                        let y = number(lexer)?;

                        px = x + dx;
                        py = y + dy;

                        cx = x2 + dx;
                        cy = y2 + dy;

                        commands.push(Command::CurveTo {
                            x1: x1 + dx,
                            y1: y1 + dy,
                            x2: x2 + dx,
                            y2: y2 + dy,
                            x: px,
                            y: py,
                        });
                    }

                    Cmd::S => {
                        let x2 = number(lexer)?;
                        let y2 = number(lexer)?;
                        let x = number(lexer)?;
                        let y = number(lexer)?;

                        if let Some(Cmd::C | Cmd::S) = last_command {
                            cx = px + (px - cx);
                            cy = py + (py - cy);
                        } else {
                            cx = px;
                            cy = py;
                        }

                        px = x + dx;
                        py = y + dy;

                        cx = x2 + dx;
                        cy = y2 + dy;

                        commands.push(Command::SmoothCurveTo {
                            x2: x2 + dx,
                            y2: y2 + dy,
                            x: px,
                            y: py,
                        });
                    }

                    Cmd::Q => {
                        let x1 = number(lexer)?;
                        let y1 = number(lexer)?;
                        let x = number(lexer)?;
                        let y = number(lexer)?;

                        px = x + dx;
                        py = y + dy;

                        cx = x1 + dx;
                        cy = y1 + dy;

                        commands.push(Command::QuadraticBezierCurveTo {
                            x1: x1 + dx,
                            y1: y1 + dy,
                            x: px,
                            y: py,
                        });
                    }

                    Cmd::T => {
                        let x = number(lexer)?;
                        let y = number(lexer)?;

                        if let Some(Cmd::Q | Cmd::T) = last_command {
                            cx = px + (px - cx);
                            cy = py + (py - cy);
                        } else {
                            cx = px;
                            cy = py;
                        }

                        px = x + dx;
                        py = y + dy;

                        commands.push(Command::SmoothQuadraticBezierCurveTo { x: px, y: py });
                    }

                    Cmd::A => {
                        let rx = number(lexer)?;
                        let ry = number(lexer)?;
                        let x_axis_rotation = number(lexer)?;
                        let large_arc_flag = number(lexer)? != 0.0;
                        let sweep_flag = number(lexer)? != 0.0;
                        let x = number(lexer)?;
                        let y = number(lexer)?;

                        let x2 = px;
                        let y2 = py;

                        px = dx + x;
                        py = dy + y;

                        commands.push(Command::EllipticalArc {
                            px: x2,
                            py: y2,

                            rx,
                            ry,
                            x_axis_rotation,
                            large_arc_flag,
                            sweep_flag,
                            x: px,
                            y: py,
                        });
                    }

                    Cmd::Z => {
                        px = sx;
                        py = sy;

                        commands.push(Command::ClosePath);
                    }
                }

                last_command = Some(command);
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
