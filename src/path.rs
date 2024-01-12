use crate::{
    viewbox::{calculate_bb, ViewBox},
    Command,
};

pub struct Path {
    pub(crate) commands: Vec<Command>,
    pub(crate) bb: (f32, f32),
}

impl Path {
    pub fn new(commands: Vec<Command>) -> Self {
        let bb = calculate_bb(commands.iter());
        Self { commands, bb }
    }

    pub fn commands(&self) -> &[Command] {
        &self.commands
    }

    pub fn take_commands(self) -> Vec<Command> {
        self.commands
    }

    pub fn bb(&self) -> (f32, f32) {
        self.bb
    }

    pub fn translate(&mut self, x: f32, y: f32) {
        for cmd in self.commands.iter_mut() {
            *cmd = cmd.translate(x, y);
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        let scalex = width / self.bb.0;
        let scaley = height / self.bb.1;

        let vb = ViewBox::new(0.0, 0.0, self.bb.0 / scalex, self.bb.1 / scaley);

        for cmd in self.commands.iter_mut() {
            *cmd = vb.scale_cmd(cmd, self.bb.0, self.bb.1);
        }
    }

    pub fn scale(&mut self, scale: f32) {
        let vb = ViewBox::new(0.0, 0.0, self.bb.0 / scale, self.bb.1 / scale);

        for cmd in self.commands.iter_mut() {
            *cmd = vb.scale_cmd(cmd, self.bb.0, self.bb.1);
        }
    }

    pub fn fit(&mut self, width: f32, height: f32) {
        let scalex = width / self.bb.0;
        let scaley = height / self.bb.1;

        let scale = scalex.min(scaley);

        let vb = ViewBox::new(0.0, 0.0, self.bb.0 / scale, self.bb.1 / scale);

        for cmd in self.commands.iter_mut() {
            *cmd = vb.scale_cmd(cmd, self.bb.0, self.bb.1);
        }
    }

    pub fn cover(&mut self, width: f32, height: f32) {
        let scalex = width / self.bb.0;
        let scaley = height / self.bb.1;

        let scale = scalex.max(scaley);

        let vb = ViewBox::new(0.0, 0.0, self.bb.0 / scale, self.bb.1 / scale);

        for cmd in self.commands.iter_mut() {
            *cmd = vb.scale_cmd(cmd, self.bb.0, self.bb.1);
        }
    }
}

impl Into<Path> for Vec<Command> {
    fn into(self) -> Path {
        Path::new(self)
    }
}
