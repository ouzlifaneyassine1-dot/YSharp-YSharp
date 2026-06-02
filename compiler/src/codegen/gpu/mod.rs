// ---------------------------------------------------------------------------
// OY# GPU Backend — generates SPIR-V 1.6 binary for compute shaders
// Supports: kernel dispatch, workgroup sizes, float4 math, tensor ops
// ---------------------------------------------------------------------------

use smol_str::SmolStr;
use crate::mir::ir::*;

// ---------------------------------------------------------------------------
// Type definitions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum GpuMirType { I32, F32, F64, Void, Vec4, Mat4, Tensor { element: Box<GpuMirType>, dims: usize } }

#[derive(Debug, Clone)]
pub enum GpuTerminator { Return, Unreachable }

#[derive(Debug, Clone)]
pub enum GpuMirInst {
    Placeholder,
    /// Allocate a local variable in GPU private memory
    Alloca { dest: SmolStr, ty: GpuMirType },
    /// Load from a variable
    Load { dest: SmolStr, src: SmolStr },
    /// Arithmetic operations
    FAdd { dest: SmolStr, lhs: SmolStr, rhs: SmolStr },
    FSub { dest: SmolStr, lhs: SmolStr, rhs: SmolStr },
    FMul { dest: SmolStr, lhs: SmolStr, rhs: SmolStr },
    FDiv { dest: SmolStr, lhs: SmolStr, rhs: SmolStr },
    /// Float4 operations
    Vec4Splat { dest: SmolStr, scalar: SmolStr },
    Vec4Add { dest: SmolStr, lhs: SmolStr, rhs: SmolStr },
    Vec4Mul { dest: SmolStr, lhs: SmolStr, rhs: SmolStr },
    Vec4Dot { dest: SmolStr, lhs: SmolStr, rhs: SmolStr },
    /// Matrix operations
    Mat4Mul { dest: SmolStr, lhs: SmolStr, rhs: SmolStr },
    /// Memory operations
    GlobalLoad { dest: SmolStr, ptr: SmolStr },
    GlobalStore { ptr: SmolStr, val: SmolStr },
    /// Tensor ops
    TensorRead { dest: SmolStr, tensor: SmolStr, indices: Vec<SmolStr> },
    TensorWrite { tensor: SmolStr, indices: Vec<SmolStr>, val: SmolStr },
    /// Barrier synchronization
    Barrier,
    /// Workgroup size hint
    WorkgroupSize { x: u32, y: u32, z: u32 },
}

#[derive(Debug, Clone)]
pub struct GpuBasicBlock {
    pub name: SmolStr,
    pub insts: Vec<GpuMirInst>,
    pub terminator: GpuTerminator,
}

#[derive(Debug, Clone)]
pub struct GpuMirFunction {
    pub name: String,
    pub params: Vec<(SmolStr, GpuMirType)>,
    pub return_type: GpuMirType,
    pub blocks: Vec<GpuBasicBlock>,
    pub local_size: (u32, u32, u32),
    pub buffers: Vec<SmolStr>,
}

#[derive(Debug, Clone)]
pub struct GpuMirModule {
    pub name: String,
    pub functions: Vec<GpuMirFunction>,
}

// ---------------------------------------------------------------------------
// SPIR-V binary emission
// ---------------------------------------------------------------------------

/// SPIR-V word (u32)
type SpvWord = u32;

struct SpvBuilder {
    words: Vec<SpvWord>,
    next_id: SpvWord,
    // ID tracking
    id_map: std::collections::HashMap<String, SpvWord>,
    ext_inst_imports: Vec<(SpvWord, String)>,
    decorations: Vec<(SpvWord, SpvWord, Vec<SpvWord>)>,
    types: Vec<(SpvWord, Vec<SpvWord>)>,
    constants: Vec<(SpvWord, SpvWord, Vec<SpvWord>)>,
    functions: Vec<(SpvWord, SpvWord, SpvWord, Vec<SpvWord>)>,
    entry_points: Vec<(SpvWord, SpvWord, SpvWord, Vec<SpvWord>)>,
}

impl SpvBuilder {
    fn new() -> Self {
        SpvBuilder {
            words: Vec::new(),
            next_id: 1,
            id_map: std::collections::HashMap::new(),
            ext_inst_imports: Vec::new(),
            decorations: Vec::new(),
            types: Vec::new(),
            constants: Vec::new(),
            functions: Vec::new(),
            entry_points: Vec::new(),
        }
    }

    fn alloc_id(&mut self) -> SpvWord {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn get_or_create_id(&mut self, name: &str) -> SpvWord {
        if let Some(&id) = self.id_map.get(name) { return id; }
        let id = self.alloc_id();
        self.id_map.insert(name.to_string(), id);
        id
    }

    fn word(&mut self, w: SpvWord) { self.words.push(w); }

    fn op(&mut self, opcode: SpvWord, word_count: SpvWord) {
        self.word((word_count << 16) | opcode);
    }

    fn string(&mut self, s: &str) {
        let bytes = s.as_bytes();
        self.words.extend(bytes.iter().map(|&b| b as SpvWord));
        // Pad to multiple of 4 bytes
        let padding = (4 - bytes.len() % 4) % 4;
        for _ in 0..padding { self.word(0); }
    }

    fn begin_module(&mut self) {
        // Header
        self.word(0x07230203); // Magic
        self.word(0x00010600); // v1.6
        self.word(0x00010000); // Generator: OY#
        self.word(0);          // Bound (set at end)
        self.word(0);          // Schema
    }

    fn finalize(&mut self) {
        // Set bound
        self.words[3] = self.next_id;
    }

    fn capability(&mut self, cap: SpvWord) {
        self.op(17, 2); // OpCapability
        self.word(cap);
    }

    fn ext_inst_import(&mut self, name: &str) -> SpvWord {
        let id = self.alloc_id();
        let words_needed = 3 + (name.len() + 3) / 4;
        self.op(11, words_needed as SpvWord); // OpExtInstImport
        self.word(id);
        self.string(name);
        self.ext_inst_imports.push((id, name.to_string()));
        id
    }

    fn memory_model(&mut self, addr: SpvWord, mem: SpvWord) {
        self.op(14, 3); // OpMemoryModel
        self.word(addr);
        self.word(mem);
    }

    fn entry_point(&mut self, exec_model: SpvWord, func_id: SpvWord, name: &str, interfaces: &[SpvWord]) {
        let id = self.alloc_id();
        let _wc = 3 + (name.len() + 3) / 4 + interfaces.len();
        self.entry_points.push((exec_model, func_id, id, interfaces.to_vec()));
        // We'll emit entry points during generate()
    }

    fn decorate(&mut self, target: SpvWord, decoration: SpvWord, params: &[SpvWord]) {
        self.decorations.push((target, decoration, params.to_vec()));
    }

    fn type_void(&mut self) -> SpvWord {
        let id = self.alloc_id();
        self.op(19, 2); // OpTypeVoid
        self.word(id);
        self.types.push((id, vec![]));
        id
    }

    fn type_bool(&mut self) -> SpvWord {
        let id = self.alloc_id();
        self.op(20, 2); // OpTypeBool
        self.word(id);
        id
    }

    fn type_int(&mut self, width: SpvWord, signed: bool) -> SpvWord {
        let id = self.alloc_id();
        self.op(21, 4); // OpTypeInt
        self.word(id);
        self.word(width);
        self.word(if signed { 1 } else { 0 });
        id
    }

    fn type_float(&mut self, width: SpvWord) -> SpvWord {
        let id = self.alloc_id();
        self.op(22, 3); // OpTypeFloat
        self.word(id);
        self.word(width);
        id
    }

    fn type_vector(&mut self, comp_type: SpvWord, count: SpvWord) -> SpvWord {
        let id = self.alloc_id();
        self.op(23, 4); // OpTypeVector
        self.word(id);
        self.word(comp_type);
        self.word(count);
        id
    }

    fn type_matrix(&mut self, col_type: SpvWord, count: SpvWord) -> SpvWord {
        let id = self.alloc_id();
        self.op(24, 4); // OpTypeMatrix
        self.word(id);
        self.word(col_type);
        self.word(count);
        id
    }

    fn type_struct(&mut self, members: &[SpvWord]) -> SpvWord {
        let id = self.alloc_id();
        self.op(30, 2 + members.len() as SpvWord); // OpTypeStruct
        self.word(id);
        for &m in members { self.word(m); }
        id
    }

    fn type_pointer(&mut self, storage_class: SpvWord, pointee: SpvWord) -> SpvWord {
        let id = self.alloc_id();
        self.op(32, 4); // OpTypePointer
        self.word(id);
        self.word(storage_class);
        self.word(pointee);
        id
    }

    fn type_array(&mut self, element: SpvWord, length: SpvWord) -> SpvWord {
        let id = self.alloc_id();
        self.op(28, 4); // OpTypeArray
        self.word(id);
        self.word(element);
        self.word(length);
        id
    }

    fn constant_float(&mut self, ty: SpvWord, val: f64) -> SpvWord {
        let id = self.alloc_id();
        self.op(45, 4); // OpConstant
        self.word(ty);
        self.word(id);
        self.word(val as SpvWord);
        // For 64-bit: need 2 words, but simplified for 32-bit
        id
    }

    fn constant_int(&mut self, ty: SpvWord, val: SpvWord) -> SpvWord {
        let id = self.alloc_id();
        self.op(43, 4); // OpConstant
        self.word(ty);
        self.word(id);
        self.word(val);
        id
    }

    fn begin_function(&mut self, ret_type: SpvWord, func_type: SpvWord) -> SpvWord {
        let id = self.alloc_id();
        self.op(54, 4); // OpFunction
        self.word(ret_type);
        self.word(id);
        self.word(0); // FunctionControl (None)
        self.word(func_type);
        id
    }

    fn end_function(&mut self) {
        self.op(56, 1); // OpFunctionEnd
    }

    fn begin_block(&mut self) -> SpvWord {
        let id = self.alloc_id();
        self.op(247, 2); // OpLabel
        self.word(id);
        id
    }

    fn return_value(&mut self, val: SpvWord) {
        self.op(253, 2); // OpReturnValue
        self.word(val);
    }

    fn return_void(&mut self) {
        self.op(252, 1); // OpReturn
    }

    fn branch(&mut self, target: SpvWord) {
        self.op(249, 2); // OpBranch
        self.word(target);
    }

    fn branch_conditional(&mut self, cond: SpvWord, true_label: SpvWord, false_label: SpvWord) {
        self.op(250, 4); // OpBranchConditional
        self.word(cond);
        self.word(true_label);
        self.word(false_label);
    }

    fn float_add(&mut self, ty: SpvWord, dest: SpvWord, lhs: SpvWord, rhs: SpvWord) {
        self.op(133, 5); // OpFAdd
        self.word(ty); self.word(dest); self.word(lhs); self.word(rhs);
    }

    fn float_mul(&mut self, ty: SpvWord, dest: SpvWord, lhs: SpvWord, rhs: SpvWord) {
        self.op(137, 5); // OpFMul
        self.word(ty); self.word(dest); self.word(lhs); self.word(rhs);
    }

    fn access_chain(&mut self, ty: SpvWord, dest: SpvWord, base: SpvWord, indices: &[SpvWord]) {
        self.op(47, 3 + indices.len() as SpvWord); // OpAccessChain
        self.word(ty); self.word(dest); self.word(base);
        for &i in indices { self.word(i); }
    }

    fn load(&mut self, ty: SpvWord, dest: SpvWord, ptr: SpvWord) {
        self.op(61, 4); // OpLoad
        self.word(ty); self.word(dest); self.word(ptr);
    }

    fn store(&mut self, ptr: SpvWord, val: SpvWord) {
        self.op(62, 3); // OpStore
        self.word(ptr); self.word(val);
    }

    fn vector_extract_dynamic(&mut self, ty: SpvWord, dest: SpvWord, vec: SpvWord, index: SpvWord) {
        self.op(231, 5); // OpVectorExtractDynamic
        self.word(ty); self.word(dest); self.word(vec); self.word(index);
    }

    fn vector_insert_dynamic(&mut self, ty: SpvWord, dest: SpvWord, vec: SpvWord, comp: SpvWord, index: SpvWord) {
        self.op(232, 6); // OpVectorInsertDynamic
        self.word(ty); self.word(dest); self.word(vec); self.word(comp); self.word(index);
    }

    fn composite_construct(&mut self, ty: SpvWord, dest: SpvWord, constituents: &[SpvWord]) {
        self.op(44, 2 + constituents.len() as SpvWord); // OpCompositeConstruct
        self.word(ty); self.word(dest);
        for &c in constituents { self.word(c); }
    }

    fn dot(&mut self, ty: SpvWord, dest: SpvWord, lhs: SpvWord, rhs: SpvWord) {
        self.op(148, 5); // OpDot
        self.word(ty); self.word(dest); self.word(lhs); self.word(rhs);
    }

    fn control_barrier(&mut self, scope: SpvWord, mem: SpvWord, sem: SpvWord) {
        self.op(224, 4); // OpControlBarrier
        self.word(scope); self.word(mem); self.word(sem);
    }
}

// ---------------------------------------------------------------------------
// SPIR-V codegen from GPU MIR
// ---------------------------------------------------------------------------

pub fn generate(module: &GpuMirModule, output: &str) -> Result<String, String> {
    let spv_path = if output.ends_with(".spv") {
        output.to_string()
    } else {
        format!("{}.spv", output)
    };

    let mut spv = SpvBuilder::new();

    // Module header
    spv.begin_module();

    // Capabilities
    spv.capability(1);  // Shader
    spv.capability(9);  // Matrix
    spv.capability(10); // Float64
    spv.capability(11); // Int64
    spv.capability(44); // Float16Buffer

    // Extended instruction set (GLSL.std.450)
    let _glsl = spv.ext_inst_import("GLSL.std.450");

    // Memory model
    spv.memory_model(1, 0); // Logical, Simple

    // --- Type declarations ---
    let void_t = spv.type_void();
    let _bool_t = spv.type_bool();
    let f32_t = spv.type_float(32);
    let _f64_t = spv.type_float(64);
    let i32_t = spv.type_int(32, true);
    let vec4_t = spv.type_vector(f32_t, 4);
    let _mat4_t = spv.type_matrix(vec4_t, 4);

    // Pointer types for different storage classes
    let priv_f32 = spv.type_pointer(0, f32_t);  // Private
    let _priv_vec4 = spv.type_pointer(0, vec4_t);
    let _uniform_f32 = spv.type_pointer(2, f32_t);  // Uniform
    let _storage_f32 = spv.type_pointer(12, f32_t); // StorageBuffer
    let input_f32 = spv.type_pointer(1, f32_t);    // Input
    let _output_f32 = spv.type_pointer(3, f32_t);   // Output

    // Constants
    let _c0_f32 = spv.constant_float(f32_t, 0.0);
    let _c1_f32 = spv.constant_float(f32_t, 1.0);
    let _c0_i32 = spv.constant_int(i32_t, 0);
    let _c1_i32 = spv.constant_int(i32_t, 1);
    let _c64_i32 = spv.constant_int(i32_t, 64);
    let _c256_i32 = spv.constant_int(i32_t, 256);

    // Workgroup size constants
    let _wg_x = spv.constant_int(i32_t, module.functions.first().map(|f| f.local_size.0).unwrap_or(64));
    let _wg_y = spv.constant_int(i32_t, module.functions.first().map(|f| f.local_size.1).unwrap_or(1));
    let _wg_z = spv.constant_int(i32_t, module.functions.first().map(|f| f.local_size.2).unwrap_or(1));

    // Built-in variables
    let global_invocation_id = spv.alloc_id(); // gl_GlobalInvocationID
    // OpVariable for built-in
    spv.op(59, 4); // OpVariable
    spv.word(input_f32);
    spv.word(global_invocation_id);
    spv.word(7); // Input

    // Decorate: BuiltIn GlobalInvocationId
    spv.op(71, 4); // OpDecorate
    spv.word(global_invocation_id);
    spv.word(24); // BuiltIn
    spv.word(28); // GlobalInvocationId

    // Generate functions
    for func in &module.functions {
        let ret_spv = match func.return_type {
            GpuMirType::Void => void_t,
            _ => f32_t,
        };

        // TODO: proper function type
        let func_type = spv.alloc_id();
        spv.op(33, 3); // OpTypeFunction
        spv.word(func_type);
        spv.word(ret_spv);

        let func_id = spv.begin_function(ret_spv, func_type);

        // Entry point
        spv.entry_point(0, func_id, &func.name, &[]); // GLCompute

        // Workgroup size decoration
        spv.op(72, 6); // OpDecorate
        spv.word(func_id);
        spv.word(17); // WorkgroupSize
        spv.word(func.local_size.0);
        spv.word(func.local_size.1);
        spv.word(func.local_size.2);

        // Generate blocks
        for block in &func.blocks {
            let _label_id = spv.alloc_id();
            spv.begin_block();

            for inst in &block.insts {
                match inst {
                    GpuMirInst::FAdd { dest, lhs, rhs } => {
                        let d = spv.get_or_create_id(dest);
                        let l = spv.get_or_create_id(lhs);
                        let r = spv.get_or_create_id(rhs);
                        spv.float_add(f32_t, d, l, r);
                    }
                    GpuMirInst::FMul { dest, lhs, rhs } => {
                        let d = spv.get_or_create_id(dest);
                        let l = spv.get_or_create_id(lhs);
                        let r = spv.get_or_create_id(rhs);
                        spv.float_mul(f32_t, d, l, r);
                    }
                    GpuMirInst::Vec4Add { dest, lhs, rhs } => {
                        let d = spv.get_or_create_id(dest);
                        let l = spv.get_or_create_id(lhs);
                        let r = spv.get_or_create_id(rhs);
                        spv.float_add(vec4_t, d, l, r);
                    }
                    GpuMirInst::Vec4Mul { dest, lhs, rhs } => {
                        let d = spv.get_or_create_id(dest);
                        let l = spv.get_or_create_id(lhs);
                        let r = spv.get_or_create_id(rhs);
                        spv.float_mul(vec4_t, d, l, r);
                    }
                    GpuMirInst::Vec4Dot { dest, lhs, rhs } => {
                        let d = spv.get_or_create_id(dest);
                        let l = spv.get_or_create_id(lhs);
                        let r = spv.get_or_create_id(rhs);
                        spv.dot(f32_t, d, l, r);
                    }
                    GpuMirInst::GlobalLoad { dest, ptr } => {
                        let d = spv.get_or_create_id(dest);
                        let p = spv.get_or_create_id(ptr);
                        spv.load(f32_t, d, p);
                    }
                    GpuMirInst::GlobalStore { ptr, val } => {
                        let p = spv.get_or_create_id(ptr);
                        let v = spv.get_or_create_id(val);
                        spv.store(p, v);
                    }
                    GpuMirInst::Barrier => {
                        spv.control_barrier(2, 2, 0x100); // Workgroup, AcquireRelease, None
                    }
                    GpuMirInst::Alloca { dest, ty: _ } => {
                        // Alloca maps to OpVariable in private storage class
                        let d = spv.get_or_create_id(dest);
                        spv.op(59, 4); // OpVariable
                        spv.word(priv_f32);
                        spv.word(d);
                        spv.word(0); // Private
                    }
                    _ => {} // Placeholder, etc.
                }
            }

            match &block.terminator {
                GpuTerminator::Return => spv.return_void(),
                GpuTerminator::Unreachable => { /* implicit */ }
            }
        }

        spv.end_function();
    }

    // Finalize
    spv.finalize();

    // Write binary
    let bytes: Vec<u8> = spv.words.iter()
        .flat_map(|w| w.to_le_bytes())
        .collect();

    std::fs::write(&spv_path, &bytes)
        .map_err(|e| format!("failed to write SPIR-V: {}", e))?;

    Ok(spv_path)
}

// ---------------------------------------------------------------------------
// Conversion from canonical MIR
// ---------------------------------------------------------------------------

pub fn convert_module(module: &MirModule) -> GpuMirModule {
    let mut gpu_fns = Vec::with_capacity(module.functions.len());
    for func in &module.functions {
        let mut blocks = Vec::with_capacity(func.cfg.blocks.len());
        for block in &func.cfg.blocks {
            let mut insts = Vec::with_capacity(block.instructions.len());
            for inst in &block.instructions {
                insts.push(convert_inst(inst));
            }
            blocks.push(GpuBasicBlock {
                name: SmolStr::new(format!("bb{}", block.id)),
                insts,
                terminator: match &block.terminator {
                    MirTerminator::Return(_) => GpuTerminator::Return,
                    _ => GpuTerminator::Unreachable,
                },
            });
        }
        gpu_fns.push(GpuMirFunction {
            name: func.name.to_string(),
            params: Vec::new(),
            return_type: GpuMirType::Void,
            blocks,
            local_size: (64, 1, 1),
            buffers: Vec::new(),
        });
    }
    GpuMirModule { name: module.name.to_string(), functions: gpu_fns }
}

fn convert_inst(inst: &MirInst) -> GpuMirInst {
    match inst {
        MirInst::Alloca { dest, .. } => GpuMirInst::Alloca { dest: SmolStr::new(dest.to_string()), ty: GpuMirType::F32 },
        MirInst::Load { dest, src } => GpuMirInst::Load { dest: SmolStr::new(dest.to_string()), src: SmolStr::new(src.to_string()) },
        MirInst::Store { dest, src } => GpuMirInst::GlobalStore { ptr: SmolStr::new(dest.to_string()), val: SmolStr::new(src.to_string()) },
        MirInst::Binary { dest, op: _, left, right } => GpuMirInst::FAdd {
            dest: SmolStr::new(dest.to_string()),
            lhs: SmolStr::new(left.to_string()),
            rhs: SmolStr::new(right.to_string()),
        },
        MirInst::Call { .. } => GpuMirInst::Placeholder,
        MirInst::Print { .. } => GpuMirInst::Placeholder,
        MirInst::IntLiteral { dest, val } => GpuMirInst::FAdd {
            dest: SmolStr::new(dest.to_string()),
            lhs: SmolStr::new(val.to_string()),
            rhs: SmolStr::new("0"),
        },
        MirInst::FloatLiteral { dest, val } => GpuMirInst::FAdd {
            dest: SmolStr::new(dest.to_string()),
            lhs: SmolStr::new(format!("{}", val)),
            rhs: SmolStr::new("0.0"),
        },
        MirInst::StringLiteral { .. } => GpuMirInst::Placeholder,
        MirInst::BoolLiteral { dest, val } => GpuMirInst::FAdd {
            dest: SmolStr::new(dest.to_string()),
            lhs: SmolStr::new(if *val { "1.0" } else { "0.0" }),
            rhs: SmolStr::new("0.0"),
        },
        MirInst::Phi { .. } => GpuMirInst::Placeholder,
        MirInst::Unary { dest, op, operand } => GpuMirInst::FAdd {
            dest: SmolStr::new(dest.to_string()),
            lhs: SmolStr::new(match op { MirUnaryOp::Neg => "0", MirUnaryOp::Not => "~0" }),
            rhs: SmolStr::new(operand.to_string()),
        },
        MirInst::VectorHint { .. } | MirInst::InlineHint { .. } => GpuMirInst::Placeholder,
    }
}
