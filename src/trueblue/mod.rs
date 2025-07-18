#[derive(Debug, Clone)]
pub enum Target {
    CLI,
}

impl Target {
    fn display(&self, message: String) {
        match *self {
            Target::CLI => println!("{}", message),
        }
    }
}

pub struct TrueBlue {
    target: Target,
    pub message: String
}

impl TrueBlue {
    pub fn new(target: Target) -> Self {
        Self {
            target: target,
            message: String::new()
        }
    }

    pub fn display(&self) {
        self.target.display(self.message.clone());
    }
}

pub fn new(target: Target) -> TrueBlue {
    TrueBlue::new(target)
}
