#[derive(Clone, PartialEq)]
pub enum LanguageType {
    Bits(usize), // `usize` denotes here the length of the bitstring
    Function(FunctionType),
    Oracle(usize, usize),
    Circuit
}


#[derive(Clone, PartialEq)]
pub struct FunctionType {
    /// the length of each input bitstring, as
    /// specified by the function definition.
    /// 
    /// for example, a signature `f(x: bits[1], y: bits[2])`
    /// will give `self.input` a value of `vec![1,2]`
    pub input: Vec<usize>,

    pub output: usize
}

impl LanguageType {
    pub fn label(&self) -> String {
        match self {
            LanguageType::Bits(_) => String::from("bits"),
            LanguageType::Function(_) => String::from("function"),
            LanguageType::Oracle(_, _) => String::from("oracle"),
            LanguageType::Circuit => String::from("circuit"),
        }
    }
}