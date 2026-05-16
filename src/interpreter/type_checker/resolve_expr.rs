use crate::interpreter::type_checker::TypeChecker;
use crate::interpreter::type_checker::lang_types::{LanguageType, FunctionType};
use crate::interpreter::type_checker::type_error::TypeError;
use crate::interpreter::parser::ast_types as ast;

impl TypeChecker {
    /// Given an expression, attempt to resolve its type.
    /// Returns an error if the composition of the expression
    /// is not type-safe.
    pub fn resolve_expr(&self, expr: &ast::Expr) -> Result<LanguageType, TypeError> {
        match expr {
            ast::Expr::BitsLiteral(b) => {
                let trimmed: String = b[2..].to_string();
                match usize::from_str_radix(&trimmed, 2) {
                    Ok(_) => { },
                    Err(_) => { return Err(TypeError::FailedBitsParse(b.to_string())); }
                };
                return Ok(LanguageType::Bits(trimmed.len()));
            },

            ast::Expr::Identifier(name) => {
                // attempt to resolve `name` in the pre-environment
                match self.identifier_types.get(name) {
                    None => Err(TypeError::UnresolvedIdentifier(name.to_string())),
                    Some(t) => Ok(t.clone())
                }
            }

            ast::Expr::Binary { op, left, right } => {
                let (l, r) = (
                    self.resolve_expr(&left)?, self.resolve_expr(&right)?
                );

                // ensure that L and R are bistrings
                let (l_size, r_size): (usize, usize) = match (&l, &r) {
                    (LanguageType::Bits(x), LanguageType::Bits(y)) => {
                        (*x, *y)
                    },
                    _ => {
                        let compound_type = format!("({}, {})", l.label(), r.label());
                        return Err(TypeError::Expected("bits, bits", compound_type));
                    }
                };

                // ensure that L and R have the same cardinality
                // (as is required by binary operations)
                if l_size != r_size {
                    return Err(TypeError::BinaryOpFailed(l_size, r_size));
                }

                // resolve the type of the expression by determining the operation
                return Ok(match &op {
                    &ast::BinOp::DotProduct => LanguageType::Bits(1),
                    _ => LanguageType::Bits(l_size)
                });
            },

            ast::Expr::Call { callee, args } => {
                // attempt to resolve callee to `LanguageType::Function`
                let resolved_name: LanguageType = self.resolve_expr(&callee)?;
                let f: FunctionType = match resolved_name {
                    LanguageType::Function(f) => f,
                    _ => { return Err(TypeError::Expected("function", resolved_name.label())) }
                };

                // flatten `args` to LanguageType::Bits(n), and flatten again to n
                let mut resolved_args: Vec<usize> = Vec::new();
                for arg in args {
                    let result = self.resolve_expr(&arg)?;
                    let n: usize = match result {
                        LanguageType::Bits(n) => n,
                        _ => { return Err(TypeError::Expected("bits", result.label())); }
                    };
                    resolved_args.push(n);
                }
                
                return match f.input == resolved_args {
                    // as long as the function definition is type-checked,
                    // and that our inputs are well-typed,
                    // then we can deduce that `f.output` is necessarily
                    // correct here.
                    true =>  Ok(LanguageType::Bits(f.output)),
                    false => Err(TypeError::InvalidArgs)
                };
            }

            ast::Expr::Grouping(g) => self.resolve_expr(&g)
        }
    }
}