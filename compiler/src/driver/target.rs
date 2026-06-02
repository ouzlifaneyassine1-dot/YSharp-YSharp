use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Native,
    Wasm,
    Gpu,
    Game,
    Kernel,
    Server,
    Desktop,
    Mobile,
}

impl Target {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "native" => Ok(Target::Native),
            "wasm" => Ok(Target::Wasm),
            "gpu" | "ai" => Ok(Target::Gpu),
            "game" => Ok(Target::Game),
            "kernel" => Ok(Target::Kernel),
            "server" => Ok(Target::Server),
            "desktop" => Ok(Target::Desktop),
            "mobile" => Ok(Target::Mobile),
            _ => Err(format!("unknown target '{}'", s)),
        }
    }

    pub fn output_extension(&self) -> &'static str {
        match self {
            Target::Native | Target::Server | Target::Desktop | Target::Mobile => {
                std::env::consts::EXE_SUFFIX
            }
            Target::Wasm => ".wasm",
            Target::Gpu => ".spv",
            Target::Game => ".ysg",
            Target::Kernel => ".o",
        }
    }

    pub fn default_output_name(&self) -> String {
        format!("output{}", self.output_extension())
    }

    pub fn requires_llvm(&self) -> bool {
        matches!(self, Target::Native | Target::Server | Target::Desktop | Target::Mobile | Target::Kernel)
    }

    pub fn requires_wasm(&self) -> bool {
        matches!(self, Target::Wasm | Target::Game)
    }

    pub fn to_mir_target(&self) -> crate::mir::ir::Target {
        match self {
            Target::Native => crate::mir::ir::Target::Native,
            Target::Wasm => crate::mir::ir::Target::Wasm,
            Target::Gpu => crate::mir::ir::Target::Gpu,
            Target::Game => crate::mir::ir::Target::Game,
            Target::Kernel => crate::mir::ir::Target::Kernel,
            Target::Server => crate::mir::ir::Target::Server,
            Target::Desktop => crate::mir::ir::Target::Desktop,
            Target::Mobile => crate::mir::ir::Target::Mobile,
        }
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Target::Native => write!(f, "native"),
            Target::Wasm => write!(f, "wasm"),
            Target::Gpu => write!(f, "gpu"),
            Target::Game => write!(f, "game"),
            Target::Kernel => write!(f, "kernel"),
            Target::Server => write!(f, "server"),
            Target::Desktop => write!(f, "desktop"),
            Target::Mobile => write!(f, "mobile"),
        }
    }
}
