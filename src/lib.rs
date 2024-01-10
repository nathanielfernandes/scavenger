mod simplification;
pub mod viewbox;

use logos::{Lexer, Logos};
use simplification::{calculate_ellipse_parameters, push_eliptical_cmds};
use std::iter::Peekable;

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
#[logos(skip r"[ ,\t\r\n]+")]
enum Token {
    // any single character
    #[regex(r"[a-zA-Z]", |lex| Cmd::map(lex.slice().chars().next().expect("char")))]
    Command((Cmd, bool)),

    // any floating point number
    #[regex(r"-?(?:0|[1-9]\d*)?(?:\.\d+)?", |lex| lex.slice().parse::<f32>().unwrap_or(0.0))]
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
        cx: f32,
        cy: f32,

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
        cx: f32,
        cy: f32,

        x: f32,
        y: f32,
    },
    // // A rx ry x-axis-rotation large-arc-flag sweep-flag x y
    // EllipticalArc {
    //     px: f32,
    //     py: f32,

    //     rx: f32,
    //     ry: f32,
    //     x_axis_rotation: f32,
    //     large_arc_flag: bool,
    //     sweep_flag: bool,
    //     x: f32,
    //     y: f32,
    // },
}

pub struct Parser<'src> {
    lexer: Peekable<Lexer<'src, Token>>,

    px: f32,
    py: f32,

    cx: f32,
    cy: f32,

    sx: f32,
    sy: f32,

    bezier_steps: i32,

    last_command: Option<Cmd>,

    commands: Vec<Command>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Expected {
    Command,
    Number,
}

impl<'src> Parser<'src> {
    pub fn new(path: &'src str) -> Parser<'src> {
        let lexer = Token::lexer(path);

        Parser {
            lexer: lexer.peekable(),

            px: 0.0,
            py: 0.0,

            cx: 0.0,
            cy: 0.0,

            sx: 0.0,
            sy: 0.0,

            bezier_steps: 16,

            last_command: None,

            commands: Vec::new(),
        }
    }

    pub fn bezier_steps(mut self, bezier_steps: i32) -> Self {
        self.bezier_steps = bezier_steps;
        self
    }

    pub fn parse(mut self) -> Result<Vec<Command>, Expected> {
        while let Some(Ok(token)) = self.lexer.next() {
            match token {
                Token::Command((command, relative)) => {
                    match command {
                        Cmd::M => self.m(relative)?,
                        Cmd::L => self.l(relative)?,
                        Cmd::H => self.h(relative)?,
                        Cmd::V => self.v(relative)?,
                        Cmd::C => self.c(relative)?,
                        Cmd::S => self.s(relative)?,
                        Cmd::Q => self.q(relative)?,
                        Cmd::T => self.t(relative)?,
                        Cmd::A => self.a(relative)?,
                        Cmd::Z => {
                            self.px = self.sx;
                            self.py = self.sy;

                            self.commands.push(Command::ClosePath);
                        }
                    }

                    self.last_command = Some(command);
                }
                Token::Number(_) => {
                    return Err(Expected::Command);
                }
            }
        }

        Ok(self.commands)
    }

    #[inline]
    fn peek<'a>(&'a mut self) -> Option<&'a Result<Token, ()>> {
        self.lexer.peek()
    }

    #[inline]
    fn number(&mut self) -> Result<f32, Expected> {
        match self.lexer.next() {
            Some(Ok(Token::Number(n))) => Ok(n),
            _ => Err(Expected::Number),
        }
    }

    #[inline]
    fn delta(&self, relative: bool) -> (f32, f32) {
        if relative {
            (self.px, self.py)
        } else {
            (0.0, 0.0)
        }
    }

    #[inline]
    fn try_number(&mut self) -> Result<f32, Expected> {
        match self.peek() {
            Some(Ok(Token::Number(n))) => {
                let n = *n;
                self.lexer.next();
                Ok(n)
            }
            _ => Err(Expected::Number),
        }
    }

    #[inline]
    fn m(&mut self, relative: bool) -> Result<(), Expected> {
        let x = self.number()?;
        let y = self.number()?;

        self.px = x + if relative { self.px } else { 0.0 };
        self.py = y + if relative { self.py } else { 0.0 };

        self.sx = self.px;
        self.sy = self.py;

        self.commands.push(Command::MoveTo {
            x: self.px,
            y: self.py,
        });

        self.l(relative)?;

        Ok(())
    }

    #[inline]
    fn l(&mut self, relative: bool) -> Result<(), Expected> {
        loop {
            let Ok(x) = self.try_number() else {
                break;
            };

            let y = self.number()?;

            self.px = x + if relative { self.px } else { 0.0 };
            self.py = y + if relative { self.py } else { 0.0 };

            self.commands.push(Command::LineTo {
                x: self.px,
                y: self.py,
            });
        }

        Ok(())
    }

    #[inline]
    fn h(&mut self, relative: bool) -> Result<(), Expected> {
        loop {
            let Ok(x) = self.try_number() else {
                break;
            };

            self.px = x + if relative { self.px } else { 0.0 };

            self.commands.push(Command::LineTo {
                x: self.px,
                y: self.py,
            });
        }

        Ok(())
    }

    #[inline]
    fn v(&mut self, relative: bool) -> Result<(), Expected> {
        loop {
            let Ok(y) = self.try_number() else {
                break;
            };

            self.py = y + if relative { self.py } else { 0.0 };

            self.commands.push(Command::LineTo {
                x: self.px,
                y: self.py,
            });
        }

        Ok(())
    }

    #[inline]
    fn c(&mut self, relative: bool) -> Result<(), Expected> {
        loop {
            let (dx, dy) = self.delta(relative);
            let Ok(x1) = self.try_number() else {
                break;
            };

            let y1 = self.number()?;
            let x2 = self.number()?;
            let y2 = self.number()?;
            let x = self.number()?;
            let y = self.number()?;

            self.px = x + dx;
            self.py = y + dy;

            self.cx = x2 + dx;
            self.cy = y2 + dy;

            self.commands.push(Command::CurveTo {
                x1: x1 + dx,
                y1: y1 + dy,
                x2: x2 + dx,
                y2: y2 + dy,
                x: self.px,
                y: self.py,
            });
        }

        Ok(())
    }

    #[inline]
    fn s(&mut self, relative: bool) -> Result<(), Expected> {
        loop {
            let (dx, dy) = self.delta(relative);
            let Ok(x2) = self.try_number() else {
                break;
            };

            let y2 = self.number()?;
            let x = self.number()?;
            let y = self.number()?;

            if let Some(Cmd::C | Cmd::S) = self.last_command {
                self.cx = self.px + (self.px - self.cx);
                self.cy = self.py + (self.py - self.cy);
            } else {
                self.cx = self.px;
                self.cy = self.py;
            }

            self.px = x + dx;
            self.py = y + dy;

            self.commands.push(Command::SmoothCurveTo {
                cx: self.cx,
                cy: self.cy,

                x2: x2 + dx,
                y2: y2 + dy,
                x: self.px,
                y: self.py,
            });

            self.cx = x2 + dx;
            self.cy = y2 + dy;
        }

        Ok(())
    }

    #[inline]
    fn q(&mut self, relative: bool) -> Result<(), Expected> {
        loop {
            let (dx, dy) = self.delta(relative);

            let Ok(x1) = self.try_number() else {
                break;
            };

            let y1 = self.number()?;
            let x = self.number()?;
            let y = self.number()?;

            self.px = x + dx;
            self.py = y + dy;

            self.cx = x1 + dx;
            self.cy = y1 + dy;

            self.commands.push(Command::QuadraticBezierCurveTo {
                x1: x1 + dx,
                y1: y1 + dy,
                x: self.px,
                y: self.py,
            });
        }

        Ok(())
    }

    #[inline]
    fn t(&mut self, relative: bool) -> Result<(), Expected> {
        loop {
            let (dx, dy) = self.delta(relative);
            let Ok(x) = self.try_number() else {
                break;
            };

            let y = self.number()?;

            if let Some(Cmd::Q | Cmd::T) = self.last_command {
                self.cx = self.px + (self.px - self.cx);
                self.cy = self.py + (self.py - self.cy);
            } else {
                self.cx = self.px;
                self.cy = self.py;
            }

            self.px = x + dx;
            self.py = y + dy;

            self.commands.push(Command::SmoothQuadraticBezierCurveTo {
                cx: self.cx,
                cy: self.cy,

                x: self.px,
                y: self.py,
            });

            self.cx = self.px;
            self.cy = self.py;
        }

        Ok(())
    }

    #[inline]
    fn a(&mut self, relative: bool) -> Result<(), Expected> {
        loop {
            let (dx, dy) = self.delta(relative);

            let Ok(rx) = self.try_number() else {
                break;
            };

            let ry = self.number()?;
            let x_axis_rotation = self.number()?;
            let large_arc_flag = self.number()? != 0.0;
            let sweep_flag = self.number()? != 0.0;
            let x = self.number()?;
            let y = self.number()?;

            let x2 = self.px;
            let y2 = self.py;

            self.px = dx + x;
            self.py = dy + y;

            // self.commands.push(Command::EllipticalArc {
            //     px: x2,
            //     py: y2,

            //     rx,
            //     ry,
            //     x_axis_rotation,
            //     large_arc_flag,
            //     sweep_flag,
            //     x: self.px,
            //     y: self.py,
            // });

            if let Some((cx, cy, start_angle, delta_angle)) = calculate_ellipse_parameters(
                x2,
                y2,
                self.px,
                self.py,
                rx,
                ry,
                x_axis_rotation,
                large_arc_flag,
                sweep_flag,
            ) {
                push_eliptical_cmds(
                    &mut self.commands,
                    cx,
                    cy,
                    rx,
                    ry,
                    start_angle,
                    start_angle + delta_angle,
                    x_axis_rotation,
                    self.bezier_steps,
                );
            }

            self.cx = self.px;
            self.cy = self.py;
        }

        Ok(())
    }
}

pub fn parse_path_str(path: &str) -> Result<Vec<Command>, Expected> {
    Parser::new(path).parse()
}
