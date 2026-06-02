pub enum RuntimeFeature {
    Gc,
    Async,
    Actors,
    Tensors,
    Ecs,
}

pub struct RuntimeConfig {
    pub features: Vec<RuntimeFeature>,
    pub gc_heap_size: usize,
    pub stack_size: usize,
    pub tensor_pool_size: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            features: vec![RuntimeFeature::Gc, RuntimeFeature::Async],
            gc_heap_size: 16 * 1024 * 1024,
            stack_size: 2 * 1024 * 1024,
            tensor_pool_size: 64 * 1024 * 1024,
        }
    }
}

pub struct Target {
    pub arch: String,
    pub os: String,
}

pub fn select_runtime(target: &Target) -> RuntimeConfig {
    let mut config = RuntimeConfig::default();
    match target.os.as_str() {
        "wasm" | "bare-metal" => {
            config.features = vec![RuntimeFeature::Gc];
            config.gc_heap_size = 4 * 1024 * 1024;
            config.stack_size = 64 * 1024;
        }
        "linux" | "macos" | "windows" => {
            config.features = vec![
                RuntimeFeature::Gc,
                RuntimeFeature::Async,
                RuntimeFeature::Tensors,
            ];
        }
        _ => {}
    }
    config
}
