mod app;
mod ui;

use app::SpriteEditorApp;
use eframe::egui;
use eframe::egui_wgpu::{WgpuConfiguration, WgpuSetup, WgpuSetupCreateNew};
#[cfg(windows)]
use eframe::wgpu;
#[cfg(windows)]
use std::sync::Arc;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 720.0])
            .with_title("Sprite Editor - Ragnarok ACT/SPR"),
        wgpu_options: wgpu_options(),
        ..Default::default()
    };
    eframe::run_native(
        "Sprite Editor",
        options,
        Box::new(|_cc| Ok(Box::new(SpriteEditorApp::default()))),
    )
}

fn wgpu_options() -> WgpuConfiguration {
    #[allow(unused_mut)]
    let mut setup = WgpuSetupCreateNew::without_display_handle();

    #[cfg(windows)]
    {
        // DX12 d'abord : Vulkan Intel UHD plante souvent (ACCESS_VIOLATION).
        setup.instance_descriptor.backends = wgpu::Backends::DX12 | wgpu::Backends::VULKAN;
        setup.native_adapter_selector = Some(Arc::new(select_windows_adapter));
    }

    WgpuConfiguration {
        wgpu_setup: WgpuSetup::CreateNew(setup),
        ..Default::default()
    }
}

#[cfg(windows)]
fn select_windows_adapter(
    adapters: &[wgpu::Adapter],
    surface: Option<&wgpu::Surface<'_>>,
) -> Result<wgpu::Adapter, String> {
    let compatible = |adapter: &wgpu::Adapter| -> bool {
        match surface {
            Some(surface) => adapter.is_surface_supported(surface),
            None => true,
        }
    };

    let pick = |backend: wgpu::Backend| {
        adapters
            .iter()
            .find(|a| a.get_info().backend == backend && compatible(a))
            .cloned()
    };

    if let Some(adapter) = pick(wgpu::Backend::Dx12) {
        log_backend(&adapter);
        return Ok(adapter);
    }
    if let Some(adapter) = pick(wgpu::Backend::Vulkan) {
        log_backend(&adapter);
        return Ok(adapter);
    }

    Err(format!(
        "Aucun adaptateur DX12/Vulkan compatible ({} trouve(s))",
        adapters.len()
    ))
}

#[cfg(windows)]
fn log_backend(adapter: &wgpu::Adapter) {
    let info = adapter.get_info();
    eprintln!(
        "wgpu backend: {:?} — {} ({:?})",
        info.backend, info.name, info.device_type
    );
}
