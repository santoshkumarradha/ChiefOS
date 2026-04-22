#[cfg(not(target_os = "linux"))]
use anyhow::anyhow;
use anyhow::Result;

use crate::inbox::InboxItem;

pub const OVERLAY_WIDTH_PERCENT: u32 = 22;
pub const OVERLAY_HEIGHT_PERCENT: u32 = 68;

#[cfg(target_os = "linux")]
pub fn run_overlay(items: Vec<InboxItem>) -> Result<()> {
    linux::run(items)
}

#[cfg(not(target_os = "linux"))]
pub fn run_overlay(_items: Vec<InboxItem>) -> Result<()> {
    Err(anyhow!(
        "Wayland layer-shell overlay is only available on Linux wlroots compositors"
    ))
}

#[cfg(target_os = "linux")]
mod linux {
    use anyhow::{Context, Result};
    use smithay_client_toolkit::compositor::{CompositorHandler, CompositorState};
    use smithay_client_toolkit::delegate_compositor;
    use smithay_client_toolkit::delegate_layer;
    use smithay_client_toolkit::delegate_output;
    use smithay_client_toolkit::delegate_registry;
    use smithay_client_toolkit::output::{OutputHandler, OutputState};
    use smithay_client_toolkit::reexports::client::globals::registry_queue_init;
    use smithay_client_toolkit::reexports::client::protocol::{wl_output, wl_surface};
    use smithay_client_toolkit::reexports::client::{Connection, QueueHandle};
    use smithay_client_toolkit::registry::{ProvidesRegistryState, RegistryState};
    use smithay_client_toolkit::registry_handlers;
    use smithay_client_toolkit::shell::wlr_layer::{
        Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
        LayerSurfaceConfigure,
    };
    use smithay_client_toolkit::shell::WaylandSurface;

    use super::{InboxItem, OVERLAY_HEIGHT_PERCENT, OVERLAY_WIDTH_PERCENT};

    const DEFAULT_OUTPUT_WIDTH: u32 = 1440;
    const DEFAULT_OUTPUT_HEIGHT: u32 = 1050;
    const COMMIT_ROUNDS: usize = 4;
    const NAMESPACE: &str = "chief-hax-inbox";

    pub fn run(items: Vec<InboxItem>) -> Result<()> {
        let conn = Connection::connect_to_env().context("connect to Wayland compositor")?;
        let (globals, mut event_queue) = registry_queue_init::<State>(&conn)?;
        let qh = event_queue.handle();
        let mut state = State::new(&globals, &qh, items)?;
        state.create_overlay(&qh);
        for _ in 0..COMMIT_ROUNDS {
            event_queue.blocking_dispatch(&mut state)?;
            if state.closed {
                break;
            }
        }
        Ok(())
    }

    struct State {
        registry_state: RegistryState,
        output_state: OutputState,
        compositor_state: CompositorState,
        layer_shell: LayerShell,
        layer_surface: Option<LayerSurface>,
        items: Vec<InboxItem>,
        closed: bool,
    }

    impl State {
        fn new(
            globals: &smithay_client_toolkit::reexports::client::globals::GlobalList,
            qh: &QueueHandle<Self>,
            items: Vec<InboxItem>,
        ) -> Result<Self> {
            Ok(Self {
                registry_state: RegistryState::new(globals),
                output_state: OutputState::new(globals, qh),
                compositor_state: CompositorState::bind(globals, qh)?,
                layer_shell: LayerShell::bind(globals, qh)?,
                layer_surface: None,
                items,
                closed: false,
            })
        }

        fn create_overlay(&mut self, qh: &QueueHandle<Self>) {
            let surface = self.compositor_state.create_surface(qh);
            let output = self.output_state.outputs().next();
            let layer_surface = self.layer_shell.create_layer_surface(
                qh,
                surface,
                Layer::Overlay,
                Some(NAMESPACE),
                output.as_ref(),
            );
            configure_layer(&layer_surface, self.items.len());
            layer_surface.commit();
            self.layer_surface = Some(layer_surface);
        }
    }

    impl CompositorHandler for State {
        fn scale_factor_changed(
            &mut self,
            _conn: &Connection,
            _qh: &QueueHandle<Self>,
            _surface: &wl_surface::WlSurface,
            _new_factor: i32,
        ) {
        }

        fn surface_enter(
            &mut self,
            _conn: &Connection,
            _qh: &QueueHandle<Self>,
            _surface: &wl_surface::WlSurface,
            _output: &wl_output::WlOutput,
        ) {
        }

        fn surface_leave(
            &mut self,
            _conn: &Connection,
            _qh: &QueueHandle<Self>,
            _surface: &wl_surface::WlSurface,
            _output: &wl_output::WlOutput,
        ) {
        }
    }

    impl OutputHandler for State {
        fn output_state(&mut self) -> &mut OutputState {
            &mut self.output_state
        }

        fn new_output(
            &mut self,
            _conn: &Connection,
            _qh: &QueueHandle<Self>,
            _output: wl_output::WlOutput,
        ) {
        }

        fn update_output(
            &mut self,
            _conn: &Connection,
            _qh: &QueueHandle<Self>,
            _output: wl_output::WlOutput,
        ) {
        }

        fn output_destroyed(
            &mut self,
            _conn: &Connection,
            _qh: &QueueHandle<Self>,
            _output: wl_output::WlOutput,
        ) {
        }
    }

    impl LayerShellHandler for State {
        fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
            self.closed = true;
        }

        fn configure(
            &mut self,
            _conn: &Connection,
            _qh: &QueueHandle<Self>,
            layer: &LayerSurface,
            _configure: LayerSurfaceConfigure,
            serial: u32,
        ) {
            layer.ack_configure(serial);
            layer.commit();
        }
    }

    impl ProvidesRegistryState for State {
        registry_handlers![OutputState];

        fn registry(&mut self) -> &mut RegistryState {
            &mut self.registry_state
        }
    }

    fn configure_layer(layer: &LayerSurface, item_count: usize) {
        let width = scaled_dimension(DEFAULT_OUTPUT_WIDTH, OVERLAY_WIDTH_PERCENT);
        let height = scaled_dimension(DEFAULT_OUTPUT_HEIGHT, OVERLAY_HEIGHT_PERCENT);
        let margin = margin_for_count(item_count);
        layer.set_size(width, height);
        layer.set_anchor(Anchor::TOP | Anchor::RIGHT);
        layer.set_margin(margin, margin, 0, 0);
        layer.set_exclusive_zone(0);
        layer.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
    }

    fn scaled_dimension(base: u32, percent: u32) -> u32 {
        base.saturating_mul(percent).saturating_div(100)
    }

    fn margin_for_count(item_count: usize) -> i32 {
        i32::try_from(item_count.min(12)).unwrap_or(0)
    }

    delegate_compositor!(State);
    delegate_output!(State);
    delegate_layer!(State);
    delegate_registry!(State);
}
