//! Theme management — high-contrast CSS, CSS provider loading, memory monitoring.

use gtk4::glib;
use gtk4::prelude::*;

/// High-contrast CSS overrides for WCAG 2.1 AA compliance.
/// Applied at PRIORITY_APPLICATION + 1 to override the base theme.
const HC_CSS: &str = "\
@define-color theme_bg_color #000000;
@define-color theme_fg_color #ffffff;
@define-color theme_base_color #000000;
@define-color theme_text_color #ffffff;
@define-color theme_selected_bg_color #4A90D9;
@define-color theme_selected_fg_color #ffffff;
@define-color theme_unfocused_bg_color #000000;
@define-color theme_unfocused_fg_color #ffffff;
@define-color theme_unfocused_base_color #000000;
@define-color theme_unfocused_text_color #ffffff;
@define-color borders #ffffff;
@define-color theme_link_color #7AB5F5;
@define-color error_color #FF6B6B;
@define-color warning_color #FFD93D;
@define-color success_color #6BCB77;
#connection-indicator.disconnected { background-color: #FF6B6B; }
#connection-indicator.connected { background-color: #6BCB77; }
#connection-indicator.connecting { background-color: #FFD93D; }
.queue-artist { color: #CCCCCC; }
.format-badge { color: #CCCCCC; }
.error-label { color: #FF6B6B; }
#album-grid-status { color: #CCCCCC; }
.breadcrumb-sep { color: #CCCCCC; }
.queue-current { background-color: #4A90D9; }
";

/// Install the base stylesheet and, when requested by config or `GTK_THEME`,
/// the high-contrast overrides on the window's display.
pub(crate) fn install_css(window: &impl IsA<gtk4::Widget>, high_contrast_cfg: bool) {
    let display = window.as_ref().display();
    // Base CSS embedded at compile time
    let css = gtk4::CssProvider::new();
    css.load_from_bytes(&glib::Bytes::from_static(include_str!("style.css").as_bytes()));
    gtk4::style_context_add_provider_for_display(
        &display,
        &css,
        gtk4::STYLE_PROVIDER_PRIORITY_USER,
    );

    let gtk_theme = std::env::var("GTK_THEME");
    let should_enable_hc = high_contrast_cfg
        || gtk_theme.as_deref() == Ok("HighContrast")
        || gtk_theme.as_deref() == Ok("Adwaita:highcontrast");
    if should_enable_hc {
        let hc_provider = gtk4::CssProvider::new();
        hc_provider.load_from_string(HC_CSS);
        gtk4::style_context_add_provider_for_display(
            &display,
            &hc_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
        );
    }
}

/// Memory monitoring timer: checks RSS every 30s, warns via toast at most once per 60s.
pub(crate) fn start_memory_monitor(toast_overlay: adw::ToastOverlay) {
    let mut last_warning: Option<std::time::Instant> = None;
    glib::timeout_add_local(std::time::Duration::from_secs(30), move || {
        if let Some(rss_mb) = crate::memory::read_rss_mb() {
            let threshold = crate::config::Config::load().memory.warning_threshold_mb;
            if rss_mb >= threshold {
                let now = std::time::Instant::now();
                let should_warn = last_warning
                    .map(|t| now.duration_since(t).as_secs() >= 60)
                    .unwrap_or(true);
                if should_warn {
                    last_warning = Some(now);
                    let toast = adw::Toast::new(&crate::strings::memory_warning(rss_mb));
                    toast.set_timeout(5);
                    toast_overlay.add_toast(toast);
                }
            }
        } else {
            log::debug!("[memory] /proc/self/status unreadable, skipping memory check");
        }
        glib::ControlFlow::Continue
    });
}
