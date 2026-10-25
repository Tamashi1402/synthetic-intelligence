//! The two shaders the viewer needs, assembled as SPIR-V words.
//!
//! **Why hand-built:** [00 §9]'s allowlist is compute-only and closed — there is
//! no `glslang`/`shaderc` row, and a `build.rs` that shells out to a system
//! compiler would make the build depend on a tool the gate harness does not pin
//! (**OD-147**: nothing in `engineering/` states where viewer SPIR-V comes from,
//! while [08 §3.1] says the atlas has "no build step" and this module holds that
//! line). A viewer needs exactly two programs — a pass-through vertex stage and a
//! sampling fragment stage — and both are small enough to emit as words rather
//! than to gain a dependency for. The numbers below are the Khronos
//! `spirv.core.grammar` opcode and enum values, transcribed, and the unit tests at
//! the bottom walk the word stream so a miscounted instruction fails a CPU test
//! instead of arriving as a driver error at runtime.
//!
//! Both stages are presentation: they read the vertex buffer the panels wrote and
//! the glyph sheet [`crate::atlas::bake`] baked. Neither touches a state byte,
//! which is what lets §5's layout rule and §9's widget ban survive in the one
//! place a GUI could have hidden a control.

/// SPIR-V's magic number, the module's first word.
pub const MAGIC: u32 = 0x0723_0203;
/// Version 1.0 — the oldest module a Vulkan 1.0 driver must accept, and enough
/// for everything here (no initializer-bearing globals, no descriptor indexing).
pub const VERSION: u32 = 0x0001_0000;
/// Generator number 0: "unknown", which is honest for a hand-built module.
const GENERATOR: u32 = 0;

/// `SpvOp` values, from the core grammar.
mod op {
    pub const MEMORY_MODEL: u32 = 14;
    pub const ENTRY_POINT: u32 = 15;
    pub const EXECUTION_MODE: u32 = 16;
    pub const CAPABILITY: u32 = 17;
    pub const TYPE_VOID: u32 = 19;
    pub const TYPE_FLOAT: u32 = 22;
    pub const TYPE_VECTOR: u32 = 23;
    pub const TYPE_IMAGE: u32 = 25;
    pub const TYPE_SAMPLER: u32 = 26;
    pub const TYPE_SAMPLED_IMAGE: u32 = 27;
    pub const TYPE_POINTER: u32 = 32;
    pub const TYPE_FUNCTION: u32 = 33;
    pub const FUNCTION: u32 = 54;
    pub const FUNCTION_END: u32 = 56;
    pub const VARIABLE: u32 = 59;
    pub const LOAD: u32 = 61;
    pub const STORE: u32 = 62;
    pub const DECORATE: u32 = 71;
    pub const IMAGE_SAMPLE_IMPLICIT_LOD: u32 = 87;
    pub const FMUL: u32 = 133;
    pub const LABEL: u32 = 248;
    pub const RETURN: u32 = 253;
}

/// `SpvExecutionModel` / `SpvDecoration` / `SpvStorageClass` and friends.
mod e {
    pub const VERTEX: u32 = 0;
    pub const FRAGMENT: u32 = 4;
    pub const LOGICAL: u32 = 0;
    pub const GLSL_450: u32 = 1;
    pub const ORIGIN_UPPER_LEFT: u32 = 7;
    pub const BUILT_IN: u32 = 11;
    pub const LOCATION: u32 = 30;
    pub const BINDING: u32 = 33;
    pub const DESCRIPTOR_SET: u32 = 34;
    pub const POSITION: u32 = 0;
    pub const UNIFORM_CONSTANT: u32 = 0;
    pub const INPUT: u32 = 1;
    pub const OUTPUT: u32 = 3;
    pub const SHADER: u32 = 1;
    pub const DIM_2D: u32 = 1;
    pub const FORMAT_UNKNOWN: u32 = 0;
    pub const NONE: u32 = 0;
}

/// A module under construction: an id counter and the instruction words, with the
/// header written last because the id bound is only known at the end.
struct Builder {
    words: Vec<u32>,
    next_id: u32,
}

impl Builder {
    fn new() -> Self {
        Self { words: Vec::new(), next_id: 1 }
    }

    fn id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn op(&mut self, opcode: u32, operands: &[u32]) {
        let count = operands.len() as u32 + 1;
        self.words.push((count << 16) | opcode);
        self.words.extend_from_slice(operands);
    }

    /// A literal string, null-terminated and padded to a whole word, as §2.3
    /// spells it.
    fn literal(text: &str) -> Vec<u32> {
        let mut bytes: Vec<u8> = text.bytes().collect();
        bytes.push(0);
        while bytes.len() % 4 != 0 {
            bytes.push(0);
        }
        bytes
            .chunks_exact(4)
            .map(|word| u32::from_le_bytes(word.try_into().expect("4 bytes")))
            .collect()
    }

    fn decorate(&mut self, target: u32, decoration: u32, argument: u32) {
        self.op(op::DECORATE, &[target, decoration, argument]);
    }

    /// The finished module: header, then every instruction in the order pushed.
    fn finish(self) -> Vec<u32> {
        let mut out = Vec::with_capacity(5 + self.words.len());
        out.extend_from_slice(&[MAGIC, VERSION, GENERATOR, self.next_id, 0]);
        out.extend_from_slice(&self.words);
        out
    }
}

/// The pass-through vertex stage: position, uv and colour in, the same three out.
///
/// Positions are already in clip space because the CPU wrote them from
/// [`crate::layout::pos`] scaled into the viewport — there is no matrix in the
/// shader, which is the point: a camera that could re-place a point would be a
/// second layout, and §5.1 says positions are a function of identity alone.
pub fn vertex_module() -> Vec<u32> {
    let mut b = Builder::new();
    let void = b.id();
    let float = b.id();
    let vec2 = b.id();
    let vec4 = b.id();
    let ptr_in_vec4 = b.id();
    let ptr_in_vec2 = b.id();
    let ptr_out_vec2 = b.id();
    let ptr_out_vec4 = b.id();
    let fn_void = b.id();
    let main = b.id();
    let entry = b.id();
    let in_pos = b.id();
    let in_uv = b.id();
    let in_color = b.id();
    let out_uv = b.id();
    let out_color = b.id();
    let gl_position = b.id();
    let load_pos = b.id();
    let load_uv = b.id();
    let load_color = b.id();

    b.op(op::CAPABILITY, &[e::SHADER]);
    b.op(op::MEMORY_MODEL, &[e::LOGICAL, e::GLSL_450]);
    b.op_with_string_entry(
        main,
        e::VERTEX,
        "main",
        &[in_pos, in_uv, in_color, out_uv, out_color, gl_position],
    );
    b.decorate(in_pos, e::LOCATION, 0);
    b.decorate(in_uv, e::LOCATION, 1);
    b.decorate(in_color, e::LOCATION, 2);
    b.decorate(out_uv, e::LOCATION, 0);
    b.decorate(out_color, e::LOCATION, 1);
    b.decorate(gl_position, e::BUILT_IN, e::POSITION);

    b.op(op::TYPE_VOID, &[void]);
    b.op(op::TYPE_FLOAT, &[float, 32]);
    b.op(op::TYPE_VECTOR, &[vec2, float, 2]);
    b.op(op::TYPE_VECTOR, &[vec4, float, 4]);
    b.op(op::TYPE_POINTER, &[ptr_in_vec4, e::INPUT, vec4]);
    b.op(op::TYPE_POINTER, &[ptr_in_vec2, e::INPUT, vec2]);
    b.op(op::TYPE_POINTER, &[ptr_out_vec2, e::OUTPUT, vec2]);
    b.op(op::TYPE_POINTER, &[ptr_out_vec4, e::OUTPUT, vec4]);
    b.op(op::TYPE_FUNCTION, &[fn_void, void]);
    for (target, class, pointer) in [
        (in_pos, e::INPUT, ptr_in_vec4),
        (in_uv, e::INPUT, ptr_in_vec2),
        (in_color, e::INPUT, ptr_in_vec4),
        (out_uv, e::OUTPUT, ptr_out_vec2),
        (out_color, e::OUTPUT, ptr_out_vec4),
        (gl_position, e::OUTPUT, ptr_out_vec4),
    ] {
        b.op(op::VARIABLE, &[pointer, target, class]);
    }

    b.op(op::FUNCTION, &[void, main, e::NONE, fn_void]);
    b.op(op::LABEL, &[entry]);
    b.op(op::LOAD, &[vec4, load_pos, in_pos]);
    b.op(op::STORE, &[gl_position, load_pos]);
    b.op(op::LOAD, &[vec2, load_uv, in_uv]);
    b.op(op::STORE, &[out_uv, load_uv]);
    b.op(op::LOAD, &[vec4, load_color, in_color]);
    b.op(op::STORE, &[out_color, load_color]);
    b.op(op::RETURN, &[]);
    b.op(op::FUNCTION_END, &[]);
    b.finish()
}

/// The fragment stage: one combined image sampler (the glyph sheet) multiplied by
/// the vertex colour. `atlas * colour` is the whole of it — the sheet carries the
/// glyph as alpha, so text colour and coverage compose without a branch, and the
/// background is the clear colour showing through a zero alpha.
pub fn fragment_module() -> Vec<u32> {
    let mut b = Builder::new();
    let void = b.id();
    let float = b.id();
    let vec2 = b.id();
    let vec4 = b.id();
    let image = b.id();
    let sampler = b.id();
    let sampled = b.id();
    let ptr_uniform = b.id();
    let ptr_in_vec2 = b.id();
    let ptr_in_vec4 = b.id();
    let ptr_out_vec4 = b.id();
    let fn_void = b.id();
    let main = b.id();
    let entry = b.id();
    let atlas = b.id();
    let in_uv = b.id();
    let in_color = b.id();
    let out_color = b.id();
    let atlas_value = b.id();
    let load_uv = b.id();
    let load_color = b.id();
    let texel = b.id();
    let mixed = b.id();

    b.op(op::CAPABILITY, &[e::SHADER]);
    b.op(op::MEMORY_MODEL, &[e::LOGICAL, e::GLSL_450]);
    // The atlas is bound by its `DescriptorSet`/`Binding` decorations and is *not*
    // named in the interface: SPIR-V 1.0–1.3 lets an entry point list only
    // `Input`/`Output` variables, and `spirv-val` refuses a `UniformConstant` id
    // there. The module version stays 1.0 — the oldest a Vulkan 1.0 driver must
    // accept — so the sampler is reached only through its decoration.
    b.op_with_string_entry(main, e::FRAGMENT, "main", &[in_uv, in_color, out_color]);
    b.op(op::EXECUTION_MODE, &[main, e::ORIGIN_UPPER_LEFT]);
    b.decorate(atlas, e::DESCRIPTOR_SET, 0);
    b.decorate(atlas, e::BINDING, 0);
    b.decorate(in_uv, e::LOCATION, 0);
    b.decorate(in_color, e::LOCATION, 1);
    b.decorate(out_color, e::LOCATION, 0);

    b.op(op::TYPE_VOID, &[void]);
    b.op(op::TYPE_FLOAT, &[float, 32]);
    b.op(op::TYPE_VECTOR, &[vec2, float, 2]);
    b.op(op::TYPE_VECTOR, &[vec4, float, 4]);
    // sampledType, Dim, Depth, Arrayed, MS, Sampled, Format.
    b.op(op::TYPE_IMAGE, &[image, float, e::DIM_2D, 0, 0, 0, 1, e::FORMAT_UNKNOWN]);
    b.op(op::TYPE_SAMPLER, &[sampler]);
    b.op(op::TYPE_SAMPLED_IMAGE, &[sampled, image]);
    b.op(op::TYPE_POINTER, &[ptr_uniform, e::UNIFORM_CONSTANT, sampled]);
    b.op(op::TYPE_POINTER, &[ptr_in_vec2, e::INPUT, vec2]);
    b.op(op::TYPE_POINTER, &[ptr_in_vec4, e::INPUT, vec4]);
    b.op(op::TYPE_POINTER, &[ptr_out_vec4, e::OUTPUT, vec4]);
    b.op(op::TYPE_FUNCTION, &[fn_void, void]);
    b.op(op::VARIABLE, &[ptr_uniform, atlas, e::UNIFORM_CONSTANT]);
    b.op(op::VARIABLE, &[ptr_in_vec2, in_uv, e::INPUT]);
    b.op(op::VARIABLE, &[ptr_in_vec4, in_color, e::INPUT]);
    b.op(op::VARIABLE, &[ptr_out_vec4, out_color, e::OUTPUT]);

    b.op(op::FUNCTION, &[void, main, e::NONE, fn_void]);
    b.op(op::LABEL, &[entry]);
    b.op(op::LOAD, &[vec2, load_uv, in_uv]);
    b.op(op::LOAD, &[vec4, load_color, in_color]);
    // The image operand of a sampling instruction is a *value* of
    // `OpTypeSampledImage`, not the `UniformConstant` pointer the descriptor binds.
    // Skipping this load is legal-looking in the word stream and impossible to the
    // driver: the NVIDIA compiler answers `NVVM compilation failed` for it.
    b.op(op::LOAD, &[sampled, atlas_value, atlas]);
    b.op(op::IMAGE_SAMPLE_IMPLICIT_LOD, &[vec4, texel, atlas_value, load_uv]);
    b.op(op::FMUL, &[vec4, mixed, load_color, texel]);
    b.op(op::STORE, &[out_color, mixed]);
    b.op(op::RETURN, &[]);
    b.op(op::FUNCTION_END, &[]);
    b.finish()
}

impl Builder {
    /// `OpEntryPoint <model> %main "main" <interface…>` — the one instruction
    /// whose operands are split by a literal string.
    fn op_with_string_entry(&mut self, entry_id: u32, model: u32, name: &str, interface: &[u32]) {
        let mut operands = vec![model, entry_id];
        operands.extend(Self::literal(name));
        operands.extend_from_slice(interface);
        self.op(op::ENTRY_POINT, &operands);
    }
}

/// One instruction of a finished module, as a validator-lite reader sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Instruction {
    pub opcode: u32,
    pub word_count: u32,
}

/// Walk a module's instruction stream. Every word after the header belongs to an
/// instruction whose own header declares its length, so a miscount is detectable
/// here rather than only by a driver: the walk returns `None` for a stream that
/// does not divide exactly into the instruction word counts.
///
/// This is a **structure** check, not a validator: operands are not typed, so no
/// subset-relationship between an id and the bound is claimed here.
pub fn instructions(words: &[u32]) -> Option<Vec<Instruction>> {
    if words.len() < 5 || words[0] != MAGIC || words[3] == 0 {
        return None;
    }
    let mut index = 5usize;
    let mut out = Vec::new();
    while index < words.len() {
        let head = words[index];
        let word_count = head >> 16;
        let opcode = head & 0xffff;
        let end = index + word_count as usize;
        // `OpReturn` and `OpFunctionEnd` are a header alone: WordCount 1 is legal.
        if word_count < 1 || end > words.len() {
            return None;
        }
        out.push(Instruction { opcode, word_count });
        index = end;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::{instructions, op, fragment_module, vertex_module, MAGIC, VERSION};

    #[test]
    fn both_modules_are_well_formed_word_streams() {
        for words in [vertex_module(), fragment_module()] {
            assert_eq!(words[0], MAGIC);
            assert_eq!(words[1], VERSION);
            let body = instructions(&words).expect("a walkable instruction stream");
            // The stream consumes every word: `instructions` returns None otherwise.
            let walked: usize = body.iter().map(|i| i.word_count as usize).sum::<usize>() + 5;
            assert_eq!(walked, words.len(), "a word left over means a bad count");
            assert_eq!(body[0].opcode, op::CAPABILITY, "§2.3's layout order");
            assert_eq!(body[1].opcode, op::MEMORY_MODEL);
            assert_eq!(body[2].opcode, op::ENTRY_POINT);
            let last = body.last().expect("a function body");
            assert_eq!(last.opcode, op::FUNCTION_END);
            assert!(body.iter().any(|i| i.opcode == op::RETURN));
        }
    }

    #[test]
    fn the_fragment_stage_declares_the_atlas_it_samples() {
        let words = fragment_module();
        let body = instructions(&words).expect("walk");
        assert!(body.iter().any(|i| i.opcode == op::IMAGE_SAMPLE_IMPLICIT_LOD));
        // OpTypeImage's own word count is 1 header + 8 operands in this build:
        // sampledType, Dim, Depth, Arrayed, MS, Sampled, Format.
        let image = body
            .iter()
            .find(|i| i.opcode == op::TYPE_IMAGE)
            .expect("an image type");
        assert_eq!(image.word_count, 9);
        // uv, colour, and the sampled image the descriptor binds. The third load is
        // what makes the sample operand a value rather than a pointer; dropping it
        // is invisible to the word walk, so its absence is caught here as a count.
        assert_eq!(body.iter().filter(|i| i.opcode == op::LOAD).count(), 3);
    }

    #[test]
    fn the_entry_point_name_is_a_padded_literal() {        // "main" is four bytes plus a null = two words; the check is that the
        // entry instruction counts them, which the walk above already proved by
        // consuming the stream exactly.
        assert_eq!(super::Builder::literal("main").len(), 2);
        assert_eq!(super::Builder::literal("i").len(), 1);
    }
}
