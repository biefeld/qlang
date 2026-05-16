#[derive(Debug, Clone)]
pub enum Statement {
    Assignment(Assignment), // reserved for bits allocation
    Function(FunctionDecl),
    Oracle(OracleDecl),
    Circuit(CircuitDecl),
    MethodCall(MethodCall), // `circuit.printCircuit()`, etc...
    Expr(Expr)
}
pub type Program = Vec<Statement>;



// ----- BISTRING ASSIGNMENT -----
#[derive(Debug, Clone)]
pub struct Assignment {
    pub name: String, // Identifier
    pub value: Expr,  // should only ever be BitsLiteral
}



// ----- METHOD CALLS ------
#[derive(Debug, Clone)]
pub struct MethodCall {
    pub name: String, // Identifier
    pub call: String, // Method Name, i.e., printCircuit, measure, or distribution
    pub args: Vec<MethodArg>
}
#[derive(Debug, Clone)]
pub struct MethodArg {
    pub name: String, // Identifier
    pub value: usize
}



// ----- FUNCTION DEFINITION -----
#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub name: String, // Identifier
    pub params: Vec<Param>,
    pub return_type: Type,
    pub body: Expr,  // functions are expression-bodied
}
#[derive(Debug, Clone)]
pub struct Param {
    pub name: String, // Identifier
    pub ty: Type,
}
#[derive(Debug, Clone)]
pub enum Type {
    Bits(usize),
    Qubits(usize)
}



// ----- ORACLE DEFINITION -----
#[derive(Debug, Clone)]
pub struct OracleDecl {
    pub name: String, // Identifier
    pub params: Vec<Param>,
    pub loads: String // Identifier
}



// ----- CIRCUIT DEFINITION -----
#[derive(Debug, Clone)]
pub struct CircuitDecl {
    pub name: String, // Identifier
    pub registers: Vec<QubitDecl>,
    pub instructions: Vec<CircuitInstr>,
}
#[derive(Debug, Clone)]
pub struct QubitDecl {
    pub name: String, // Identifier
    pub init: String, // "++", "00"
}
#[derive(Debug, Clone)]
pub struct CircuitInstr {
    pub name: String, // Gate/ Oracle
    pub args: Vec<CircuitRef>, // qubit registers
}
#[derive(Debug, Clone)]
pub struct CircuitRef {
    pub name: String, // Identifier
    pub applies: Applies
}
#[derive(Debug, Clone)]
pub enum Applies {
    One(usize), // captures a number
    All // captures '_'
}



// ----- EXPRESSIONS -----
#[derive(Debug, Clone)]
pub enum Expr {
    Identifier(String),
    BitsLiteral(String),

    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },

    Call {
        callee: Box<Expr>, // function identifier
        args: Vec<Expr>,
    },

    Grouping(Box<Expr>),
}
#[derive(Debug, Clone)]
pub enum BinOp {
    Xor,
    Or,
    And,
    DotProduct
}