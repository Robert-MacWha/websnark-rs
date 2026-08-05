use ark_bn254::Fr;
use ark_ff::{Field, PrimeField};
use num_bigint::BigInt;
use std::sync::OnceLock;

use crate::{
    circom::ast::{BinOpKind, Expr, Function, Stmt},
    circuit::{CircuitError, Value, rt_ctx::RTCtx},
};

const PRIME: &str = "21888242871839275222246405745257275088548364400416034343698204186575808495617";
const MASK: &str = "28948022309329048855892746252171976963317496166410141009864396001978282409983";

fn bigint_prime() -> &'static BigInt {
    static BIGINT_PRIME: OnceLock<BigInt> = OnceLock::new();
    #[allow(clippy::unwrap_used)]
    BIGINT_PRIME.get_or_init(|| PRIME.parse().unwrap())
}

fn bigint_mask() -> &'static BigInt {
    static BIGINT_MASK: OnceLock<BigInt> = OnceLock::new();
    #[allow(clippy::unwrap_used)]
    BIGINT_MASK.get_or_init(|| MASK.parse().unwrap())
}

/// Executes a function and returns its return value, or zero if it doesn't return anything.
pub fn execute_function(ctx: &mut RTCtx, func: &Function) -> Result<Value, CircuitError> {
    Ok(execute_block(ctx, &func.body)?.unwrap_or(Value::Number(BigInt::ZERO)))
}

fn execute_stmt(ctx: &mut RTCtx, stmt: &Stmt) -> Result<Option<Value>, CircuitError> {
    match stmt {
        Stmt::Block(statements) => execute_block(ctx, statements),
        Stmt::Expr(expr) => {
            execute_expr(ctx, expr)?;
            Ok(None)
        }
        Stmt::Assert { lhs, rhs, loc } => {
            let a = execute_expr(ctx, lhs)?.into_fr()?;
            let b = execute_expr(ctx, rhs)?.into_fr()?;
            ctx.assert_eq(&a, &b, loc)?;
            Ok(None)
        }
        Stmt::If { cond, then, else_ } => {
            let cond = execute_expr(ctx, cond)?;
            if is_truthy(&cond)? {
                execute_stmt(ctx, then)
            } else if let Some(else_) = else_ {
                execute_stmt(ctx, else_)
            } else {
                Ok(None)
            }
        }
        Stmt::While { cond, body } => {
            while is_truthy(&execute_expr(ctx, cond)?)? {
                if let Some(v) = execute_stmt(ctx, body)? {
                    return Ok(Some(v));
                }
            }
            Ok(None)
        }
        Stmt::For {
            init,
            cond,
            update,
            body,
        } => {
            execute_expr(ctx, init)?;
            while is_truthy(&execute_expr(ctx, cond)?)? {
                if let Some(v) = execute_stmt(ctx, body)? {
                    return Ok(Some(v));
                }
                execute_expr(ctx, update)?;
            }
            Ok(None)
        }
        Stmt::Return(expr) => {
            let value = execute_expr(ctx, expr)?;
            Ok(Some(value))
        }
    }
}

#[allow(clippy::too_many_lines)]
fn execute_expr(ctx: &mut RTCtx, expr: &Expr) -> Result<Value, CircuitError> {
    match expr {
        Expr::NumberLit(fr) => Ok(Value::Fr(*fr)),
        Expr::PrimeConst => Ok(bigint_prime().clone().into()),
        Expr::MaskConst => Ok(bigint_mask().clone().into()),
        Expr::ArrayLit(arr) => {
            let res = execute_exprs(ctx, arr)?;
            Ok(Value::Array(res))
        }
        Expr::GetSignal(name, sels) => {
            let sels = execute_exprs(ctx, sels)?;
            ctx.get_signal(name, sels)
        }
        Expr::GetVar(name, sels) => {
            let sels = execute_exprs(ctx, sels)?;
            ctx.get_var(name, sels)
        }
        Expr::GetPin(component_name, component_sels, signal_name, signal_sels) => {
            let component_sels = execute_exprs(ctx, component_sels)?;
            let signal_sels = execute_exprs(ctx, signal_sels)?;
            ctx.get_pin(component_name, component_sels, signal_name, signal_sels)
        }
        Expr::SetSignal(name, sels, value) => {
            let sels = execute_exprs(ctx, sels)?;
            let value = execute_expr(ctx, value)?;
            ctx.set_signal(name, sels, value.clone())?;
            Ok(value)
        }
        Expr::SetVar(name, sels, value) => {
            let sels = execute_exprs(ctx, sels)?;
            let value = execute_expr(ctx, value)?;
            ctx.set_var(name, sels, value.clone())?;
            Ok(value)
        }
        Expr::SetPin(component_name, component_sels, signal_name, signal_sels, value) => {
            let component_sels = execute_exprs(ctx, component_sels)?;
            let signal_sels = execute_exprs(ctx, signal_sels)?;
            let value = execute_expr(ctx, value)?;
            ctx.set_pin(
                component_name,
                component_sels,
                signal_name,
                signal_sels,
                value.clone(),
            )?;
            Ok(value)
        }
        Expr::CallFunction(name, args) => {
            let args = execute_exprs(ctx, args)?;
            ctx.call_function(name, &args)
        }
        Expr::BinOp { op, lhs, rhs } => {
            let lhs = execute_expr(ctx, lhs)?;
            let rhs = execute_expr(ctx, rhs)?;

            match op {
                BinOpKind::Add => Ok((lhs.into_fr()? + rhs.into_fr()?).into()),
                BinOpKind::Sub => Ok((lhs.into_fr()? - rhs.into_fr()?).into()),
                BinOpKind::Mul => Ok((lhs.into_fr()? * rhs.into_fr()?).into()),
                BinOpKind::Mod => {
                    //? Modulo by the prime is a no-op, so we can return lhs directly
                    if matches!(&lhs, Value::Fr(_))
                        && let Value::Number(rhs_n) = &rhs
                        && rhs_n == bigint_prime()
                    {
                        return Ok(lhs);
                    }

                    Ok((lhs.into_number()? % rhs.into_number()?).into())
                }
                BinOpKind::Div => Ok((lhs.into_number()? / rhs.into_number()?).into()),
                BinOpKind::Eq => Ok((lhs.into_fr()? == rhs.into_fr()?).into()),
                BinOpKind::Neq => Ok((lhs.into_fr()? != rhs.into_fr()?).into()),
                BinOpKind::Lt => Ok((lhs.into_fr()? < rhs.into_fr()?).into()),
                BinOpKind::Gt => Ok((lhs.into_fr()? > rhs.into_fr()?).into()),
                BinOpKind::And => Ok((lhs.into_number()? & rhs.into_number()?).into()),
                BinOpKind::Shl => {
                    let lhs = lhs.into_fr()?.into_bigint();
                    let n = rhs.into_u32()?;
                    Ok(Fr::from(lhs << n).into())
                }
                BinOpKind::Shr => {
                    let lhs = lhs.into_fr()?.into_bigint();
                    let n = rhs.into_u32()?;
                    Ok(Fr::from(lhs >> n).into())
                }
            }
        }
        Expr::Inverse(base, _modulos) => {
            let base = execute_expr(ctx, base)?.into_fr()?;
            //? Inverse always exists in a field, so we can ignore the module arg.
            // let modulos = execute_expr(ctx, modulos)?;
            // let modulos = execute_expr(ctx, modulos)?.into_number()?;
            // if modulos != *bigint_prime() {
            //     bail!("modulos must be equal to the prime");
            // }

            base.inverse()
                .ok_or(CircuitError::InvalidInverse)
                .map(std::convert::Into::into)
        }
        Expr::ModPow(base, exp, _modulos) => {
            let base = execute_expr(ctx, base)?.into_fr()?;
            let exp = execute_expr(ctx, exp)?.into_u32()?;
            //? ModPow is always used to convert to Montgomery form, so we can ignore the modulos arg.
            // let modulos = execute_expr(ctx, modulos)?.into_number()?;
            // if modulos != *bigint_prime() {
            //     bail!("modulos must be equal to the prime");
            // }

            Ok(base.pow([u64::from(exp)]).into())
        }
        Expr::LogicalOr(lhs, rhs) => {
            let lhs = execute_expr(ctx, lhs)?;
            if !lhs.is_zero()? {
                return Ok(lhs);
            }
            let rhs = execute_expr(ctx, rhs)?;
            Ok(rhs)
        }
        Expr::Ternary { cond, then, else_ } => {
            let cond = execute_expr(ctx, cond)?;
            if cond.is_zero()? {
                execute_expr(ctx, else_)
            } else {
                execute_expr(ctx, then)
            }
        }
    }
}

/// Executes a block of statements and returns the value of the first return statement, if any.
fn execute_block(ctx: &mut RTCtx, stmts: &[Stmt]) -> Result<Option<Value>, CircuitError> {
    for s in stmts {
        if let Some(v) = execute_stmt(ctx, s)? {
            return Ok(Some(v));
        }
    }
    Ok(None)
}

/// Executes a list of expressions and returns their results.
fn execute_exprs(ctx: &mut RTCtx, exprs: &[Expr]) -> Result<Vec<Value>, CircuitError> {
    let mut res = Vec::with_capacity(exprs.len());
    for expr in exprs {
        res.push(execute_expr(ctx, expr)?);
    }
    Ok(res)
}

/// Returns true if the given value is "truthy" (not zero).
fn is_truthy(v: &Value) -> Result<bool, CircuitError> {
    Ok(!v.is_zero()?)
}
