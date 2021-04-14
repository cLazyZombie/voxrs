use voxrs_math::IVec2;

#[allow(dead_code)]
pub struct Console {
    outputs: Vec<String>,
    pos: IVec2,
    size: IVec2,
}

impl Console {
    pub fn build() -> Builder {
        Builder::new()
    }
}

pub struct Builder {
    pos: IVec2,
    size: IVec2,
}

impl Builder {
    fn new() -> Self {
        Self {
            pos: IVec2::new(0, 0),
            size: IVec2::new(100, 100),
        }
    }

    pub fn pos(mut self, pos: IVec2) -> Self {
        self.pos = pos;
        self
    }

    pub fn size(mut self, size: IVec2) -> Self {
        self.size = size;
        self
    }

    pub fn build(self) -> Console {
        Console {
            outputs: Vec::new(),
            pos: self.pos,
            size: self.size,
        }
    }
}
