use ark_bn254::Fr;
use ark_ff::{AdditiveGroup, Field};

use crate::{
    circom::ast::{BinOpKind, Expr, Function, Stmt},
    circuit::{
        CircuitError, Value,
        rt_ctx::RTCtx,
        value::{bigint_to_fr, fr_to_bigint},
    },
};

/// Executes a function and returns its return value, or zero if it doesn't return anything.
pub fn execute_function(ctx: &mut RTCtx, func: &Function) -> Result<Value, CircuitError> {
    Ok(execute_block(ctx, &func.body)?.unwrap_or(0u64.into()))
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
        //? p == 0 (mod p), which is correct for Fr-native ops.
        Expr::PrimeConst => Ok(Value::Fr(Fr::ZERO)),
        //? MASK > prime, so it can't be represented as an Fr; only valid as And's rhs (below).
        Expr::MaskConst => Err(CircuitError::RuntimeError(
            "MaskConst used outside of And".to_string(),
        )),
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
        //? lhs is already reduced mod p, so this is a no-op (see Inverse/ModPow).
        Expr::BinOp {
            op: BinOpKind::Mod,
            lhs,
            rhs,
        } => {
            debug_assert!(matches!(rhs.as_ref(), Expr::PrimeConst));
            Ok(execute_expr(ctx, lhs)?.into_fr()?.into())
        }
        //? MASK == 2^254 - 1 and every valid Fr value is < prime < 2^254, so this is a no-op.
        Expr::BinOp {
            op: BinOpKind::And,
            lhs,
            rhs,
        } if matches!(rhs.as_ref(), Expr::MaskConst) => execute_expr(ctx, lhs),
        Expr::BinOp { op, lhs, rhs } => {
            let lhs = execute_expr(ctx, lhs)?;
            let rhs = execute_expr(ctx, rhs)?;

            match op {
                BinOpKind::Add => Ok((lhs.into_fr()? + rhs.into_fr()?).into()),
                BinOpKind::Sub => Ok((lhs.into_fr()? - rhs.into_fr()?).into()),
                BinOpKind::Mul => Ok((lhs.into_fr()? * rhs.into_fr()?).into()),
                BinOpKind::Mod => unreachable!("handled above"),
                BinOpKind::Div => {
                    let lhs = fr_to_bigint(lhs.into_fr()?);
                    let rhs = fr_to_bigint(rhs.into_fr()?);
                    Ok(bigint_to_fr(&(lhs / rhs)).into())
                }
                BinOpKind::Eq => Ok((lhs.into_fr()? == rhs.into_fr()?).into()),
                BinOpKind::Neq => Ok((lhs.into_fr()? != rhs.into_fr()?).into()),
                BinOpKind::Lt => Ok((lhs.into_fr()? < rhs.into_fr()?).into()),
                BinOpKind::Gt => Ok((lhs.into_fr()? > rhs.into_fr()?).into()),
                BinOpKind::And => {
                    let lhs = fr_to_bigint(lhs.into_fr()?);
                    let rhs = fr_to_bigint(rhs.into_fr()?);
                    Ok(bigint_to_fr(&(lhs & rhs)).into())
                }
                BinOpKind::Shl => {
                    let lhs = fr_to_bigint(lhs.into_fr()?);
                    let n = rhs.into_u32()?;
                    Ok(bigint_to_fr(&(lhs << n)).into())
                }
                BinOpKind::Shr => {
                    let lhs = fr_to_bigint(lhs.into_fr()?);
                    let n = rhs.into_u32()?;
                    Ok(bigint_to_fr(&(lhs >> n)).into())
                }
            }
        }
        Expr::Inverse(base, modulos) => {
            debug_assert!(matches!(modulos.as_ref(), Expr::PrimeConst));
            let base = execute_expr(ctx, base)?.into_fr()?;
            base.inverse()
                .ok_or(CircuitError::InvalidInverse)
                .map(std::convert::Into::into)
        }
        Expr::ModPow(base, exp, modulos) => {
            debug_assert!(matches!(modulos.as_ref(), Expr::PrimeConst));
            let base = execute_expr(ctx, base)?.into_fr()?;
            let exp = execute_expr(ctx, exp)?.into_u32()?;
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
