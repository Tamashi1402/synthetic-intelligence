//! 08 §5.3 + [02 §10]: the window half — `winit` for the surface, `ash` for the
//! swapchain, and the two residency paths the spec names.
//!
//! This module is the only place in the crate that talks to a driver, and it is
//! compiled only with the `vulkan` feature. Everything it *decides* comes from
//! [`crate::batch`] (geometry) and [`crate::panels`] (rows), both of which are pure
//! and CPU-tested, so a reviewer who cannot see the screen can still read what the
//! screen would have held.
//!
//! ## The one writable file
//!
//! [`crate::WRITABLE_FILE`] is opened exactly once, here, in
//! [`Viewer::submit`], in append mode. No other path in this module is opened for
//! writing, and the module never creates a directory, never renames, never deletes
//! — which is what makes [05 §20]'s `check_gui_observer_only` a grep rather than a
//! review.
//!
//! ## Every other file is handed over
//!
//! [00 §8] says the only files this crate may open are process B's own lane tails,
//! while [08 §2] and §8's table have the panels reading `SIst`, `mindlog`, the sleep
//! log and the fingerprint. The resolution is [`Handover`]: the launcher (which owns
//! the schema crates) decodes those artifacts and passes them in as values, and a
//! panel **forwards** them. A pane that opened the snapshot itself would need
//! `si-store`, and [02 §10]'s crate graph has no such edge. The same rule is why
//! [`genome_residency`] takes **bytes** rather than a path: `SIgen` is `si-store`'s
//! file to map (**OD-148**).
//!
//! ## Device loss
//!
//! [`Outcome::Degraded`] is the specified behaviour of [02 §10]: "device loss ⇒ the
//! viewer degrades to headless; **the loop never notices**". No failure in this
//! module is allowed to reach process A, because the only channel to A is the inbox
//! the owner types into.

use crate::atlas::{bake, sheet_width, CELL_H};
use crate::batch::{
    rgba8, Batch, COLOUR_OFFSET, POSITION_OFFSET, UV_OFFSET, VERTEX_BYTES, View, MAX_QUADS,
};
use crate::frames::{frame_bytes, inbox_header, sense_frame, Lane, Outbound};
use crate::panels::{build, Artifacts, Row};
use anyhow::{bail, Result};
use ash::khr;
use ash::vk;
use std::ffi::{c_char, CString};
use anyhow::Context as _;
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

/// Frame cap the DoD names (`--exit-after-frames 120`), as a default so a run that
/// forgot the flag still ends.
pub const DEFAULT_EXIT_AFTER_FRAMES: u64 = 120;

/// What the launcher hands over: the lane bytes it read, and the decoded artifacts.
///
/// A trait rather than a struct of `Vec<u8>` fields because the live viewer must
/// re-read the tail every frame while a replay reads it once, and §2.2's law says
/// those two are **the same code** — only the source differs.
pub trait Handover {
    /// The start of a composition, so a source that caches can drop the previous
    /// frame's answer. A replay re-reads nothing and leaves this empty.
    fn begin_frame(&mut self) {}

    /// The bytes of the output lane's tail, as much as has been written so far.
    fn lane_tail(&mut self) -> Result<Vec<u8>>;

    /// The non-lane artifacts, already decoded by the launcher.
    fn artifacts(&mut self) -> Result<Artifacts>;

    /// `SI_PROFILE`'s state, forwarded so §7's instrumentation rule is honoured: an
    /// unmeasured cell prints the dash sentence, never a `0`.
    fn profile(&self) -> bool {
        false
    }

    /// The monologue lane's tail, for O10's pane (blank until **OD-38** rules).
    fn monologue_tail(&mut self) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }
}

/// Where the inbox lives, and whose run it belongs to.
#[derive(Clone, Debug)]
pub struct Session {
    /// Process A's run directory: the one place the inbox is allowed to be written.
    pub run_dir: PathBuf,
    /// The run's id, forwarded into the inbox header so a reader can refuse a lane
    /// that belongs to a different run by name.
    pub run_id: String,
}

impl Session {
    pub fn inbox(&self) -> PathBuf {
        self.run_dir.join(crate::WRITABLE_FILE)
    }
}

/// How the viewer was asked to end.
#[derive(Clone, Debug)]
pub struct Options {
    pub session: Session,
    pub extent: (u32, u32),
    pub exit_after_frames: u64,
}

impl Options {
    pub fn new(session: Session) -> Self {
        Self { session, extent: (1280, 800), exit_after_frames: DEFAULT_EXIT_AFTER_FRAMES }
    }
}

/// What a viewer run produced. Both arms are exit-0 outcomes; the difference is
/// whether an image was ever presented, and that difference is **reported**, never
/// smoothed over.
#[derive(Clone, Debug)]
pub enum Outcome {
    /// A window came up and this many frames were presented on it.
    ///
    /// The last frame's batch is reported alongside: a log that says only "120
    /// frames" cannot tell a full panel from an empty one, and §7's rule is that the
    /// numbers the run produced are on the screen, not implied.
    Windowed { frames: u64, device: String, vertices: usize, dropped: usize, hidden: usize },
    /// No window was ever presented, and why. §10's degradation, named.
    Degraded { reason: String, frames: u64 },
    /// The owner asked for it, or the event loop ended for another reason, before
    /// any frame could be presented.
    Closed { reason: String },
}

impl Outcome {
    /// The line `viewer.log` gets. `Windowed` names the device because §7's "device
    /// reality, reported never assumed" row is the point of the panel.
    pub fn summary(&self) -> String {
        match self {
            Outcome::Windowed { frames, device, vertices, dropped, hidden } => format!(
                "{frames} frames presented on {device}; last frame {} quads drawn, \
                 {dropped} quads over the batch budget, {hidden} rows not shown",
                vertices / 6
            ),
            Outcome::Degraded { reason, frames } => format!(
                "headless: {reason} — {frames} panel frames composed without a surface \
                 (02 §10's degradation; process A was never notified)"
            ),
            Outcome::Closed { reason } => format!("closed before a frame: {reason}"),
        }
    }

    pub fn frames(&self) -> u64 {
        match self {
            Outcome::Windowed { frames, .. } | Outcome::Degraded { frames, .. } => *frames,
            Outcome::Closed { .. } => 0,
        }
    }
}

/// The whole Vulkan context, once it exists.
///
/// Held behind an `Option` in [`Viewer`] because the context can only be built after
/// `resumed`, and its failure is the degradation path rather than a panic.
///
/// `pub` because [`genome_residency`] and [`Residency::flush`] take it: the launcher
/// owns the pinned `SIgen` mapping ([00 §8], **OD-148**) and hands this crate bytes,
/// so the residency call is made from outside — and an out-lifetime it cannot name
/// would be a leak the compiler is right to complain about.
pub struct Context {
    /// The loaded loader. **This field is never read and must never be removed**:
    /// `ash::Entry::load()` owns the only reference to `vulkan-1.dll`, and ash's
    /// `Instance`/`Device`/`khr::*` handles hold raw function pointers into it, not a
    /// borrow of it. Dropping the entry unmaps the library under every later call —
    /// which is a jump into freed address space, not a `VkResult` — so the entry has
    /// to outlive the context that makes the calls.
    #[allow(dead_code)]
    entry: ash::Entry,
    instance: ash::Instance,
    surface: vk::SurfaceKHR,
    surface_ext: khr::surface::Instance,
    swapchain_ext: khr::swapchain::Device,
    device: ash::Device,
    physical: vk::PhysicalDevice,
    graphics_family: u32,
    present_family: u32,
    queue: vk::Queue,
    device_name: String,
    vram_bytes: vk::DeviceSize,
    render_pass: vk::RenderPass,
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    descriptor_layout: vk::DescriptorSetLayout,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set: vk::DescriptorSet,
    sampler: vk::Sampler,
    atlas_image: vk::Image,
    atlas_memory: vk::DeviceMemory,
    atlas_view: vk::ImageView,
    vertex_buffer: vk::Buffer,
    vertex_memory: vk::DeviceMemory,
    vertex_pointer: *mut std::ffi::c_void,
    command_pool: vk::CommandPool,
    /// One command buffer per swapchain image, recorded fresh each frame and never
    /// reallocated: §7's resident-bytes row counts one buffer set for the session,
    /// and a pool created per frame is a per-frame allocation the spec does not have.
    commands: Vec<vk::CommandBuffer>,
    /// The frame slot, cycled so at most one frame per image is in flight.
    slot: usize,
    swapchain: vk::SwapchainKHR,
    images: Vec<vk::Image>,
    views: Vec<vk::ImageView>,
    framebuffers: Vec<vk::Framebuffer>,
    draw_semaphores: Vec<vk::Semaphore>,
    present_semaphores: Vec<vk::Semaphore>,
    in_flight: Vec<vk::Fence>,
    extent: (u32, u32),
    /// §7's sync row: the host waits this context performed **after** its own
    /// atlas upload. Frame pacing rides a fence — that is the viewer's own
    /// submission bookkeeping, [02 §10]'s "the viewer's own submissions are its
    /// business". A `device_wait_idle` is the thing §5.3 warns about, a GUI adding
    /// a host sync, so *that* is what this counts, and it must read `0`.
    host_waits: u64,
}

/// The window, the panels and the owner's half-typed line.
pub struct Viewer<H: Handover> {
    handover: H,
    options: Options,
    context: Option<Context>,
    window: Option<Window>,
    typed: String,
    view: View,
    frames: u64,
    /// The last frame that reached a submit, kept so `viewer.log` can say what was
    /// on the screen: quad count, and the two truncation counters.
    last_vertices: usize,
    last_dropped: usize,
    last_hidden: usize,
    degradation: Option<String>,
    drawn_through: u64,
    /// The previous frame's truncation note, printed as this frame's first PERF row.
    /// A batch cannot append to itself, and §2.2 wants an over-budget frame to say so
    /// rather than to show a clean picture that is a lie.
    pending_note: Option<Row>,
    /// §7's sync counter: the number of host-side waits the viewer itself performed.
    /// It must read `0` for the v1 CPU walker, and the panel is where a GUI-induced
    /// sync would become visible if one were ever added.
    pub sync_count: u64,
}

impl<H: Handover> Viewer<H> {
    pub fn new(handover: H, options: Options) -> Self {
        Self {
            handover,
            options,
            context: None,
            window: None,
            typed: String::new(),
            view: View::default(),
            frames: 0,
            last_vertices: 0,
            last_dropped: 0,
            last_hidden: 0,
            degradation: None,
            drawn_through: 0,
            pending_note: None,
            sync_count: 0,
        }
    }

    /// Run to the frame cap. This is what `si-cli view` calls.
    ///
    /// A window needs an OS event loop, and the loop needs the thread; if winit
    /// itself refuses (`EventLoopError` — no display server, a headless service, a
    /// locked session), the run is **degraded rather than failed**, and the reason is
    /// winit's own words. That is the only path by which a machine with no desktop
    /// can satisfy [11 §B11]'s DoD, and it is the spec's own §10 behaviour, not a
    /// substitute for the window.
    pub fn run(mut self) -> Result<Outcome> {
        let event_loop = match EventLoop::new() {
            Ok(loop_) => loop_,
            Err(error) => {
                return Ok(Outcome::Degraded {
                    reason: format!("no window system: {error}"),
                    frames: 0,
                })
            }
        };
        event_loop.set_control_flow(ControlFlow::Poll);
        if let Err(error) = event_loop.run_app(&mut self) {
            return Ok(Outcome::Degraded {
                reason: format!("event loop refused: {error}"),
                frames: self.frames,
            });
        }
        Ok(match self.degradation.take() {
            Some(reason) => Outcome::Degraded { reason, frames: self.frames },
            None if self.frames == 0 => {
                Outcome::Closed { reason: "the event loop ended with no frame".to_string() }
            }
            None => Outcome::Windowed {
                frames: self.frames,
                device: self
                    .context
                    .as_ref()
                    .map(|context| context.device_name.clone())
                    .unwrap_or_else(|| "unknown device".to_string()),
                vertices: self.last_vertices,
                dropped: self.last_dropped,
                hidden: self.last_hidden,
            },
        })
    }

    /// Compose the panels from the handover — the code every path shares.
    fn panels(&mut self) -> Result<(Lane, Artifacts, bool)> {
        self.handover.begin_frame();
        let bytes = self.handover.lane_tail()?;
        let lane = Lane::decode(&bytes).context("the output lane refused to decode")?;
        let artifacts = self.handover.artifacts()?;
        Ok((lane, artifacts, self.handover.profile()))
    }

    /// The owner's line, minted as the owner's own frame and appended to the inbox.
    ///
    /// **The crate's one write site.** `source` is [`crate::SOURCE_OWNER`] because
    /// this process is the owner speaking ([00 §7]); a `1` here would be the viewer
    /// forging a supporter line, which §9 lists as a refused widget.
    fn submit(&mut self, line: String) -> Result<()> {
        if line.trim().is_empty() {
            return Ok(());
        }
        let step = self.frames;
        let event = sense_frame(
            step,
            i32::try_from(step).unwrap_or(i32::MAX),
            &Outbound {
                text: line,
                speaker_identity: si_core::eventlog::IDENTITY_UNTAGGED,
                conversation: 0,
            },
        )?;
        let header = inbox_header(&self.options.session.run_id);
        let path = self.options.session.run_dir.join(crate::WRITABLE_FILE);
        let mut file = std::fs::OpenOptions::new().read(true).write(true).create(true).truncate(false).open(&path).with_context(|| format!("{} could not be opened", path.display()))?;
        // A fresh inbox is a `SIro` container with a header, so a reader refuses a
        // mismatched run **by name**; an existing one is appended, never rewritten.
        if file.seek(SeekFrom::End(0))? == 0 {
            let container = si_core::container::Container::new(
                si_core::container::Format::StreamRecord,
                header,
                Vec::new(),
            )
            .encode()?;
            file.write_all(&container)?;
        }
        file.write_all(&frame_bytes(&event))?;
        Ok(())
    }
}

impl<H: Handover> ApplicationHandler for Viewer<H> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attributes = WindowAttributes::default()
            .with_title("SI viewer")
            .with_inner_size(PhysicalSize::new(self.options.extent.0, self.options.extent.1));
        let window = match event_loop.create_window(attributes) {
            Ok(window) => window,
            Err(error) => {
                self.degradation = Some(format!("no window: {error}"));
                event_loop.exit();
                return;
            }
        };
        let extent = (
            window.inner_size().width.max(1),
            window.inner_size().height.max(1),
        );
        self.options.extent = extent;
        match unsafe { Context::new(&window, extent) } {
            Ok(context) => self.context = Some(context),
            Err(error) => {
                // [02 §10]: the viewer degrades; the loop never notices.
                self.degradation = Some(error.to_string());
                event_loop.exit();
                return;
            }
        }
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    self.options.extent = (size.width, size.height);
                    if let Some(context) = self.context.as_mut() {
                        if let Err(error) = context.recreate(size.width, size.height) {
                            self.degradation =
                                Some(format!("the swapchain refused a resize: {error}"));
                            event_loop.exit();
                            return;
                        }
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state != ElementState::Pressed {
                    return;
                }
                match event.logical_key {
                    // `NumpadEnter` is a `KeyCode`, not a `NamedKey`: winit maps the
                    // keypad's enter to the same logical key as the main one, so the
                    // owner's submit is this arm and nothing else.
                    Key::Named(NamedKey::Enter) => {
                        let line = std::mem::take(&mut self.typed);
                        if let Err(error) = self.submit(line) {
                            self.degradation = Some(format!("the inbox refused a line: {error}"));
                            event_loop.exit();
                            return;
                        }
                    }
                    Key::Named(NamedKey::Backspace) => {
                        self.typed.pop();
                    }
                    Key::Named(NamedKey::ArrowLeft) => {
                        self.view = self.view.turned(-View::STEP, 0.0)
                    }
                    Key::Named(NamedKey::ArrowRight) => {
                        self.view = self.view.turned(View::STEP, 0.0)
                    }
                    Key::Named(NamedKey::ArrowUp) => {
                        self.view = self.view.turned(0.0, -View::STEP)
                    }
                    Key::Named(NamedKey::ArrowDown) => {
                        self.view = self.view.turned(0.0, View::STEP)
                    }
                    Key::Character(text) => {
                        // §9's refusals apply to the keyboard too: the viewer types
                        // the owner's line and nothing else, so `text` is the only
                        // input that reaches a byte.
                        self.typed.push_str(&text);
                    }
                    _ => {}
                }
            }
            WindowEvent::RedrawRequested => {
                let batch = match self.compose() {
                    Ok(batch) => batch,
                    Err(error) => {
                        self.degradation = Some(format!("the lane could not be read: {error}"));
                        event_loop.exit();
                        return;
                    }
                };
                self.frames += 1;
                if let Some(context) = self.context.as_mut() {
                    if let Err(error) = unsafe { context.draw(&batch) } {
                        // Device loss arrives as an acquire, submit or present
                        // refusal. It ends this viewer's frames, not process A's
                        // loop — there is no channel from here to there.
                        self.degradation = Some(format!("the surface refused a frame: {error}"));
                        event_loop.exit();
                        return;
                    }
                    self.sync_count = context.host_waits;
                    self.last_vertices = batch.vertices.len();
                    self.last_dropped = batch.dropped;
                    self.last_hidden = batch.hidden_rows;
                }
                if self.frames >= self.options.exit_after_frames {
                    event_loop.exit();
                    return;
                }
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

impl<H: Handover> Viewer<H> {
    /// The panels for this frame, plus the batch the renderer will draw.
    ///
    /// The composition happens **whether or not there is a context**: the lane is
    /// read and the rows built either way, because §2.2's replay and a degraded run
    /// must show the same thing, and the frame cap counts composed frames, not
    /// presented ones.
    fn compose(&mut self) -> Result<Batch> {
        let (lane, artifacts, profile) = self.panels()?;
        let mut panels = build(&lane, &artifacts, profile);
        self.drawn_through = lane.frames.last().map(|frame| frame.seq).unwrap_or(0);
        if let Some(context) = &self.context {
            for row in context.reality(&self.view, self.drawn_through) {
                panels.perf.rows.push(row);
            }
        }
        if let Some(note) = self.pending_note.take() {
            panels.perf.rows.push(note);
        }
        let batch = crate::batch::frame(&panels, self.options.extent, &self.view);
        // The note is **this frame's measurement and next frame's row**: a batch
        // cannot append to itself, and §2.2 wants the truncation visible rather than
        // silent.
        self.pending_note = crate::batch::drop_note(&batch);
        Ok(batch)
    }
}

impl Context {
    /// Build the whole chain: instance → surface → device → swapchain → pipeline →
    /// atlas → buffers.
    ///
    /// # Safety
    ///
    /// Every call here is a raw Vulkan entry point, and the window must outlive the
    /// surface created from its handle.
    unsafe fn new(window: &Window, extent: (u32, u32)) -> Result<Self> {
        let entry = ash::Entry::load().context("no Vulkan loader on this machine")?;
        let names = surface_extensions(window)?;
        let app_name = CString::new("si-viewer").expect("a name without a nul byte");
        let engine = CString::new("si-vulkan").expect("a name without a nul byte");
        let application_info = vk::ApplicationInfo::default()
            .application_name(&app_name)
            .engine_name(&engine)
            .api_version(vk::make_api_version(0, 1, 1, 0));
        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&application_info)
            .enabled_extension_names(&names);
        let instance = entry
            .create_instance(&create_info, None)
            .context("the Vulkan loader refused an instance")?;

        let surface_ext = khr::surface::Instance::new(&entry, &instance);
        let win32_ext = khr::win32_surface::Instance::new(&entry, &instance);
        let (hwnd, hinstance) = win32_handles(window)?;
        let surface = win32_ext
            .create_win32_surface(
                &vk::Win32SurfaceCreateInfoKHR::default()
                    .hinstance(hinstance)
                    .hwnd(hwnd),
                None,
            )
            .context("the driver refused the window's surface")?;

        let physical = pick_device(&instance, &surface_ext, surface)?;
        let properties = instance.get_physical_device_properties(physical);
        let device_name = name_of(properties.device_name.as_slice());
        let memory_properties = instance.get_physical_device_memory_properties(physical);
        let vram_bytes = memory_properties
            .memory_heaps
            .iter()
            .take(memory_properties.memory_heap_count as usize)
            .map(|heap| heap.size)
            .max()
            .unwrap_or(0);

        let families = instance.get_physical_device_queue_family_properties(physical);
        let mut graphics_family = None;
        let mut present_family = None;
        for (index, family) in families.iter().enumerate() {
            if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) && graphics_family.is_none() {
                graphics_family = Some(index as u32);
            }
            if surface_ext
                .get_physical_device_surface_support(physical, index as u32, surface)
                .unwrap_or(false)
            {
                present_family = Some(index as u32);
            }
        }
        let graphics_family = graphics_family.context("no graphics queue on this device")?;
        let present_family = present_family.context("no present queue for this surface")?;

        let device_extensions = vec![khr::swapchain::NAME.as_ptr()];
        let priority = [1.0f32];
        let mut unique_families = vec![graphics_family];
        if present_family != graphics_family {
            unique_families.push(present_family);
        }
        let queue_infos: Vec<vk::DeviceQueueCreateInfo> = unique_families
            .iter()
            .map(|family| {
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(*family)
                    .queue_priorities(&priority)
            })
            .collect();
        let device_create = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_infos)
            .enabled_extension_names(&device_extensions);
        let device = instance
            .create_device(physical, &device_create, None)
            .context("the driver refused a device")?;
        let swapchain_ext = khr::swapchain::Device::new(&instance, &device);
        let queue = device.get_device_queue(graphics_family, 0);

        let render_pass = build_render_pass(&device)?;
        let (descriptor_layout, descriptor_pool, sampler, atlas_image, atlas_memory, atlas_view, descriptor_set) =
            build_atlas(&device, &instance, physical, queue, graphics_family)?;
        let (vertex_buffer, vertex_memory, vertex_pointer) = resident_batch(&device, &instance, physical)?;
        let pipeline_layout = device
            .create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::default().set_layouts(std::slice::from_ref(&descriptor_layout)),
                None,
            )
            .context("no pipeline layout")?;
        let pipeline = build_pipeline(&device, pipeline_layout, render_pass)?;

        let (swapchain, images, views, framebuffers, actual_extent) = swapchain::create(
            &device,
            &swapchain_ext,
            &surface_ext,
            physical,
            surface,
            render_pass,
            extent,
            graphics_family,
            present_family,
        )?;
        let command_pool = device
            .create_command_pool(
                &vk::CommandPoolCreateInfo::default()
                    .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                    .queue_family_index(graphics_family),
                None,
            )
            .context("no command pool")?;
        let count = images.len();
        let commands = device
            .allocate_command_buffers(
                &vk::CommandBufferAllocateInfo::default()
                    .command_pool(command_pool)
                    .level(vk::CommandBufferLevel::PRIMARY)
                    .command_buffer_count(count as u32),
            )
            .context("no command buffers")?;
        let draw_semaphores = semaphores(&device, count)?;
        let present_semaphores = semaphores(&device, count)?;
        let in_flight = fences(&device, count)?;

        Ok(Self {
            entry,
            instance,
            surface,
            surface_ext,
            swapchain_ext,
            device,
            physical,
            graphics_family,
            present_family,
            queue,
            device_name,
            vram_bytes,
            render_pass,
            pipeline,
            pipeline_layout,
            descriptor_layout,
            descriptor_pool,
            descriptor_set,
            sampler,
            atlas_image,
            atlas_memory,
            atlas_view,
            vertex_buffer,
            vertex_memory,
            vertex_pointer,
            command_pool,
            commands,
            slot: 0,
            swapchain,
            images,
            views,
            framebuffers,
            draw_semaphores,
            present_semaphores,
            in_flight,
            extent: actual_extent,
            host_waits: 0,
        })
    }

    /// §7's rows: the residency of the atlas and of the batch, and the device's own
    /// budget. **Reported, never assumed** — the spec's 1.6 GB figure is arithmetic
    /// on a schema constant and this is the machine it met.
    fn reality(&self, view: &View, drawn_through: u64) -> Vec<Row> {
        let atlas = sheet_width() * CELL_H * std::mem::size_of::<u8>() * 4;
        vec![
            Row::new("device", self.device_name.clone()),
            Row::new("device memory budget", self.vram_bytes.to_string()),
            Row::new("atlas resident bytes", atlas.to_string()),
            Row::new("batch resident bytes", (MAX_QUADS * VERTEX_BYTES).to_string()),
            Row::new("swapchain images", self.images.len().to_string()),
            Row::new("sync count", self.host_waits.to_string()),
            Row::new("drawn through seq", drawn_through.to_string()),
            // The view is the viewer's own chrome, printed so a screenshot is
            // reproducible from the panel's text (§2.2's replay claim).
            Row::new("preview view", format!("{view:?}")),
        ]
    }
}

/// The instance extensions: `VK_KHR_surface` plus the platform's surface extension.
///
/// The names leave as raw `*const c_char` because that is what
/// `InstanceCreateInfo::enabled_extension_names` holds — a `Vec<&CStr>` would be a
/// temporary the driver reads after the borrow ends.
#[cfg(target_os = "windows")]
fn surface_extensions(_window: &Window) -> Result<Vec<*const c_char>> {
    Ok(vec![khr::surface::NAME.as_ptr(), khr::win32_surface::NAME.as_ptr()])
}

/// The window's HWND and HINSTANCE, read through `raw-window-handle` rather than
/// guessed at.
#[cfg(target_os = "windows")]
fn win32_handles(window: &Window) -> Result<(isize, isize)> {
    use raw_window_handle::HasWindowHandle;
    let handle = window
        .window_handle()
        .context("the window has no handle to borrow")?;
    match handle.as_raw() {
        raw_window_handle::RawWindowHandle::Win32(win32) => {
            Ok((win32.hwnd.get(), win32.hinstance.map_or(0, |value| value.get())))
        }
        other => bail!("the viewer's surface is Win32 and this handle is {other:?}"),
    }
}

#[cfg(not(target_os = "windows"))]
fn surface_extensions(_window: &Window) -> Result<Vec<*const c_char>> {
    bail!(
        "08 §5.3 chose `winit` + `ash` and this build wires the Win32 surface only; \
         another platform is **OD-149**, not a silent fallback"
    )
}

#[cfg(not(target_os = "windows"))]
fn win32_handles(_window: &Window) -> Result<(isize, isize)> {
    bail!("no Win32 surface on this platform (OD-149)")
}

/// The first device that can present to this surface, preferring a discrete one.
unsafe fn pick_device(
    instance: &ash::Instance,
    surface_ext: &khr::surface::Instance,
    surface: vk::SurfaceKHR,
) -> Result<vk::PhysicalDevice> {
    let candidates = instance
        .enumerate_physical_devices()
        .context("the loader enumerated no device")?;
    if candidates.is_empty() {
        bail!("no Vulkan device exists on this machine");
    }
    for candidate in &candidates {
        let properties = instance.get_physical_device_properties(*candidate);
        if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU
            && surface_ext
                .get_physical_device_surface_support(*candidate, 0, surface)
                .unwrap_or(false)
        {
            return Ok(*candidate);
        }
    }
    Ok(candidates[0])
}

fn name_of(raw: &[c_char]) -> String {
    let bytes: Vec<u8> = raw
        .iter()
        .map(|value| *value as u8)
        .take_while(|byte| *byte != 0)
        .collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

/// One color attachment, cleared, stored, and transitioned to present. The load op
/// is `CLEAR` because a panel that showed last frame's stale row would be a second
/// history the loop does not have (§2.2: the lane is the scrollback, not the
/// framebuffer).
unsafe fn build_render_pass(device: &ash::Device) -> Result<vk::RenderPass> {
    let attachment = vk::AttachmentDescription::default()
        .format(vk::Format::B8G8R8A8_UNORM)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);
    let reference = vk::AttachmentReference::default().attachment(0).layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
    let subpass = vk::SubpassDescription::default()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(std::slice::from_ref(&reference));
    let dependency = vk::SubpassDependency::default()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);
    device
        .create_render_pass(
            &vk::RenderPassCreateInfo::default()
                .attachments(std::slice::from_ref(&attachment))
                .subpasses(std::slice::from_ref(&subpass))
                .dependencies(std::slice::from_ref(&dependency)),
            None,
        )
        .context("no render pass")
}

/// The glyph sheet as a device-local image with a combined image sampler, uploaded
/// from [`crate::atlas::bake`] through **one** staging buffer, and the descriptor
/// set the fragment stage reads.
unsafe fn build_atlas(
    device: &ash::Device,
    instance: &ash::Instance,
    physical: vk::PhysicalDevice,
    queue: vk::Queue,
    family: u32,
) -> Result<(
    vk::DescriptorSetLayout,
    vk::DescriptorPool,
    vk::Sampler,
    vk::Image,
    vk::DeviceMemory,
    vk::ImageView,
    vk::DescriptorSet,
)> {
    let sheet = rgba8(&bake());
    let width = sheet_width() as u32;
    let height = CELL_H as u32;
    let layout = device
        .create_descriptor_set_layout(
            &vk::DescriptorSetLayoutCreateInfo::default().bindings(std::slice::from_ref(
                &vk::DescriptorSetLayoutBinding::default()
                    .binding(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            )),
            None,
        )
        .context("no descriptor layout")?;
    let pool = device
        .create_descriptor_pool(
            &vk::DescriptorPoolCreateInfo::default()
                .max_sets(1)
                .pool_sizes(std::slice::from_ref(
                    &vk::DescriptorPoolSize::default()
                        .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                        .descriptor_count(1),
                )),
            None,
        )
        .context("no descriptor pool")?;
    let sets = device
        .allocate_descriptor_sets(
            &vk::DescriptorSetAllocateInfo::default()
                .descriptor_pool(pool)
                .set_layouts(std::slice::from_ref(&layout)),
        )
        .context("no descriptor set")?;
    let set = sets[0];

    let image = device
        .create_image(
            &vk::ImageCreateInfo::default()
                .image_type(vk::ImageType::TYPE_2D)
                .format(vk::Format::R8G8B8A8_UNORM)
                .extent(vk::Extent3D { width, height, depth: 1 })
                .mip_levels(1)
                .array_layers(1)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .initial_layout(vk::ImageLayout::UNDEFINED),
            None,
        )
        .context("no atlas image")?;
    let requirements = device.get_image_memory_requirements(image);
    let memory = device
        .allocate_memory(
            &vk::MemoryAllocateInfo::default()
                .allocation_size(requirements.size)
                .memory_type_index(find_memory_type(
                    instance,
                    physical,
                    requirements.memory_type_bits,
                    vk::MemoryPropertyFlags::DEVICE_LOCAL,
                )?),
            None,
        )
        .context("no atlas memory")?;
    device
        .bind_image_memory(image, memory, 0)
        .context("the atlas could not be bound")?;

    let staging = resident(
        device,
        instance,
        physical,
        sheet.len() as vk::DeviceSize,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )?;
    std::ptr::copy_nonoverlapping(sheet.as_ptr(), staging.2.cast::<u8>(), sheet.len());

    let view = device
        .create_image_view(
            &vk::ImageViewCreateInfo::default()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(vk::Format::R8G8B8A8_UNORM)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                }),
            None,
        )
        .context("no atlas view")?;
    let sampler = device
        .create_sampler(
            &vk::SamplerCreateInfo::default()
                .mag_filter(vk::Filter::NEAREST)
                .min_filter(vk::Filter::NEAREST)
                .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .border_color(vk::BorderColor::INT_TRANSPARENT_BLACK)
                .unnormalized_coordinates(false),
            None,
        )
        .context("no sampler")?;

    device
        .update_descriptor_sets(
            std::slice::from_ref(&vk::WriteDescriptorSet::default().
                dst_set(set).
                dst_binding(0).
                descriptor_count(1).
                descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).
                image_info(std::slice::from_ref(&vk::DescriptorImageInfo::default().
                    sampler(sampler).
                    image_view(view).
                    image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)))),
            &[],
        );

    // The one-time upload: a staging copy is allowed for the **atlas** (an asset the
    // viewer baked itself), and W.5's no-staging rule belongs to the genome.
    upload(device, queue, family, image, staging.0, width, height)?;
    device.destroy_buffer(staging.0, None);
    device.free_memory(staging.1, None);
    Ok((layout, pool, sampler, image, memory, view, set))
}

/// A persistently mapped batch buffer: `HOST_VISIBLE|HOST_COHERENT`, mapped once and
/// written in place. [07 §9]'s W.5 idiom, applied to the viewer's own geometry,
/// which is why a frame does not allocate.
unsafe fn resident_batch(
    device: &ash::Device,
    instance: &ash::Instance,
    physical: vk::PhysicalDevice,
) -> Result<(vk::Buffer, vk::DeviceMemory, *mut std::ffi::c_void)> {
    resident(
        device,
        instance,
        physical,
        (MAX_QUADS * VERTEX_BYTES) as vk::DeviceSize,
        vk::BufferUsageFlags::VERTEX_BUFFER,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )
}

unsafe fn resident(
    device: &ash::Device,
    instance: &ash::Instance,
    physical: vk::PhysicalDevice,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Buffer, vk::DeviceMemory, *mut std::ffi::c_void)> {
    let buffer = device
        .create_buffer(&vk::BufferCreateInfo::default().size(size).usage(usage), None)
        .context("no buffer")?;
    let requirements = device.get_buffer_memory_requirements(buffer);
    let memory = device
        .allocate_memory(
            &vk::MemoryAllocateInfo::default()
                .allocation_size(requirements.size)
                .memory_type_index(find_memory_type(
                    instance,
                    physical,
                    requirements.memory_type_bits,
                    properties,
                )?),
            None,
        )
        .context("no memory")?;
    device
        .bind_buffer_memory(buffer, memory, 0)
        .context("the buffer could not be bound")?;
    let pointer = device
        .map_memory(memory, 0, size, vk::MemoryMapFlags::empty())
        .context("the mapping refused")?;
    Ok((buffer, memory, pointer))
}

unsafe fn find_memory_type(
    instance: &ash::Instance,
    physical: vk::PhysicalDevice,
    bits: u32,
    properties: vk::MemoryPropertyFlags,
) -> Result<u32> {
    let memory = instance.get_physical_device_memory_properties(physical);
    for index in 0..memory.memory_type_count {
        let candidate = &memory.memory_types[usize::try_from(index).expect("a 32-bit index")];
        if candidate.property_flags.contains(properties) && (bits & (1 << index)) != 0 {
            return Ok(index);
        }
    }
    bail!("the device has no memory type with the flags the viewer asked for")
}

/// Both stages, from [`crate::spirv`]'s hand-assembled words.
///
/// One pipeline, one primitive: the whole viewer is a triangle list of quads, so
/// there is no widget that could grow behaviour ([08 §9]) and nothing here that
/// could re-simulate a walk ([08 §1]'s third law).
unsafe fn build_pipeline(
    device: &ash::Device,
    layout: vk::PipelineLayout,
    render_pass: vk::RenderPass,
) -> Result<vk::Pipeline> {
    let vertex_words = crate::spirv::vertex_module();
    let fragment_words = crate::spirv::fragment_module();
    let vertex = device
        .create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&vertex_words), None)
        .context("the driver refused the vertex module")?;
    let fragment = device
        .create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&fragment_words), None)
        .context("the driver refused the fragment module")?;
    let entry = CString::new("main").expect("a name without a nul byte");
    let stages = [
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vertex)
            .name(&entry),
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(fragment)
            .name(&entry),
    ];
    let bindings = [vk::VertexInputBindingDescription::default()
        .binding(0)
        .stride(VERTEX_BYTES as u32)
        .input_rate(vk::VertexInputRate::VERTEX)];
    let attributes = [
        vk::VertexInputAttributeDescription::default()
            .location(0)
            .binding(0)
            .format(vk::Format::R32G32B32A32_SFLOAT)
            .offset(POSITION_OFFSET as u32),
        vk::VertexInputAttributeDescription::default()
            .location(1)
            .binding(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(UV_OFFSET as u32),
        vk::VertexInputAttributeDescription::default()
            .location(2)
            .binding(0)
            .format(vk::Format::R32G32B32A32_SFLOAT)
            .offset(COLOUR_OFFSET as u32),
    ];
    let input = vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(&bindings)
        .vertex_attribute_descriptions(&attributes);
    let assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
    let viewport = vk::PipelineViewportStateCreateInfo::default()
        .viewport_count(1)
        .scissor_count(1);
    let raster = vk::PipelineRasterizationStateCreateInfo::default()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(vk::CullModeFlags::NONE)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .depth_bias_enable(false)
        .line_width(1.0);
    let multisample = vk::PipelineMultisampleStateCreateInfo::default()
        .rasterization_samples(vk::SampleCountFlags::TYPE_1)
        .sample_shading_enable(false);
    let blend = vk::PipelineColorBlendAttachmentState::default()
        .blend_enable(true)
        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_write_mask(vk::ColorComponentFlags::RGBA);
    let blending = vk::PipelineColorBlendStateCreateInfo::default()
        .logic_op_enable(false)
        .attachments(std::slice::from_ref(&blend));
    let dynamic = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic);
    let create = vk::GraphicsPipelineCreateInfo::default()
        .stages(&stages)
        .vertex_input_state(&input)
        .input_assembly_state(&assembly)
        .viewport_state(&viewport)
        .rasterization_state(&raster)
        .multisample_state(&multisample)
        .color_blend_state(&blending)
        .dynamic_state(&dynamic_state)
        .layout(layout)
        .render_pass(render_pass)
        .subpass(0);
    let created = device
        .create_graphics_pipelines(vk::PipelineCache::null(), std::slice::from_ref(&create), None);
    device.destroy_shader_module(vertex, None);
    device.destroy_shader_module(fragment, None);
    let pipelines = match created {
        Ok(pipelines) => pipelines,
        Err((_, code)) => bail!("the driver refused the panel pipeline: {code}"),
    };
    pipelines
        .first()
        .copied()
        .context("the driver returned no pipeline")
}

/// The swapchain, its views and its framebuffers.
mod swapchain {
    use super::*;

    pub(super) unsafe fn create(
        device: &ash::Device,
        swapchain_ext: &khr::swapchain::Device,
        surface_ext: &khr::surface::Instance,
        physical: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        render_pass: vk::RenderPass,
        extent: (u32, u32),
        graphics_family: u32,
        present_family: u32,
    ) -> Result<(
        vk::SwapchainKHR,
        Vec<vk::Image>,
        Vec<vk::ImageView>,
        Vec<vk::Framebuffer>,
        (u32, u32),
    )> {
        let capabilities = surface_ext
            .get_physical_device_surface_capabilities(physical, surface)
            .context("the surface has no capabilities")?;
        let formats = surface_ext
            .get_physical_device_surface_formats(physical, surface)
            .context("the surface has no format")?;
        if formats.is_empty() {
            bail!("the surface reports no format the viewer can draw into");
        }
        let modes = surface_ext
            .get_physical_device_surface_present_modes(physical, surface)
            .unwrap_or_default();
        // FIFO is the only mode the spec guarantees, and it is the one that cannot
        // make the viewer's cadence a third clock (§4's "one clock" law reaches the
        // swapchain too).
        let present_mode = if modes.contains(&vk::PresentModeKHR::FIFO) {
            vk::PresentModeKHR::FIFO
        } else {
            *modes.first().context("the surface offers no present mode at all")?
        };
        let format = formats
            .iter()
            .find(|candidate| candidate.format == vk::Format::B8G8R8A8_UNORM)
            .copied()
            .unwrap_or(formats[0]);
        let actual = if capabilities.current_extent.width == u32::MAX {
            extent
        } else {
            (capabilities.current_extent.width, capabilities.current_extent.height)
        };
        let min_extent = capabilities.min_image_extent;
        let max_extent = capabilities.max_image_extent;
        let clamped = (
            actual.0.clamp(min_extent.width, max_extent.width),
            actual.1.clamp(min_extent.height, max_extent.height),
        );
        let mut count = capabilities.min_image_count.max(2);
        if capabilities.max_image_count > 0 {
            count = count.min(capabilities.max_image_count);
        }
        let (image_array_layers, surface_transform, composite_alpha) = (
            // A swapchain image is one layer; the field is a u32 and 1 is always
            // supported, so this is a constant rather than a negotiation.
            1u32,
            capabilities.current_transform,
            if capabilities
                .supported_composite_alpha
                .contains(vk::CompositeAlphaFlagsKHR::INHERIT)
            {
                vk::CompositeAlphaFlagsKHR::INHERIT
            } else {
                vk::CompositeAlphaFlagsKHR::OPAQUE
            },
        );
        let swapchain = swapchain_ext
            .create_swapchain(
                &vk::SwapchainCreateInfoKHR::default()
                    .surface(surface)
                    .min_image_count(count)
                    .image_format(format.format)
                    .image_color_space(format.color_space)
                    .image_extent(vk::Extent2D {
                        width: clamped.0,
                        height: clamped.1,
                    })
                    .image_array_layers(image_array_layers)
                    .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                    .image_sharing_mode(if graphics_family == present_family {
                        vk::SharingMode::EXCLUSIVE
                    } else {
                        vk::SharingMode::CONCURRENT
                    })
                    .queue_family_indices(&[graphics_family, present_family])
                    .pre_transform(surface_transform)
                    .composite_alpha(composite_alpha)
                    .present_mode(present_mode)
                    .clipped(true),
                None,
            )
            .context("the driver refused the swapchain")?;
        let images = swapchain_ext
            .get_swapchain_images(swapchain)
            .context("no swapchain images")?;
        let mut views = Vec::with_capacity(images.len());
        let mut framebuffers = Vec::with_capacity(images.len());
        for image in &images {
            let view = device
                .create_image_view(
                    &vk::ImageViewCreateInfo::default()
                        .image(*image)
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(format.format)
                        .subresource_range(vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        }),
                    None,
                )
                .context("no swapchain view")?;
            let framebuffer = device
                .create_framebuffer(
                    &vk::FramebufferCreateInfo::default()
                        .render_pass(render_pass)
                        .attachments(std::slice::from_ref(&view))
                        .width(clamped.0)
                        .height(clamped.1)
                        .layers(1),
                    None,
                )
                .context("no framebuffer")?;
            views.push(view);
            framebuffers.push(framebuffer);
        }
        Ok((swapchain, images, views, framebuffers, clamped))
    }
}

unsafe fn semaphores(device: &ash::Device, count: usize) -> Result<Vec<vk::Semaphore>> {
    let info = vk::SemaphoreCreateInfo::default();
    (0..count)
        .map(|_| device.create_semaphore(&info, None).map_err(|e| anyhow::anyhow!("{e:?}")))
        .collect()
}

unsafe fn fences(device: &ash::Device, count: usize) -> Result<Vec<vk::Fence>> {
    let info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
    (0..count)
        .map(|_| device.create_fence(&info, None).map_err(|e| anyhow::anyhow!("{e:?}")))
        .collect()
}

/// The one-time upload of the baked sheet, with both layout transitions in the same
/// command buffer.
///
/// The only host wait in this module that is *not* counted by
/// [`Context::host_waits`]: it happens once, before the first frame, on the viewer's
/// own queue, and §7's sync row is about a GUI adding a host sync into the frame
/// path. The atlas is the one asset here, and W.5's no-staging rule belongs to the
/// genome, not to a sheet this crate baked.
unsafe fn upload(
    device: &ash::Device,
    queue: vk::Queue,
    family: u32,
    image: vk::Image,
    staging: vk::Buffer,
    width: u32,
    height: u32,
) -> Result<()> {
    let pool = device
        .create_command_pool(
            &vk::CommandPoolCreateInfo::default().queue_family_index(family),
            None,
        )
        .context("no upload pool")?;
    let buffers = device
        .allocate_command_buffers(
            &vk::CommandBufferAllocateInfo::default()
                .command_pool(pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1),
        )
        .context("no upload buffer")?;
    let command = buffers[0];
    device
        .begin_command_buffer(
            command,
            &vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
        )
        .context("upload begin")?;
    device.cmd_pipeline_barrier(
        command,
        vk::PipelineStageFlags::TOP_OF_PIPE,
        vk::PipelineStageFlags::TRANSFER,
        vk::DependencyFlags::empty(),
        &[],
        &[],
        std::slice::from_ref(&vk::ImageMemoryBarrier::default().
            src_access_mask(vk::AccessFlags::empty()).
            dst_access_mask(vk::AccessFlags::TRANSFER_WRITE).
            old_layout(vk::ImageLayout::UNDEFINED).
            new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL).
            image(image).
            subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })),
    );
    device.cmd_copy_buffer_to_image(
        command,
        staging,
        image,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        std::slice::from_ref(&vk::BufferImageCopy::default().
            image_subresource(vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            }).
            image_extent(vk::Extent3D { width, height, depth: 1 })),
    );
    device.cmd_pipeline_barrier(
        command,
        vk::PipelineStageFlags::TRANSFER,
        vk::PipelineStageFlags::FRAGMENT_SHADER,
        vk::DependencyFlags::empty(),
        &[],
        &[],
        std::slice::from_ref(&vk::ImageMemoryBarrier::default().
            src_access_mask(vk::AccessFlags::TRANSFER_WRITE).
            dst_access_mask(vk::AccessFlags::INPUT_ATTACHMENT_READ | vk::AccessFlags::SHADER_READ).
            old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL).
            new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL).
            image(image).
            subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })),
    );
    device.end_command_buffer(command).context("upload end")?;
    device
        .queue_submit(
            queue,
            std::slice::from_ref(&vk::SubmitInfo::default().command_buffers(std::slice::from_ref(&command))),
            vk::Fence::null(),
        )
        .context("upload submit")?;
    device.queue_wait_idle(queue).context("upload wait")?;
    device.destroy_command_pool(pool, None);
    Ok(())
}

impl Context {
    /// One frame: write the batch, acquire, record, submit, present.
    ///
    /// `Err` is the device-loss path and the caller turns it into
    /// [`Outcome::Degraded`]; it is never a panic, because a viewer that dies loudly
    /// is a viewer the owner has to debug instead of read.
    ///
    /// There is no host wait in this path. Pacing rides [`Self::in_flight`]'s fence,
    /// which is the viewer's own submission bookkeeping ([02 §10]: "the viewer's own
    /// submissions are its business"), not a `device_wait_idle` — and a frame that
    /// needed one would show up in §7's sync row.
    unsafe fn draw(&mut self, batch: &Batch) -> Result<()> {
        self.write_batch(batch);
        let slot = self.next_slot();
        let index = self.acquire(slot)?;
        let command = self.record(index, slot, batch)?;
        self.submit_frame(slot, command)?;
        self.present(index, slot)
    }

    /// The slot for this frame, with its in-flight fence already waited and reset.
    ///
    /// Blocking on the *slot's* fence is what keeps one frame per image in flight.
    unsafe fn next_slot(&mut self) -> usize {
        let slot = self.slot % self.in_flight.len().max(1);
        self.slot = self.slot + 1;
        let fence = self.in_flight[slot];
        let _ = self.device.wait_for_fences(std::slice::from_ref(&fence), true, u64::MAX);
        let _ = self.device.reset_fences(std::slice::from_ref(&fence));
        slot
    }

    unsafe fn acquire(&mut self, slot: usize) -> Result<usize> {
        let (index, _suboptimal) = self
            .swapchain_ext
            .acquire_next_image(
                self.swapchain,
                u64::MAX,
                self.draw_semaphores[slot],
                vk::Fence::null(),
            )
            .context("acquire failed: the device is lost")?;
        Ok(index as usize)
    }

    /// Record into the slot's command buffer: the clear, the one pipeline, the one
    /// vertex buffer and the one descriptor set.
    unsafe fn record(
        &self,
        index: usize,
        slot: usize,
        batch: &Batch,
    ) -> Result<vk::CommandBuffer> {
        let command = self.commands[slot];
        self.device
            .reset_command_buffer(command, vk::CommandBufferResetFlags::empty())
            .context("the frame buffer would not reset")?;
        self.device
            .begin_command_buffer(command, &vk::CommandBufferBeginInfo::default())
            .context("frame begin")?;
        let clear = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.07, 0.08, 0.10, 1.0],
            },
        }];
        self.device.cmd_begin_render_pass(
            command,
            &vk::RenderPassBeginInfo::default()
                .render_pass(self.render_pass)
                .framebuffer(self.framebuffers[index])
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: vk::Extent2D {
                        width: self.extent.0,
                        height: self.extent.1,
                    },
                })
                .clear_values(&clear),
            vk::SubpassContents::INLINE,
        );
        self.device.cmd_bind_pipeline(
            command,
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline,
        );
        let viewport = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: self.extent.0 as f32,
            height: self.extent.1 as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        self.device.cmd_set_viewport(command, 0, &viewport);
        let scissor = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D {
                width: self.extent.0,
                height: self.extent.1,
            },
        }];
        self.device.cmd_set_scissor(command, 0, &scissor);
        self.device.cmd_bind_vertex_buffers(command, 0, &[self.vertex_buffer], &[0]);
        self.device.cmd_bind_descriptor_sets(
            command,
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline_layout,
            0,
            &[self.descriptor_set],
            &[],
        );
        self.device.cmd_draw(command, batch.vertices.len() as u32, 1, 0, 0);
        self.device.cmd_end_render_pass(command);
        self.device.end_command_buffer(command).context("frame end")?;
        Ok(command)
    }

    unsafe fn submit_frame(&self, slot: usize, command: vk::CommandBuffer) -> Result<()> {
        // The acquire semaphore is waited on *at the stage that writes the swapchain
        // image*, which is what `pWaitDstStageMask` says: a submit that waits with a
        // NULL mask is a spec violation the driver accepts quietly and the validation
        // layer names.
        let wait_stage = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        self.device
            .queue_submit(
                self.queue,
                std::slice::from_ref(
                    &vk::SubmitInfo::default()
                        .wait_semaphores(std::slice::from_ref(&self.draw_semaphores[slot]))
                        .wait_dst_stage_mask(&wait_stage)
                        .command_buffers(std::slice::from_ref(&command))
                        .signal_semaphores(std::slice::from_ref(&self.present_semaphores[slot])),
                ),
                self.in_flight[slot],
            )
            .context("submit failed: the device is lost")
    }

    unsafe fn present(&self, index: usize, slot: usize) -> Result<()> {
        let image_index = index as u32;
        self.swapchain_ext
            .queue_present(
                self.queue,
                &vk::PresentInfoKHR::default()
                    .wait_semaphores(std::slice::from_ref(&self.present_semaphores[slot]))
                    .swapchains(std::slice::from_ref(&self.swapchain))
                    .image_indices(std::slice::from_ref(&image_index)),
            )
            .context("present failed: the device is lost")?;
        Ok(())
    }

    /// A resize: drop the swapchain's images and build the next ones.
    ///
    /// This is the one host wait in the frame path, and it is **counted** rather than
    /// hidden: §7's sync row exists so that a GUI-induced device sync is visible as a
    /// number instead of a rumour. A resize is the owner's action, not SI's, so it is
    /// legal to pay for — and a run that never resized reads `0`.
    fn recreate(&mut self, width: u32, height: u32) -> Result<()> {
        unsafe {
            let _ = self.device.device_wait_idle();
            self.host_waits += 1;
            for framebuffer in &self.framebuffers {
                self.device.destroy_framebuffer(*framebuffer, None);
            }
            for view in &self.views {
                self.device.destroy_image_view(*view, None);
            }
            self.swapchain_ext.destroy_swapchain(self.swapchain, None);
            match swapchain::create(
                &self.device,
                &self.swapchain_ext,
                &self.surface_ext,
                self.physical,
                self.surface,
                self.render_pass,
                (width, height),
                self.graphics_family,
                self.present_family,
            ) {
                Ok((swapchain, images, views, framebuffers, extent)) => {
                    self.swapchain = swapchain;
                    self.images = images;
                    self.views = views;
                    self.framebuffers = framebuffers;
                    self.extent = extent;
                    // A driver may hand back a different image count than the last
                    // swapchain had. The frame slot is one command buffer, one draw
                    // semaphore, one present semaphore and one fence *per image*, so
                    // the sets are rebuilt rather than left short.
                    if self.images.len() != self.commands.len() {
                        let count = self.images.len();
                        self.device.free_command_buffers(self.command_pool, &self.commands);
                        self.commands = self
                            .device
                            .allocate_command_buffers(
                                &vk::CommandBufferAllocateInfo::default()
                                    .command_pool(self.command_pool)
                                    .level(vk::CommandBufferLevel::PRIMARY)
                                    .command_buffer_count(count as u32),
                            )
                            .context("the resize left no command buffers")?;
                        for fence in &self.in_flight {
                            self.device.destroy_fence(*fence, None);
                        }
                        for semaphore in
                            self.draw_semaphores.iter().chain(&self.present_semaphores)
                        {
                            self.device.destroy_semaphore(*semaphore, None);
                        }
                        self.draw_semaphores = semaphores(&self.device, count)?;
                        self.present_semaphores = semaphores(&self.device, count)?;
                        self.in_flight = fences(&self.device, count)?;
                        self.slot = 0;
                    }
                    Ok(())
                }
                Err(error) => {
                    // A failed recreate leaves the old handles destroyed and the next
                    // draw's acquire returning an error, which is the degradation path.
                    self.extent = (width, height);
                    Err(error)
                }
            }
        }
    }

    /// Write the frame's vertices into the persistent mapping. No staging copy, no
    /// allocation: §7's resident-bytes row counts one buffer for the whole session.
    unsafe fn write_batch(&self, batch: &Batch) {
        let bytes = bytemuck::cast_slice(&batch.vertices);
        if bytes.is_empty() {
            return;
        }
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), self.vertex_pointer.cast::<u8>(), bytes.len());
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            // Teardown, not a frame: the wait that ends a session is not the host
            // sync §7's sync row counts, and by the time this runs the PERF panel has
            // already been printed with the frame path's real number.
            let _ = self.device.device_wait_idle();
            // Dependency order, not declaration order: a descriptor set names the
            // sampler and the view it was written with, a framebuffer names the
            // view, and a mapped buffer must be unmapped before it is destroyed.
            // Vulkan destroys a device with live children as undefined behaviour, so
            // the leaves go first and `destroy_device` goes last.
            // Destroying the pool releases the set it handed out; `free_descriptor_sets`
            // would need `FREE_DESCRIPTOR_SET_BIT`, a flag this pool does not carry and
            // a lifetime this session does not have (one set, for the whole run).
            self.device.destroy_descriptor_pool(self.descriptor_pool, None);
            self.device.destroy_descriptor_set_layout(self.descriptor_layout, None);
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.pipeline_layout, None);
            for framebuffer in &self.framebuffers {
                self.device.destroy_framebuffer(*framebuffer, None);
            }
            for view in &self.views {
                self.device.destroy_image_view(*view, None);
            }
            self.swapchain_ext.destroy_swapchain(self.swapchain, None);
            self.device.destroy_sampler(self.sampler, None);
            self.device.destroy_image_view(self.atlas_view, None);
            self.device.destroy_image(self.atlas_image, None);
            self.device.free_memory(self.atlas_memory, None);
            self.device.unmap_memory(self.vertex_memory);
            self.device.destroy_buffer(self.vertex_buffer, None);
            self.device.free_memory(self.vertex_memory, None);
            self.device.destroy_command_pool(self.command_pool, None);
            for fence in &self.in_flight {
                self.device.destroy_fence(*fence, None);
            }
            for semaphore in self.draw_semaphores.iter().chain(&self.present_semaphores) {
                self.device.destroy_semaphore(*semaphore, None);
            }
            self.device.destroy_render_pass(self.render_pass, None);
            self.device.destroy_device(None);
            self.surface_ext.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
            // `entry` drops with the rest of the fields, after the last call: the
            // loader library stays mapped until this point.
        }
    }
}

/// [02 §10]'s first row: the genome's read-only bytes, resident as persistently
/// mapped `HOST_VISIBLE|HOST_COHERENT` with **no staging copy**, and the
/// `flush_count` [07 §9] W.5 exports.
///
/// `bytes` is handed over by the launcher, which is the crate that owns the pinned
/// `SIgen` mapping ([00 §8], **OD-148**): this function takes the bytes of the
/// merged CSR runs and never opens a file.
pub struct Residency {
    context: *const Context,
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    pointer: *mut std::ffi::c_void,
    length: vk::DeviceSize,
    /// How many times the host flushed this mapping. W.5's rule is "flush only when
    /// the host reads or overwrites a live buffer", and the viewer only ever reads,
    /// so on the v1 CPU path **this stays `0`** and the PERF panel prints it as such.
    pub flush_count: u64,
}

impl std::fmt::Debug for Residency {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Residency")
            .field("length", &self.length)
            .field("flush_count", &self.flush_count)
            .finish_non_exhaustive()
    }
}

/// Map the handed-over genome bytes into a persistently mapped device buffer.
///
/// # Safety
///
/// The `context` must outlive the returned [`Residency`], because its `Drop` calls
/// into the same device.
pub unsafe fn genome_residency(context: &Context, bytes: &[u8]) -> Result<Residency> {
    if bytes.is_empty() {
        bail!("an empty genome is not a residency: CONFIG SI has no frozen genome at all (07 §9)");
    }
    let (buffer, memory, pointer) = resident(
        &context.device,
        &context.instance,
        context.physical,
        bytes.len() as vk::DeviceSize,
        vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )?;
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), pointer.cast::<u8>(), bytes.len());
    Ok(Residency {
        context: context as *const Context,
        buffer,
        memory,
        pointer,
        length: bytes.len() as vk::DeviceSize,
        flush_count: 0,
    })
}

impl Residency {
    /// The resident length in bytes, for §7's split row: genome vs live L3 vs lanes.
    pub fn length(&self) -> u64 {
        self.length
    }

    /// W.5's flush, which the **host-coherent** mapping makes a no-op for a reader.
    /// It exists because the spec exports the counter, and it increments only on an
    /// explicit call — never inside a draw, so the PERF panel's `0` is a fact about
    /// the walk being CPU (OD-44), not a bookkeeping accident.
    pub fn flush(&mut self, context: &Context) -> Result<()> {
        // A safe method wrapping one unsafe entry point: the flush is the only place
        // this crate tells the driver the host touched a live buffer (W.5).
        unsafe {
            context
                .device
                .flush_mapped_memory_ranges(std::slice::from_ref(
                    &vk::MappedMemoryRange::default().memory(self.memory).size(self.length),
                ))
            .context("the flush refused")?;
        }
        self.flush_count += 1;
        Ok(())
    }

    /// [02 §10]'s third row: edges formed **between** thoughts, uploaded into the
    /// overlay. Never mid-thought (§16.5's CPU/GPU split), which is why this is an
    /// explicit call the session makes at a commit rather than something a draw does.
    pub fn upload_overlay(&mut self, context: &Context, edges: &[u8]) -> Result<()> {
        if edges.len() as u64 > self.length {
            bail!(
                "the overlay is {} bytes and the residency {} — an overlay that does not fit \
                 is a budget failure, not a resize",
                edges.len(),
                self.length
            );
        }
        unsafe {
            std::ptr::copy_nonoverlapping(edges.as_ptr(), self.pointer.cast::<u8>(), edges.len());
        }
        // An overwrite of a live buffer is exactly W.5's flush condition.
        self.flush(context)
    }
}

impl Drop for Residency {
    fn drop(&mut self) {
        let context = unsafe { &*self.context };
        unsafe {
            context.device.unmap_memory(self.memory);
            context.device.destroy_buffer(self.buffer, None);
            context.device.free_memory(self.memory, None);
        }
    }
}

/// The inbox path, kept as a function so the launcher can report **where** it wrote
/// without this crate naming a second path.
pub fn inbox_path(session: &Session) -> PathBuf {
    session.inbox()
}

/// The lane file the launcher reads, resolved against the run directory. Kept here
/// so the name lives in one place and `tests/laws.rs` can count it.
pub fn lane_path(run_dir: &Path, name: &str) -> PathBuf {
    run_dir.join(name)
}

/// The `Handover` for a replay: one read of the run directory, then the same panels
/// forever (§2.2's "a gate artifact is inspectable with the product's own eyes").
pub struct Replay {
    lane: Vec<u8>,
    artifacts: Artifacts,
    profile: bool,
}

impl Replay {
    pub fn new(lane: Vec<u8>, artifacts: Artifacts, profile: bool) -> Self {
        Self { lane, artifacts, profile }
    }
}

impl Handover for Replay {
    fn lane_tail(&mut self) -> Result<Vec<u8>> {
        Ok(self.lane.clone())
    }

    fn artifacts(&mut self) -> Result<Artifacts> {
        Ok(self.artifacts.clone())
    }

    fn profile(&self) -> bool {
        self.profile
    }
}

/// The live handover: the tail is re-read every frame, which is the whole of
/// §2.2's back-pressure rule — the viewer reads more or less often, and the loop
/// never waits for it.
///
/// **One poll per frame.** A composition asks for the lane and then for the
/// artifacts, and those two answers must describe the same instant; a source read
/// twice would be a viewer that showed a lane from one moment and a snapshot from
/// the next. [`Handover::begin_frame`] is where the cache is dropped.
pub struct Live<P>
where
    P: FnMut() -> Result<(Vec<u8>, Artifacts, bool)>,
{
    source: P,
    polled: Option<(Vec<u8>, Artifacts, bool)>,
}

impl<P> Live<P>
where
    P: FnMut() -> Result<(Vec<u8>, Artifacts, bool)>,
{
    pub fn new(source: P) -> Self {
        Self { source, polled: None }
    }

    fn poll(&mut self) -> Result<()> {
        if self.polled.is_none() {
            self.polled = Some((self.source)()?);
        }
        Ok(())
    }
}

impl<P> Handover for Live<P>
where
    P: FnMut() -> Result<(Vec<u8>, Artifacts, bool)>,
{
    fn begin_frame(&mut self) {
        self.polled = None;
    }

    fn lane_tail(&mut self) -> Result<Vec<u8>> {
        self.poll()?;
        Ok(self
            .polled
            .as_ref()
            .map(|(lane, _, _)| lane.clone())
            .unwrap_or_default())
    }

    fn artifacts(&mut self) -> Result<Artifacts> {
        self.poll()?;
        self.polled
            .as_ref()
            .map(|(_, artifacts, _)| artifacts.clone())
            .context("the frame's poll left nothing cached")
    }

    fn profile(&self) -> bool {
        self.polled.as_ref().map(|(_, _, profile)| *profile).unwrap_or(false)
    }
}
