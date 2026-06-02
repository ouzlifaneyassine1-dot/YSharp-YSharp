use std::collections::HashMap;
use smol_str::SmolStr;

/// Backend-specific MIR types for WASM codegen
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmMirType { I32, I64, F32, F64, Void }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmBinaryOp { Add, Sub, Mul, Div, Rem, And, Or, Xor, Shl, Shr, Eq, Ne, Lt, Le, Gt, Ge, }
#[derive(Debug, Clone)]
pub enum WasmTerminator { Goto(SmolStr), BranchIf { cond: SmolStr, then_block: SmolStr, else_block: SmolStr }, Return(Option<SmolStr>), Unreachable, }
#[derive(Debug, Clone)]
pub enum WasmMirInst {
    Alloca { dest: SmolStr, ty: WasmMirType },
    Load { dest: SmolStr, src: SmolStr },
    Store { dest: SmolStr, src: SmolStr },
    Binary { dest: SmolStr, op: WasmBinaryOp, lhs: SmolStr, rhs: SmolStr },
    Call { dest: Option<SmolStr>, name: String, args: Vec<SmolStr> },
    Phi { dest: SmolStr, incoming: Vec<(SmolStr, SmolStr)> },
}
#[derive(Debug, Clone)]
pub struct WasmBasicBlock { pub name: SmolStr, pub insts: Vec<WasmMirInst>, pub terminator: WasmTerminator, }
#[derive(Debug, Clone)]
pub struct WasmMirFunction { pub name: String, pub params: Vec<(SmolStr, WasmMirType)>, pub return_type: WasmMirType, pub blocks: Vec<WasmBasicBlock>, }
#[derive(Debug, Clone)]
pub struct WasmMirModule { pub name: String, pub functions: Vec<WasmMirFunction>, pub globals: Vec<WasmMirGlobal>, }
#[derive(Debug, Clone)]
pub struct WasmMirGlobal { pub name: String, pub ty: WasmMirType, pub init: Option<Vec<u8>>, pub mutable: bool, }

pub fn generate(module: &WasmMirModule, output: &str) -> Result<String, String> {
    let wasm_path = if output.ends_with(".wasm") { output.to_string() } else { format!("{}.wasm", output) };

    let mut wasm_bytes = Vec::new();

    // Simple WASM binary emission without wasm-encoder dependency
    // Preamble: WASM magic + version
    wasm_bytes.extend_from_slice(b"\x00asm");
    wasm_bytes.extend_from_slice(&1u32.to_le_bytes()); // version 1

    // Type section
    let mut type_content = Vec::new();
    // Each function needs a type entry: just (params -> return)
    let mut type_indices: HashMap<String, u32> = HashMap::new();
    let type_entries: Vec<(Vec<u8>, Option<u8>)> = module.functions.iter().map(|f| {
        let param_types: Vec<u8> = f.params.iter().map(|(_, t)| wasm_valtype(*t)).collect();
        let ret_type = if f.return_type == WasmMirType::Void { None } else { Some(wasm_valtype(f.return_type)) };
        (param_types, ret_type)
    }).collect();

    // Deduplicate
    let mut deduped: Vec<(Vec<u8>, Option<u8>)> = Vec::new();
    for entry in &type_entries {
        let key = format!("{:?}", entry);
        if !type_indices.contains_key(&key) {
            type_indices.insert(key, deduped.len() as u32);
            deduped.push(entry.clone());
        }
    }

    // Encode type section
    encode_type_section(&mut type_content, &deduped);
    encode_section(&mut wasm_bytes, 1, &type_content); // section 1 = Type

    // Function section
    let mut func_content = Vec::new();
    encode_uleb(&mut func_content, module.functions.len() as u32);
    for f in &module.functions {
        let key = format!("{:?}", (f.params.iter().map(|(_, t)| wasm_valtype(*t)).collect::<Vec<_>>(),
            if f.return_type == WasmMirType::Void { None } else { Some(wasm_valtype(f.return_type)) }));
        let idx = type_indices.get(&key).copied().unwrap_or(0);
        encode_uleb(&mut func_content, idx);
    }
    encode_section(&mut wasm_bytes, 3, &func_content); // section 3 = Function

    // Memory section (default 1 page)
    let mut mem_content = Vec::new();
    mem_content.push(0x00); // limits: min
    encode_uleb(&mut mem_content, 1); // 1 page
    encode_section(&mut wasm_bytes, 5, &mem_content); // section 5 = Memory

    // Export section
    let mut export_content = Vec::new();
    encode_uleb(&mut export_content, (module.functions.len() + 1) as u32);
    for (i, f) in module.functions.iter().enumerate() {
        let name_bytes = f.name.as_bytes();
        encode_uleb(&mut export_content, name_bytes.len() as u32);
        export_content.extend_from_slice(name_bytes);
        export_content.push(0x00); // Func export kind
        encode_uleb(&mut export_content, i as u32);
    }
    // Export memory
    let mem_name = b"memory";
    encode_uleb(&mut export_content, mem_name.len() as u32);
    export_content.extend_from_slice(mem_name);
    export_content.push(0x02); // Mem export kind
    encode_uleb(&mut export_content, 0);
    encode_section(&mut wasm_bytes, 7, &export_content); // section 7 = Export

    // Code section
    let mut code_content = Vec::new();
    encode_uleb(&mut code_content, module.functions.len() as u32);
    for f in &module.functions {
        let mut body = Vec::new();
        // Declare locals: collect all unique SmolStr values used as variables
        let mut locals = Vec::new();
        for b in &f.blocks {
            for inst in &b.insts {
                match inst {
                    WasmMirInst::Alloca { dest, ty } => {
                        locals.push((dest.clone(), *ty));
                    }
                    _ => {}
                }
            }
        }
        // Encode locals (grouped by type)
        if !locals.is_empty() {
            encode_uleb(&mut body, locals.len() as u32);
            for (_, ty) in &locals {
                encode_uleb(&mut body, 1);
                body.push(wasm_valtype(*ty));
            }
        } else {
            encode_uleb(&mut body, 0);
        }

        // Build a local map
        let mut local_map: HashMap<SmolStr, u32> = HashMap::new();
        let mut next_local = f.params.len() as u32;
        for (i, (name, _)) in f.params.iter().enumerate() {
            local_map.insert(name.clone(), i as u32);
        }
        for (name, _) in &locals {
            if !local_map.contains_key(name) {
                local_map.insert(name.clone(), next_local);
                next_local += 1;
            }
        }

        // Block depth map
        let mut block_depths: HashMap<SmolStr, u32> = HashMap::new();
        for (i, b) in f.blocks.iter().enumerate() {
            block_depths.insert(b.name.clone(), (f.blocks.len() - 1 - i) as u32);
        }

        // Emit instructions per block
        for (bi, block) in f.blocks.iter().enumerate() {
            if bi > 0 {
                body.push(0x02); body.push(0x40); // block (empty type)
            }
            for inst in &block.insts {
                wasm_emit_inst(inst, &local_map, &mut body);
            }
            wasm_emit_terminator(&block.terminator, &block_depths, &local_map, &mut body);
        }
        for _ in 1..f.blocks.len() { body.push(0x0B); } // end blocks

        // Write function body size + body
        let mut func_body = Vec::new();
        encode_uleb(&mut func_body, body.len() as u32);
        func_body.extend_from_slice(&body);
        code_content.extend_from_slice(&func_body);
    }
    encode_section(&mut wasm_bytes, 10, &code_content); // section 10 = Code

    std::fs::write(&wasm_path, &wasm_bytes).map_err(|e| format!("failed to write WASM: {}", e))?;
    Ok(wasm_path)
}

fn wasm_valtype(ty: WasmMirType) -> u8 {
    match ty { WasmMirType::I32 => 0x7F, WasmMirType::I64 => 0x7E, WasmMirType::F32 => 0x7D, WasmMirType::F64 => 0x7C, WasmMirType::Void => 0x40, }
}

fn wasm_emit_inst(inst: &WasmMirInst, local_map: &HashMap<SmolStr, u32>, buf: &mut Vec<u8>) {
    match inst {
        WasmMirInst::Alloca { dest, ty: _ } => {
            // Zero-init: i32.const 0; local.set dest
            buf.push(0x41); encode_uleb(buf, 0);
            let idx = local_map.get(dest).copied().unwrap_or(0);
            buf.push(0x21); encode_uleb(buf, idx);
        }
        WasmMirInst::Load { dest, src } => {
            let idx = local_map.get(src).copied().unwrap_or(0);
            buf.push(0x20); encode_uleb(buf, idx);
            let didx = local_map.get(dest).copied().unwrap_or(0);
            buf.push(0x21); encode_uleb(buf, didx);
        }
        WasmMirInst::Store { dest, src } => {
            let sidx = local_map.get(src).copied().unwrap_or(0);
            buf.push(0x20); encode_uleb(buf, sidx);
            let didx = local_map.get(dest).copied().unwrap_or(0);
            buf.push(0x21); encode_uleb(buf, didx);
        }
        WasmMirInst::Binary { dest, op, lhs, rhs } => {
            let li = local_map.get(lhs).copied().unwrap_or(0);
            let ri = local_map.get(rhs).copied().unwrap_or(0);
            buf.push(0x20); encode_uleb(buf, li);
            buf.push(0x20); encode_uleb(buf, ri);
            match op {
                WasmBinaryOp::Add => buf.push(0x6A),
                WasmBinaryOp::Sub => buf.push(0x6B),
                WasmBinaryOp::Mul => buf.push(0x6C),
                WasmBinaryOp::Div => buf.push(0x6D),
                WasmBinaryOp::Rem => buf.push(0x6F),
                WasmBinaryOp::And => buf.push(0x71),
                WasmBinaryOp::Or => buf.push(0x72),
                WasmBinaryOp::Eq => buf.push(0x46),
                WasmBinaryOp::Ne => buf.push(0x47),
                WasmBinaryOp::Lt => buf.push(0x48),
                WasmBinaryOp::Le => buf.push(0x4C),
                WasmBinaryOp::Gt => buf.push(0x4A),
                WasmBinaryOp::Ge => buf.push(0x4E),
                WasmBinaryOp::Shl => buf.push(0x74),
                WasmBinaryOp::Shr => buf.push(0x75),
                WasmBinaryOp::Xor => buf.push(0x73),
            }
            let didx = local_map.get(dest).copied().unwrap_or(0);
            buf.push(0x21); encode_uleb(buf, didx);
        }
        WasmMirInst::Call { dest, name: _, args } => {
            for a in args {
                let ai = local_map.get(a).copied().unwrap_or(0);
                buf.push(0x20); encode_uleb(buf, ai);
            }
            buf.push(0x10); encode_uleb(buf, 0); // call function 0
            if let Some(d) = dest {
                let didx = local_map.get(d).copied().unwrap_or(0);
                buf.push(0x21); encode_uleb(buf, didx);
            }
        }
        WasmMirInst::Phi { dest, incoming: _ } => {
            let didx = local_map.get(dest).copied().unwrap_or(0);
            buf.push(0x21); encode_uleb(buf, didx);
        }
    }
}

fn wasm_emit_terminator(term: &WasmTerminator, block_depths: &HashMap<SmolStr, u32>, local_map: &HashMap<SmolStr, u32>, buf: &mut Vec<u8>) {
    match term {
        WasmTerminator::Goto(target) => {
            if let Some(depth) = block_depths.get(target) {
                if *depth > 0 { buf.push(0x0C); encode_uleb(buf, *depth); }
            }
        }
        WasmTerminator::BranchIf { cond, then_block, else_block } => {
            let ci = local_map.get(cond).copied().unwrap_or(0);
            buf.push(0x20); encode_uleb(buf, ci);
            let td = block_depths.get(then_block).copied().unwrap_or(0);
            let ed = block_depths.get(else_block).copied().unwrap_or(0);
            if td < ed { buf.push(0x0D); encode_uleb(buf, td); if ed > 0 { buf.push(0x0C); encode_uleb(buf, ed); } }
            else { buf.push(0x45); buf.push(0x0D); encode_uleb(buf, ed); if td > 0 { buf.push(0x0C); encode_uleb(buf, td); } }
        }
        WasmTerminator::Return(Some(v)) => {
            let vi = local_map.get(v).copied().unwrap_or(0);
            buf.push(0x20); encode_uleb(buf, vi);
            buf.push(0x0F);
        }
        WasmTerminator::Return(None) => { buf.push(0x0F); }
        WasmTerminator::Unreachable => { buf.push(0x00); }
    }
}

fn encode_uleb(buf: &mut Vec<u8>, mut val: u32) {
    loop { let byte = (val & 0x7F) as u8; val >>= 7; if val == 0 { buf.push(byte); break; } else { buf.push(byte | 0x80); } }
}

fn encode_type_section(buf: &mut Vec<u8>, types: &[(Vec<u8>, Option<u8>)]) {
    encode_uleb(buf, types.len() as u32);
    for (params, ret) in types {
        buf.push(0x60); // functype
        encode_uleb(buf, params.len() as u32);
        buf.extend_from_slice(params);
        match ret {
            Some(r) => { encode_uleb(buf, 1); buf.push(*r); }
            None => { encode_uleb(buf, 0); }
        }
    }
}

fn encode_section(wasm: &mut Vec<u8>, section_id: u8, content: &[u8]) {
    wasm.push(section_id);
    encode_uleb(wasm, content.len() as u32);
    wasm.extend_from_slice(content);
}
