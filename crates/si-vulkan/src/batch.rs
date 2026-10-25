//! 08 §5.3's "one surface, several panels as immediate-mode regions", as a pure
//! function of [`crate::panels::Panels`].
//!
//! Nothing here calls Vulkan. That split is deliberate: the batch is the part a CPU
//! test can hold accountable — the tiling, the line breaking, the display LUT, the
//! fact that a gap is drawn as a visible marker rather than as an absence — while
//! [`crate::surface`] is the part that needs a driver. A reviewer who wants to know
//! what the window *would* have shown gets an answer from this module, not from a
//! screenshot.
//!
//! Coordinates are generated in **pixels from the top-left**, the same origin the
//! fragment stage declares (`OriginUpperLeft` in [`crate::spirv`]), and converted
//! to clip space once, here. Positions come from [`crate::layout::pos`] — identity —
//! and [`View`] below is a *view* over those grid ints: it can turn the picture and
//! can never move a byte, which is §5.2's "the LUT may not be inverted" applied to
//! geometry.

use crate::atlas::{glyph, sheet_width, CELL_H, CELL_W, SUBSTITUTE_INDEX};
use crate::panels::{Panels, Preview, Row};

/// One vertex: clip position, atlas uv, colour. The layout matches [`crate::spirv`]'s
/// vertex inputs — `vec4` at location 0, `vec2` at 1, `vec4` at 2 — and
/// `bytemuck::Pod` is what lets [`crate::surface`] hand the slice straight to the
/// driver instead of re-writing it field by field.
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 4],
    pub uv: [f32; 2],
    pub colour: [f32; 4],
}

/// The vertex stride the pipeline declares.
pub const VERTEX_BYTES: usize = std::mem::size_of::<Vertex>();

/// The attribute offsets the pipeline declares, derived from the field sizes so a
/// reordered `#[repr(C)]` struct fails the stride assert instead of drawing garbage.
pub const POSITION_OFFSET: usize = 0;
pub const UV_OFFSET: usize = POSITION_OFFSET + std::mem::size_of::<[f32; 4]>();
pub const COLOUR_OFFSET: usize = UV_OFFSET + std::mem::size_of::<[f32; 2]>();

/// Quads a frame may hold. The cap exists because the batch buffer is allocated
/// once and persistently mapped ([07 §9]'s W.5 idiom, applied to the viewer's own
/// data): a batch that could grow without bound would be a reallocation per frame.
/// Overflow is **counted, never silent** — §2.2's "drops display frames, never
/// data" is the same rule aimed at the viewer's own budget.
pub const MAX_QUADS: usize = 1 << 15;

/// A cell plus one row of leading, so two rows of text never touch.
pub const LINE_HEIGHT_PX: u32 = (CELL_H as u32) + 1;

/// Chrome colours, fixed here and nowhere else. R-21's costume test applies to a
/// colour too: if a hue varied with a value the viewer had *chosen*, the panel
/// would be classifying, and classification is `si-audit`'s job.
const TEXT: [f32; 4] = [0.88, 0.90, 0.94, 1.0];
const TITLE: [f32; 4] = [1.0, 0.82, 0.35, 1.0];

/// One panel's region of the surface, in pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    pub name: &'static str,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Region {
    /// Text columns this region can hold.
    pub fn columns(&self) -> usize {
        (self.width.saturating_sub(4) / CELL_W as u32).max(1) as usize
    }

    /// Text rows this region can hold, after its title line.
    pub fn lines(&self) -> usize {
        (self.height / LINE_HEIGHT_PX).saturating_sub(1) as usize
    }

    /// The row of pixels a given text line starts on.
    fn line_y(&self, line: usize) -> u32 {
        self.y + (line as u32 + 1) * LINE_HEIGHT_PX
    }
}

/// The five regions, tiled: CHAT down the left, EMOTIONS and PREVIEW stacked on the
/// right, ORGANS under them, VRAM/PERF along the bottom.
///
/// The tiling is fixed rather than user-arrangeable because a draggable panel has a
/// position, and a position the viewer keeps is a piece of state process B owns —
/// §1's first law says it owns none.
pub fn regions(width: u32, height: u32) -> [Region; 5] {
    let width = width.max(4 * CELL_W as u32);
    let height = height.max(3 * LINE_HEIGHT_PX);
    let left = width * 3 / 10;
    let right = width - left;
    let column = right / 2;
    let strip = (height / 6).max(2 * LINE_HEIGHT_PX);
    let body = height - strip;
    let upper = body * 5 / 9;
    [
        Region { name: "CHAT", x: 0, y: 0, width: left, height: body },
        Region { name: "EMOTIONS", x: left, y: 0, width: column, height: upper },
        Region {
            name: "TESSERACT PREVIEW",
            x: left + column,
            y: 0,
            width: right - column,
            height: upper,
        },
        Region {
            name: "ORGANS STATUS",
            x: left,
            y: upper,
            width: right,
            height: body - upper,
        },
        Region { name: "VRAM/PERF", x: 0, y: body, width, height: strip },
    ]
}

/// The view of the preview: two rotations over the grid ints, in whole degrees.
///
/// Presentation only. `turned` moves the picture the way holding a paper model
/// does; it does not re-place a point, because a point's place is
/// [`crate::layout::pos`], and that function takes a handle and a shelf and nothing
/// else (**08 §5.1**). There is no analog control: the arrow keys step
/// [`View::STEP`], and a slider that moved a camera is one refactor away from a
/// slider that moves `a` (**08 §9**'s first row).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    pub yaw: f32,
    pub pitch: f32,
    /// Grid half-extent to region half-width, in q8. A display scale, so the cloud
    /// fits without a per-frame bound scan.
    pub zoom_q8: u32,
}

impl Default for View {
    fn default() -> Self {
        Self { yaw: 0.0, pitch: 0.0, zoom_q8: 256 }
    }
}

impl View {
    /// The step the arrow keys take, in degrees.
    pub const STEP: f32 = 15.0;

    pub fn turned(mut self, dyaw: f32, dpitch: f32) -> Self {
        self.yaw = wrap_degrees(self.yaw + dyaw);
        self.pitch = wrap_degrees(self.pitch + dpitch);
        self
    }
}

fn wrap_degrees(value: f32) -> f32 {
    let mut v = value % 360.0;
    if v > 180.0 {
        v -= 360.0;
    } else if v < -180.0 {
        v += 360.0;
    }
    v
}

/// A marker's shape in the preview. A gap is drawn as a visible outline rather than
/// as an empty pixel, because §5.2's dark-spots row says a gap is a *thing SI knows
/// it does not know* and the preview is where that is legible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Point,
    Gap,
}

impl Shape {
    /// The atlas cell drawn: `o` for a lit point, `?` for a named empty slot. Both
    /// are glyphs the atlas already carries, so the preview invents no font.
    pub fn cell(self) -> u16 {
        let code: u8 = match self {
            Shape::Point => b'o',
            Shape::Gap => b'?',
        };
        u16::from(code - crate::atlas::FIRST)
    }

    /// The marker's side in pixels. A gap is at least one cell wide so it cannot
    /// render as nothing.
    pub fn side(self, base: u32) -> u32 {
        match self {
            Shape::Point => base.max(1),
            Shape::Gap => base.max(CELL_W as u32),
        }
    }
}

/// One frame's geometry: the text and preview quads, plus what did not fit.
#[derive(Clone, Debug)]
pub struct Batch {
    pub vertices: Vec<Vertex>,
    /// Quads that did not fit [`MAX_QUADS`].
    pub dropped: usize,
    /// Rows the regions could not show at all. A row is dropped whole, never cut
    /// mid-word, so a fragment is never displayed as if it were the record.
    pub hidden_rows: usize,
    extent: (u32, u32),
}

impl Default for Batch {
    fn default() -> Self {
        Self { vertices: Vec::new(), dropped: 0, hidden_rows: 0, extent: (0, 0) }
    }
}

impl Batch {
    fn quad(&mut self, x: u32, y: u32, w: u32, h: u32, cell: u16, colour: [f32; 4]) {
        if self.vertices.len() + 6 > MAX_QUADS * 6 {
            self.dropped += 1;
            return;
        }
        let uv = cell_uv(cell);
        let at = |px: u32, py: u32, u: f32, v: f32| Vertex {
            position: clip(px, py, self.extent.0, self.extent.1),
            uv: [u, v],
            colour,
        };
        let (u0, v0, u1, v1) = (uv[0], uv[1], uv[2], uv[3]);
        let a = at(x, y, u0, v0);
        let b = at(x + w, y, u1, v0);
        let c = at(x + w, y + h, u1, v1);
        let d = at(x, y + h, u0, v1);
        self.vertices.extend_from_slice(&[a, b, c, a, c, d]);
    }

    /// Lay `text` out as glyph quads, wrapping at the region's column count. Every
    /// character is drawn, including one outside the published ASCII band, which
    /// draws as the substitute cell (§3.1's limit, rendered rather than dropped).
    fn text(&mut self, x: u32, y_start: u32, columns: usize, text: &str, colour: [f32; 4]) {
        for (row, line) in wrap(text, columns).into_iter().enumerate() {
            let y = y_start + row as u32 * LINE_HEIGHT_PX;
            for (column, ch) in line.chars().take(columns).enumerate() {
                let cell = match glyph(ch) {
                    Some(_) => (ch as u32 - crate::atlas::FIRST as u32) as u16,
                    None => SUBSTITUTE_INDEX,
                };
                self.quad(
                    x + column as u32 * CELL_W as u32,
                    y,
                    CELL_W as u32,
                    CELL_H as u32,
                    cell,
                    colour,
                );
            }
        }
    }

    /// A line as a strip of one-pixel quads: the viewer has one pipeline and one
    /// primitive, which is the smallest thing that can be checked without a shader
    /// per widget. DDA, integer end-points, no interpolation of a *value* — only of
    /// a position (§5.2's "nothing is interpolated into a value the loop did not
    /// compute").
    fn line(&mut self, from: (u32, u32), to: (u32, u32), colour: [f32; 4]) {
        let (mut x, mut y) = from;
        let dx = to.0.abs_diff(x) as i64;
        let dy = to.1.abs_diff(y) as i64;
        let sx = if to.0 > x { 1i64 } else { -1 };
        let sy = if to.1 > y { 1i64 } else { -1 };
        let mut error = dx - dy;
        let limit = self.vertices.len() + (dx.max(dy) as usize) * 6;
        if limit > MAX_QUADS * 6 {
            self.dropped += 1;
            return;
        }
        loop {
            self.quad(x, y, 1, 1, SUBSTITUTE_INDEX, colour);
            if x == to.0 && y == to.1 {
                break;
            }
            let doubled = error * 2;
            if doubled > -dy {
                error -= dy;
                x = (x as i64 + sx).max(0) as u32;
            }
            if doubled < dx {
                error += dx;
                y = (y as i64 + sy).max(0) as u32;
            }
        }
    }

    /// Rows into a region from its first text line down, whole rows only.
    fn rows(&mut self, region: &Region, rows: &[Row]) {
        let columns = region.columns();
        let capacity = region.lines();
        let mut line = 1usize;
        for row in rows {
            let lines = wrap(&row_text(row), columns).len();
            if line + lines > capacity + 1 {
                self.hidden_rows += 1;
                continue;
            }
            for (index, text) in wrap(&row_text(row), columns).into_iter().enumerate() {
                self.text(
                    region.x + 2,
                    region.line_y(line - 1 + index),
                    columns,
                    &text,
                    TEXT,
                );
            }
            line += lines;
        }
    }
}

/// The frame: every panel into its region, plus the preview's geometry.
pub fn frame(panels: &Panels, extent: (u32, u32), view: &View) -> Batch {
    let regions = regions(extent.0, extent.1);
    let mut batch = Batch { vertices: Vec::new(), extent, dropped: 0, hidden_rows: 0 };
    for region in &regions {
        batch.text(region.x + 2, region.y, region.columns(), region.name, TITLE);
    }
    batch.rows(&regions[0], &panels.chat.rows);
    batch.rows(&regions[1], &panels.emotions.rows);
    batch.rows(&regions[3], &panels.organs.rows);
    batch.rows(&regions[4], &panels.perf.rows);
    preview(&mut batch, &regions[2], &panels.preview, view);
    batch
}

/// The preview: lit points whose side and intensity come from `h_q12` through the
/// display LUT, traversed edges the same way, and the stop-reason rows below.
fn preview(batch: &mut Batch, region: &Region, panel: &Preview, view: &View) {
    let centre = (region.x + region.width / 2, region.y + region.height / 2);
    for node in &panel.nodes {
        let (x, y, _) = project(node.position, view);
        let shape = if node.h_q12 == 0 { Shape::Gap } else { Shape::Point };
        let side = shape.side(2 + (node.h_q12.abs() >> 12).min(6) as u32);
        let intensity = (node.h_q12.abs().min(4096) as f32 / 4096.0).clamp(0.15, 1.0);
        let (px, py) = place(centre, x, y);
        batch.quad(px, py, side, side, shape.cell(), [intensity, intensity, intensity, 1.0]);
    }
    for segment in &panel.edges {
        let from = project(crate::layout::pos(segment.from, None), view);
        let to = project(crate::layout::pos(segment.to, None), view);
        // §5.2: the display LUT's floor is not the schema's, so a dim edge still
        // draws, and OWNED vs BASE differ by hue *and* by the number in the row
        // below — never by hue alone.
        let intensity = segment
            .intensity()
            .map_or(0.15, |value| (value as f32 / 255.0).max(0.15));
        let colour = if segment.owned {
            [intensity, intensity * 0.55, intensity * 0.55, 1.0]
        } else {
            [intensity * 0.55, intensity * 0.55, intensity, 1.0]
        };
        batch.line(place(centre, from.0, from.1), place(centre, to.0, to.1), colour);
    }
    batch.rows(region, &panel.rows);
}

/// A signed grid offset into a pixel position, clamped to stay on the surface.
fn place(centre: (u32, u32), x: i32, y: i32) -> (u32, u32) {
    (
        (centre.0 as i64 + x as i64).clamp(0, u32::MAX as i64) as u32,
        (centre.1 as i64 + y as i64).clamp(0, u32::MAX as i64) as u32,
    )
}

/// Grid ints to pixels-from-centre under [`View`]: two rotations, one scale, one
/// divide. Nothing is learned and nothing is relaxed, and no distance in the picture
/// is a distance in meaning (**08 §5.1**).
pub fn project(position: [i32; 3], view: &View) -> (i32, i32, i32) {
    let [x, y, z] = position;
    let (cy, sy) = (view.yaw.to_radians().cos(), view.yaw.to_radians().sin());
    let (cp, sp) = (view.pitch.to_radians().cos(), view.pitch.to_radians().sin());
    let rx = x as f32 * cy + z as f32 * sy;
    let rz = -x as f32 * sy + z as f32 * cy;
    let ry = y as f32 * cp - rz * sp;
    let depth = (y as f32 * sp + rz * cp) as i32;
    let scale = view.zoom_q8 as f32 / 256.0 / (crate::layout::EXTENT as f32 / 2.0);
    ((rx * scale * 512.0) as i32, (ry * scale * 512.0) as i32, depth)
}

/// Pixel to clip space, with y pointing down the surface.
pub fn clip(px: u32, py: u32, width: u32, height: u32) -> [f32; 4] {
    let x = px as f32 / width.max(1) as f32 * 2.0 - 1.0;
    let y = 1.0 - py as f32 / height.max(1) as f32 * 2.0;
    [x, y, 0.0, 1.0]
}

/// A cell's uv rectangle: the sheet is one strip of cells, so the index selects the
/// column and the cell is the sheet's whole height.
pub fn cell_uv(atlas_index: u16) -> [f32; 4] {
    let columns = (sheet_width() / CELL_W) as f32;
    [
        atlas_index as f32 / columns,
        0.0,
        (atlas_index as f32 + 1.0) / columns,
        1.0,
    ]
}

/// The coverage map [`crate::atlas::bake`] produces, widened to RGBA8 for the
/// upload: white ink, coverage in alpha, so `atlas * colour` in the fragment stage
/// tints the glyph with the row's colour and lets the clear colour show through the
/// background. The sheet is already row-major, so this is a widening and no
/// transpose.
pub fn rgba8(sheet: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(sheet.len() * 4);
    for coverage in sheet {
        out.extend_from_slice(&[0xff, 0xff, 0xff, *coverage]);
    }
    out
}

/// Break a line at whitespace only — §3.1's whole line-breaking rule, chosen
/// because a hyphenation or word-boundary table is a corpus inside `src/`.
pub fn wrap(text: &str, columns: usize) -> Vec<String> {
    let columns = columns.max(1);
    let mut out: Vec<String> = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.chars().count() + 1 + word.chars().count() <= columns {
            current.push(' ');
            current.push_str(word);
        } else {
            out.push(std::mem::take(&mut current));
            current.push_str(word);
        }
        // A word longer than the column count is broken rather than dropped: the
        // row is the record, and half a visible word beats a missing one.
        while current.chars().count() > columns {
            let head: String = current.chars().take(columns).collect();
            out.push(head);
            current = current.chars().skip(columns).collect();
        }
    }
    if !current.is_empty() || out.is_empty() {
        out.push(current);
    }
    out
}

/// A row exactly as the panel prints it.
pub fn row_text(row: &Row) -> String {
    format!("{}: {}", row.key, row.value)
}

/// The truncation note, so an over-budget frame says so rather than showing a clean
/// picture that is a lie (§2.2).
pub fn drop_note(batch: &Batch) -> Option<Row> {
    if batch.dropped == 0 && batch.hidden_rows == 0 {
        return None;
    }
    Some(Row::new(
        "display",
        format!(
            "{} quads dropped, {} rows hidden — data is never dropped (08 §2.2)",
            batch.dropped, batch.hidden_rows
        ),
    ))
}
