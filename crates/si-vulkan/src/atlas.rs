//! 08 §3.1: the glyph atlas, baked procedurally.
//!
//! **Glyphs, not words.** A bitmap font with a baked *word* list would be a
//! lexicon sitting in the assets — the §3.2 allowlist's spirit applied to a
//! `.png` — so the atlas is 95 cells of shape and nothing else. Line breaking in
//! the renderer is by whitespace and character width, because a hyphenation
//! table, a word-boundary list and a spell checker are each a corpus, and a
//! corpus inside `src/` is R-03/R-10's shape.
//!
//! **Printable ASCII in v1, published as a limit rather than a bug** (§3.1):
//! Unicode shaping drags in `harfbuzz`-class data tables, which [00 §9]'s
//! allowlist law forbids and the audit would have to hash (**OD-42**). Text
//! outside the range renders as [`SUBSTITUTE`] and is stored **verbatim** in the
//! sense record, so nothing is lost, only unprettified.

/// The first codepoint the atlas carries.
pub const FIRST: u8 = 0x20;
/// The last codepoint the atlas carries, inclusive.
pub const LAST: u8 = 0x7e;
/// Cell width in pixels, and the monospace advance of every cell (§3.1).
pub const CELL_W: usize = 5;
/// Cell height in pixels.
pub const CELL_H: usize = 7;

/// One row of a cell, low five bits, leftmost column first.
type Row = u8;

/// The 95 cells in codepoint order, `#` marking an on-pixel. Authored here and
/// baked at open, so the atlas has no build step, no asset file and no hash item
/// ([00 §5]'s artifact discipline, applied to a font).
const SHAPES: [&str; CELLS] = [
    "...../...../...../...../...../...../.....", // space
    "..#../..#../..#../..#../..#../...../..#..", // !
    ".#.#./.#.#./...../...../...../...../.....",  // "
    ".#.#./#####/.#.#./#####/.#.#./...../.....", // #
    "..#../#####/..#.#/####./..#../#####/..#..", // $
    "##..#/.#.#./..#../.#.#./#..##/...../.....", // %
    ".###./#.#../#.#../.###./#...#/#.#.#/..###", // &
    "..#../..#../...../...../...../...../.....", // '
    "...#./..#../.#.../.#.../...../..#../...#.", // (
    ".#.../..#../...#./...#./...../..#../.#...", // )
    "#.#.#/.###./#####/.###./#.#.#/...../.....", // *
    "...../..#../..#../#####/..#../..#../.....", // +
    "...../...../...../...../..#../..#../.#...", // ,
    "...../...../...../#####/...../...../.....", // -
    "...../...../...../...../...../..#../.....", // .
    "....#/....#/...#./..#../.#.../#..../.....", // /
    ".###./#...#/#...#/#.#.#/#..##/#...#/.###.", // 0
    "..#../.##../..#../..#../..#../..#../.###.", // 1
    ".###./#...#/....#/...#./..#../.#.../#####", // 2
    "####./....#/..#../...#./....#/#...#/.###.", // 3
    "...#./..##./.#.#./#..#./#####/...#./...#.", // 4
    "#####/#..../.####/....#/....#/#...#/.###.", // 5
    "..#../.#.../#..../####./#...#/#...#/.###.", // 6
    "#####/....#/...#./..#../.#.../..#../..#..", // 7
    ".###./#...#/#...#/.###./#...#/#...#/.###.", // 8
    ".###./#...#/#...#/.####/....#/...#./..#..", // 9
    "...../..#../...../...../..#../...../.....", // :
    "...../..#../...../...../..#../..#../.#...", // ;
    "...#./..#../.#.../#..../.#.../..#../...#.", // <
    "...../...../#####/...../#####/...../.....", // =
    ".#.../..#../...#./....#/...#./..#../.#...", // >
    ".###./#...#/....#/...#./..#../...../..#..", // ?
    ".###./#...#/#.###/#.#.#/#.###/#..../.###.", // @
    ".###./#...#/#...#/#####/#...#/#...#/#...#", // A
    "####./#...#/####./#...#/#...#/#...#/####.", // B
    ".###./#...#/#..../#..../#..../#...#/.###.", // C
    "####./#...#/#...#/#...#/#...#/#...#/####.", // D
    "#####/#..../#####/#..../#####/#..../#####", // E
    "#####/#..../#####/#..../#####/#..../#....", // F
    ".###./#...#/#..../#.###/#...#/#...#/.###.", // G
    "#...#/#...#/#####/#...#/#...#/#...#/#...#", // H
    "..#../..#../..#../..#../..#../..#../..#..", // I
    "....#/....#/....#/....#/#...#/#...#/.###.", // J
    "#...#/#..#./#.#../##.../#.#../#..#./#...#", // K
    "#..../#..../#..../#..../#..../#..../#####", // L
    "#...#/##.##/#.#.#/#...#/#...#/#...#/#...#", // M
    "#...#/##..#/#.#.#/#..##/#...#/#...#/#...#", // N
    ".###./#...#/#...#/#...#/#...#/#...#/.###.", // O
    "####./#...#/####./#..../#..../#..../#....", // P
    ".###./#...#/#...#/#...#/#.#.#/#..#./..##.", // Q
    "####./#...#/####./#.#../#..#./#...#/#...#", // R
    ".####/#..../#..../.###./....#/....#/####.", // S
    "#####/..#../..#../..#../..#../..#../..#..", // T
    "#...#/#...#/#...#/#...#/#...#/#...#/.###.", // U
    "#...#/#...#/#...#/#...#/#...#/.#.#./..#..", // V
    "#...#/#...#/#...#/#.#.#/#.#.#/##.##/#...#", // W
    "#...#/#...#/.#.#./..#../.#.#./#...#/#...#", // X
    "#...#/#...#/.#.#./..#../..#../..#../..#..", // Y
    "#####/....#/...#./..#../.#.../#..../#####", // Z
    ".###./.#.../.#.../.#.../.#.../.#.../.###.", // [
    "#..../#..../.#.../..#../...#./....#/....#", // backslash
    ".###./...#./...#./...#./...#./...#./.###.", // ]
    "..#../.#.#./#...#/...../...../...../.....", // ^
    "...../...../...../...../...../...../#####", // _
    ".#.../#..../...../...../...../...../.....", // `
    "...../...../.###./....#/.####/#...#/.####", // a
    "#..../#..../####./#...#/#...#/#...#/####.", // b
    "...../...../.###./#..../#..../#...#/.###.", // c
    "....#/....#/.####/#...#/#...#/#...#/.####", // d
    "...../...../.###./#...#/#####/#..../.###.", // e
    "..##./.#..#/.#.../###../.#.../.#.../.#...", // f
    "...../.####/#...#/#...#/####./....#/####.", // g
    "#..../#..../####./#...#/#...#/#...#/#...#", // h
    "..#../...../.##../..#../..#../..#../.###.", // i
    "....#/...../....#/....#/....#/#...#/.###.", // j
    "#..../#..../#..#./#.#../##.../#.#../#..#.", // k
    ".##../..#../..#../..#../..#../..#../.###.", // l
    "...../...../##.#./#.#.#/#.#.#/#...#/#...#", // m
    "...../...../####./#...#/#...#/#...#/#...#", // n
    "...../...../.###./#...#/#...#/#...#/.###.", // o
    "...../...../####./#...#/####./#..../#....", // p
    "...../...../.####/#...#/.####/....#/....#", // q
    "...../...../#.##./##.../#..../#..../#....", // r
    "...../...../.####/#..../.###./....#/####.", // s
    ".#.../.#.../###../.#.../.#.../.#..#/..##.", // t
    "...../...../#...#/#...#/#...#/#...#/.####", // u
    "...../...../#...#/#...#/#...#/.#.#./..#..", // v
    "...../...../#...#/#...#/#.#.#/#.#.#/.#.#.", // w
    "...../...../#...#/.#.#./..#../.#.#./#...#", // x
    "...../...../#...#/#...#/.####/....#/####.", // y
    "...../...../#####/..#../.#.../#..../#####", // z
    "...#./..#../..#../.#.../..#../..#../...#.", // {
    "..#../..#../..#../..#../..#../..#../..#..", // |
    ".#.../..#../..#../...#./..#../..#../.#...", // }
    "...../...../.#..#/#.#.#/#..#./...../.....", // ~
];

/// The cell drawn for anything outside `FIRST..=LAST`: §3.1's published limit,
/// rendered rather than dropped. It is baked as the atlas's last cell, so a
/// substituted character is a real cell and not a silent space.
pub const SUBSTITUTE: [Row; CELL_H] = bits("#####/#...#/#.###/#.##./#..##/#...#/#####");

/// The atlas cell index of [`SUBSTITUTE`], one past the codepoint range.
pub const SUBSTITUTE_INDEX: u16 = (LAST - FIRST + 1) as u16;

const CELLS: usize = (LAST - FIRST + 1) as usize;

/// The baked sheet's width in pixels: one [`CELL_W`] column per cell, and the
/// substitute cell is a column like any other.
pub const fn sheet_width() -> usize {
    (CELLS + 1) * CELL_W
}

/// The atlas's own size in bytes, one byte per pixel — the number the VRAM/PERF
/// panel prints next to the "lanes" split (§7's resident-bytes row), because an
/// atlas that quietly grew is an asset the audit never hashed.
pub const SHEET_BYTES: usize = sheet_width() * CELL_H;

const fn bits(shape: &str) -> [Row; CELL_H] {
    let bytes = shape.as_bytes();
    let mut rows = [0u8; CELL_H];
    let mut row = 0usize;
    let mut col = 0usize;
    let mut index = 0usize;
    while index < bytes.len() {
        let byte = bytes[index];
        match byte {
            b'#' => {
                if row < CELL_H && col < CELL_W {
                    rows[row] |= 1 << (CELL_W - 1 - col);
                }
                col += 1;
            }
            b'.' => col += 1,
            b'/' => {
                row += 1;
                col = 0;
            }
            _ => {}
        }
        index += 1;
    }
    rows
}

/// The cell for one character. `None` says it is outside the published scope.
pub fn glyph(ch: char) -> Option<[Row; CELL_H]> {
    let code = ch as u32;
    if !(FIRST as u32..=LAST as u32).contains(&code) {
        return None;
    }
    let index = (code - FIRST as u32) as usize;
    // `SHAPES` is `CELLS` long by the const above, so an index inside the
    // codepoint range is inside the table.
    Some(bits(SHAPES[index]))
}

/// The baked coverage map: [`sheet_width()`] pixels across, [`CELL_H`] rows down,
/// one byte per pixel in **row-major** order — the order a Vulkan buffer-to-image
/// copy expects, so the upload needs no transpose. [`SUBSTITUTE`] is the last
/// column. `sheet[i] != 0` means coverage at pixel `i`.
pub fn bake() -> Vec<u8> {
    let width = sheet_width();
    let mut sheet = vec![0u8; SHEET_BYTES];
    let cells = SHAPES.iter().map(|shape| bits(shape)).chain([SUBSTITUTE]);
    for (cell, rows) in cells.enumerate() {
        for (row, value) in rows.iter().enumerate() {
            for col in 0..CELL_W {
                let on = (value >> (CELL_W - 1 - col)) & 1;
                sheet[row * width + cell * CELL_W + col] = on * 0xff;
            }
        }
    }
    sheet
}

/// The monospace advance of a run of text, in pixels: whitespace and character
/// width only (§3.1's line-breaking rule, which is also the whole of it).
pub fn width_of(text: &str) -> usize {
    text.chars().count() * CELL_W
}

/// A text cell's identity for the renderer's quad batch: the atlas cell index
/// plus the substitute flag, so a reviewer can tell an unrendered codepoint from
/// a rendered one. §3.1: the bytes stay verbatim in the sense record either way.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub atlas_index: u16,
    pub substituted: bool,
}

/// Map a string to cells. Every character outside ASCII's printable band becomes
/// the substitute cell, which is what makes the limit visible instead of silent.
pub fn cells(text: &str) -> Vec<Cell> {
    text.chars()
        .map(|ch| match glyph(ch) {
            Some(_) => Cell { atlas_index: (ch as u32 - FIRST as u32) as u16, substituted: false },
            None => Cell { atlas_index: SUBSTITUTE_INDEX, substituted: true },
        })
        .collect()
}
