use crate::codec::command_available;

#[derive(Clone, Copy, PartialEq)]
pub enum Resolution {
    Original,
    High,
    Low,
}

impl Resolution {
    pub const ALL: [Self; 3] = [Self::Original, Self::High, Self::Low];

    pub fn short_side(self) -> Option<u32> {
        match self {
            Resolution::Original => None,
            Resolution::High => Some(3000),
            Resolution::Low => Some(1440),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Resolution::Original => "Original",
            Resolution::High => "3000 px",
            Resolution::Low => "1440 px",
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            Resolution::Original => "Original",
            Resolution::High => "High",
            Resolution::Low => "Low",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Quality {
    High,
    Medium,
    Low,
}

impl Quality {
    pub const ALL: [Self; 3] = [Self::High, Self::Medium, Self::Low];

    pub fn value_for(self, format: OutputFormat) -> f32 {
        match (self, format) {
            (Quality::High, OutputFormat::Jpg) => 95.0,
            (Quality::Medium, OutputFormat::Jpg) => 85.0,
            (Quality::Low, OutputFormat::Jpg) => 75.0,
            (Quality::High, OutputFormat::Avif) => 85.0,
            (Quality::Medium, OutputFormat::Avif) => 75.0,
            (Quality::Low, OutputFormat::Avif) => 60.0,
            (Quality::High, OutputFormat::Jxl) => 90.0,
            (Quality::Medium, OutputFormat::Jxl) => 80.0,
            (Quality::Low, OutputFormat::Jxl) => 65.0,
            (Quality::High, OutputFormat::Heic) => 60.0,
            (Quality::Medium, OutputFormat::Heic) => 50.0,
            (Quality::Low, OutputFormat::Heic) => 35.0,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Quality::High => "High",
            Quality::Medium => "Medium",
            Quality::Low => "Low",
        }
    }

    pub fn label_with_value(self, format: OutputFormat) -> String {
        format!("{} q{:.0}", self.label(), self.value_for(format))
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Jpg,
    Avif,
    Jxl,
    Heic,
}

impl OutputFormat {
    pub const ALL: [Self; 4] = [Self::Jpg, Self::Avif, Self::Jxl, Self::Heic];

    pub fn extension(self) -> &'static str {
        match self {
            OutputFormat::Jpg => "jpg",
            OutputFormat::Avif => "avif",
            OutputFormat::Jxl => "jxl",
            OutputFormat::Heic => "heic",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            OutputFormat::Jpg => "JPG",
            OutputFormat::Avif => "AVIF",
            OutputFormat::Jxl => "JXL",
            OutputFormat::Heic => "HEIC",
        }
    }

    pub fn encoder(self) -> &'static str {
        match self {
            OutputFormat::Jpg => "cjpeg",
            OutputFormat::Avif => "avifenc",
            OutputFormat::Jxl => "cjxl",
            OutputFormat::Heic => "heif-enc",
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct AvailableEncoders {
    pub jpeg: bool,
    pub avif: bool,
    pub jxl: bool,
    pub heic: bool,
}

impl AvailableEncoders {
    pub fn detect() -> Self {
        Self {
            jpeg: command_available("cjpeg"),
            avif: command_available("avifenc"),
            jxl: command_available("cjxl"),
            heic: command_available("heif-enc"),
        }
    }

    pub fn any(self) -> bool {
        self.jpeg || self.avif || self.jxl || self.heic
    }

    pub fn first_available(self) -> Option<OutputFormat> {
        if self.jpeg {
            Some(OutputFormat::Jpg)
        } else if self.avif {
            Some(OutputFormat::Avif)
        } else if self.jxl {
            Some(OutputFormat::Jxl)
        } else if self.heic {
            Some(OutputFormat::Heic)
        } else {
            None
        }
    }

    pub fn has(self, format: OutputFormat) -> bool {
        match format {
            OutputFormat::Jpg => self.jpeg,
            OutputFormat::Avif => self.avif,
            OutputFormat::Jxl => self.jxl,
            OutputFormat::Heic => self.heic,
        }
    }
}
