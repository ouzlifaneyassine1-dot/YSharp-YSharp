use std::collections::HashMap;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::types::BasicTypeEnum;
use inkwell::values::{
    AnyValueEnum, BasicValueEnum, FloatValue, FunctionValue, IntValue, PhiValue,
    PointerValue,
};
use inkwell::{FloatPredicate, IntPredicate};

use smol_str::SmolStr;

use crate::mir::{BinaryOp, MirType, MirValue};

pub struct LlvmBuilder<'ctx> {
    builder: Builder<'ctx>,
    values: HashMap<MirValue, AnyValueEnum<'ctx>>,
    context: &'ctx Context,
}

impl<'ctx> LlvmBuilder<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self {
            builder: context.create_builder(),
            values: HashMap::new(),
            context,
        }
    }

    pub fn context(&self) -> &'ctx Context {
        self.context
    }

    pub fn builder(&self) -> &Builder<'ctx> {
        &self.builder
    }

    pub fn builder_mut(&mut self) -> &mut Builder<'ctx> {
        &mut self.builder
    }

    // -- Value map management ---------------------------------------------------

    pub fn bind(&mut self, name: MirValue, value: AnyValueEnum<'ctx>) {
        self.values.insert(name, value);
    }

    pub fn lookup(&self, name: &MirValue) -> Option<AnyValueEnum<'ctx>> {
        self.values.get(name).copied()
    }

    pub fn lookup_basic(&self, name: &MirValue) -> Result<BasicValueEnum<'ctx>, String> {
        self.lookup(name)
            .and_then(|v| v.into_basic_value())
            .ok_or_else(|| format!("value `{}` not found or not a basic value", name))
    }

    pub fn lookup_ptr(&self, name: &MirValue) -> Result<PointerValue<'ctx>, String> {
        self.lookup(name)
            .and_then(|v| v.into_pointer_value())
            .ok_or_else(|| format!("value `{}` not found or not a pointer", name))
    }

    pub fn lookup_int(&self, name: &MirValue) -> Result<IntValue<'ctx>, String> {
        self.lookup(name)
            .and_then(|v| v.into_int_value())
            .ok_or_else(|| format!("value `{}` not found or not an integer", name))
    }

    pub fn lookup_float(&self, name: &MirValue) -> Result<FloatValue<'ctx>, String> {
        self.lookup(name)
            .and_then(|v| v.into_float_value())
            .ok_or_else(|| format!("value `{}` not found or not a float", name))
    }

    // -- Type mapping -----------------------------------------------------------

    pub fn mir_type_to_basic(&self, ty: &MirType) -> Result<BasicTypeEnum<'ctx>, String> {
        match ty {
            MirType::I8 | MirType::U8 => Ok(self.context.i8_type().into()),
            MirType::I16 | MirType::U16 => Ok(self.context.i16_type().into()),
            MirType::I32 | MirType::U32 => Ok(self.context.i32_type().into()),
            MirType::I64 | MirType::U64 => Ok(self.context.i64_type().into()),
            MirType::F32 => Ok(self.context.f32_type().into()),
            MirType::F64 => Ok(self.context.f64_type().into()),
            MirType::Bool => Ok(self.context.bool_type().into()),
            MirType::Void => Err("void has no basic type representation".to_string()),
            MirType::Ptr(_) | MirType::Array(_, _) | MirType::Struct(_) => {
                Ok(self.context.ptr_type(inkwell::AddressSpace::Generic).into())
            }
            MirType::Func(_, _) => Err("function type cannot be a basic value".to_string()),
        }
    }

    // -- Instruction building ---------------------------------------------------

    pub fn build_alloca(&mut self, dest: &MirValue, ty: &MirType) -> Result<PointerValue<'ctx>, String> {
        let llvm_ty = self.mir_type_to_basic(ty)?;
        let ptr = self.builder.build_alloca(llvm_ty, dest.as_str());
        self.bind(dest.clone(), ptr.into());
        Ok(ptr)
    }

    pub fn build_load(&mut self, dest: &MirValue, src: &MirValue) -> Result<(), String> {
        let ptr = self.lookup_ptr(src)?;
        let loaded = self.builder.build_load(ptr, dest.as_str());
        self.bind(dest.clone(), loaded);
        Ok(())
    }

    pub fn build_store(&mut self, dest: &MirValue, src: &MirValue) -> Result<(), String> {
        let ptr = self.lookup_ptr(dest)?;
        let val = self.lookup_basic(src)?;
        self.builder.build_store(ptr, val);
        Ok(())
    }

    pub fn build_gep(&mut self, dest: &MirValue, ptr: &MirValue, indices: &[MirValue]) -> Result<(), String> {
        let base_ptr = self.lookup_ptr(ptr)?;
        let index_vals: Vec<IntValue<'ctx>> = indices
            .iter()
            .map(|i| self.lookup_int(i))
            .collect::<Result<Vec<_>, _>>()?;
        let index_refs: Vec<&IntValue<'ctx>> = index_vals.iter().collect();
        let elem_ptr = unsafe {
            self.builder
                .build_gep(base_ptr, &index_refs, dest.as_str())
        };
        self.bind(dest.clone(), elem_ptr.into());
        Ok(())
    }

    pub fn build_binary(&mut self, dest: &MirValue, op: BinaryOp, lhs: &MirValue, rhs: &MirValue) -> Result<(), String> {
        let l = self.lookup(lhs).ok_or_else(|| format!("value `{}` not found", lhs))?;
        let r = self.lookup(rhs).ok_or_else(|| format!("value `{}` not found", rhs))?;

        let result = match (l, r) {
            (AnyValueEnum::IntValue(a), AnyValueEnum::IntValue(b)) => {
                self.build_int_binary(op, a, b, dest.as_str())?.into()
            }
            (AnyValueEnum::FloatValue(a), AnyValueEnum::FloatValue(b)) => {
                self.build_float_binary(op, a, b, dest.as_str())?.into()
            }
            _ => return Err("type mismatch in binary operation".to_string()),
        };

        self.bind(dest.clone(), result);
        Ok(())
    }

    fn build_int_binary(
        &self,
        op: BinaryOp,
        lhs: IntValue<'ctx>,
        rhs: IntValue<'ctx>,
        name: &str,
    ) -> Result<IntValue<'ctx>, String> {
        match op {
            BinaryOp::Add => Ok(self.builder.build_int_add(lhs, rhs, name)),
            BinaryOp::Sub => Ok(self.builder.build_int_sub(lhs, rhs, name)),
            BinaryOp::Mul => Ok(self.builder.build_int_mul(lhs, rhs, name)),
            BinaryOp::Div => Ok(self.builder.build_int_signed_div(lhs, rhs, name)),
            BinaryOp::Rem => Ok(self.builder.build_int_signed_rem(lhs, rhs, name)),
            BinaryOp::And => Ok(self.builder.build_and(lhs, rhs, name)),
            BinaryOp::Or => Ok(self.builder.build_or(lhs, rhs, name)),
            BinaryOp::Xor => Ok(self.builder.build_xor(lhs, rhs, name)),
            BinaryOp::Shl => Ok(self.builder.build_left_shift(lhs, rhs, name)),
            BinaryOp::Shr => Ok(self.builder.build_right_shift(lhs, rhs, true, name)),
            BinaryOp::Eq => Ok(self.builder.build_int_compare(IntPredicate::EQ, lhs, rhs, name)),
            BinaryOp::Ne => Ok(self.builder.build_int_compare(IntPredicate::NE, lhs, rhs, name)),
            BinaryOp::Lt => Ok(self.builder.build_int_compare(IntPredicate::SLT, lhs, rhs, name)),
            BinaryOp::Le => Ok(self.builder.build_int_compare(IntPredicate::SLE, lhs, rhs, name)),
            BinaryOp::Gt => Ok(self.builder.build_int_compare(IntPredicate::SGT, lhs, rhs, name)),
            BinaryOp::Ge => Ok(self.builder.build_int_compare(IntPredicate::SGE, lhs, rhs, name)),
        }
    }

    fn build_float_binary(
        &self,
        op: BinaryOp,
        lhs: FloatValue<'ctx>,
        rhs: FloatValue<'ctx>,
        name: &str,
    ) -> Result<FloatValue<'ctx>, String> {
        let (l, r) = (lhs, rhs);
        match op {
            BinaryOp::Add => Ok(self.builder.build_float_add(l, r, name)),
            BinaryOp::Sub => Ok(self.builder.build_float_sub(l, r, name)),
            BinaryOp::Mul => Ok(self.builder.build_float_mul(l, r, name)),
            BinaryOp::Div => Ok(self.builder.build_float_div(l, r, name)),
            BinaryOp::Rem => Ok(self.builder.build_float_rem(l, r, name)),
            BinaryOp::Eq => Ok(self.builder.build_float_compare(FloatPredicate::OEQ, l, r, name)),
            BinaryOp::Ne => Ok(self.builder.build_float_compare(FloatPredicate::ONE, l, r, name)),
            BinaryOp::Lt => Ok(self.builder.build_float_compare(FloatPredicate::OLT, l, r, name)),
            BinaryOp::Le => Ok(self.builder.build_float_compare(FloatPredicate::OLE, l, r, name)),
            BinaryOp::Gt => Ok(self.builder.build_float_compare(FloatPredicate::OGT, l, r, name)),
            BinaryOp::Ge => Ok(self.builder.build_float_compare(FloatPredicate::OGE, l, r, name)),
            other => Err(format!("unsupported float binary op: {:?}", other)),
        }
    }

    pub fn build_call(
        &mut self,
        dest: Option<&MirValue>,
        callee: FunctionValue<'ctx>,
        args: &[MirValue],
    ) -> Result<(), String> {
        let arg_vals: Vec<BasicValueEnum<'ctx>> = args
            .iter()
            .map(|a| self.lookup_basic(a))
            .collect::<Result<Vec<_>, _>>()?;
        let arg_refs: Vec<&BasicValueEnum<'ctx>> = arg_vals.iter().collect();
        let call_site = self.builder.build_call(callee, &arg_refs, "call");
        if let Some(d) = dest {
            if let Some(ret) = call_site.try_as_basic_value().left() {
                self.bind(d.clone(), ret.into());
            }
        }
        Ok(())
    }

    pub fn build_phi(
        &mut self,
        dest: &MirValue,
        ty: &MirType,
    ) -> Result<PhiValue<'ctx>, String> {
        let llvm_ty = self.mir_type_to_basic(ty)?;
        let phi = self.builder.build_phi(llvm_ty, dest.as_str());
        self.bind(dest.clone(), phi.as_any_value_enum());
        Ok(phi)
    }

    pub fn build_br(&mut self, target: &inkwell::basic_block::BasicBlock<'ctx>) -> Result<(), String> {
        self.builder.build_unconditional_br(*target);
        Ok(())
    }

    pub fn build_cond_br(
        &mut self,
        cond: &MirValue,
        then_block: &inkwell::basic_block::BasicBlock<'ctx>,
        else_block: &inkwell::basic_block::BasicBlock<'ctx>,
    ) -> Result<(), String> {
        let cond_val = self.lookup_int(cond)?;
        let zero = self.context.i64_type().const_zero();
        let cmp = self
            .builder
            .build_int_compare(IntPredicate::NE, cond_val, zero, "cond");
        self.builder.build_conditional_br(cmp, *then_block, *else_block);
        Ok(())
    }

    pub fn build_ret(&mut self, value: Option<&MirValue>) -> Result<(), String> {
        match value {
            Some(v) => {
                let val = self.lookup_basic(v)?;
                self.builder.build_return(Some(&val));
            }
            None => {
                self.builder.build_return(None);
            }
        }
        Ok(())
    }
}
