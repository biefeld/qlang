use crate::engine::register_error::RegisterError;

// for facilitating lexically-scoped functions
pub type Lambda = std::sync::Arc<dyn Fn(Vec<usize>) -> Result<usize, RegisterError> + Send + Sync>;

#[derive(Clone)]
pub struct BlackBox {
    input_size: usize,
    output_size: usize,
    method: Lambda
}

impl core::fmt::Debug for BlackBox {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let msg = format!(
            "function with input {} and output {}",
            self.input_size,
            self.output_size
        );
        f.write_str(&msg)
    }
}

/// CONSTRUCTORS
impl BlackBox {
    /// Creates a black box determined by some function
    /// `f :: {0,1}^(input_size) -> {0,1}^(output_size)`
    /// 
    /// Pre `input_size >= output_size >= 1`
    pub fn new(f: Lambda, input_size: usize, output_size: usize) -> Self {
        Self { input_size, output_size, method: f }
    }
}

/// PROJECTIONS
impl BlackBox {
    pub fn input_size(&self) -> usize { self.input_size }
    pub fn output_size(&self) -> usize { self.output_size }
}

/// API-CALLABLE METHODS
impl BlackBox {
    /// Pre:
    /// 
    /// - `input ∈ {0,1}^k` (note that this is not enforced!)
    /// 
    /// - `self.input_size == k`
    pub fn eval(&self, input: usize) -> Result<usize, RegisterError> {
        return (self.method) (vec![input]);
    }
}