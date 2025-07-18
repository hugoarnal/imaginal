mod targets;

const DEFAULT_WIDTH: u16 = 32;
const DEFAULT_HEIGHT: u16 = 8;

#[derive(Debug, Clone)]
pub enum Target {
    CLI,
}

impl Target {
    fn display(&self, matrix: Vec<bool>, width: u16, height: u16) {
        match *self {
            Target::CLI => targets::cli::display(matrix, width, height),
        }
    }
}

pub struct TrueBlue {
    target: Target,
    pub message: String,
    width: u16,
    height: u16,
    matrix: Vec<bool>,
}

impl TrueBlue {
    pub fn new(target: Target) -> Self {
        let mut tb = Self {
            target: target,
            message: String::new(),
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            matrix: Vec::new(),
        };
        for _ in 1..((tb.width * tb.height) - 1) {
            tb.matrix.push(false);
        }
        tb
    }

    pub fn display(&self) {
        self.target.display(self.matrix.clone(), self.width, self.height);
    }
}

pub fn new(target: Target) -> TrueBlue {
    TrueBlue::new(target)
}
