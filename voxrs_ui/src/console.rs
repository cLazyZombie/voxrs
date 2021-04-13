#[allow(dead_code)]
pub struct Console {
    outputs: Vec<String>,
    width: u32,
    height: u32,
}

impl Console {
    pub fn build() -> Builder {
        Builder::new()
    }
}

pub struct Builder {
    width: u32,
    height: u32,
}

impl Builder {
    fn new() -> Self {
        Self {
            width: 100,
            height: 100,
        }
    }

    pub fn width(mut self, w: u32) -> Self {
        self.width = w;
        self
    }

    pub fn height(mut self, h: u32) -> Self {
        self.height = h;
        self
    }

    pub fn build(self) -> Console {
        Console {
            outputs: Vec::new(),
            width: self.width,
            height: self.height,
        }
    }
}
