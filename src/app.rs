use dioxus::prelude::*;
use dioxus_router::prelude::*;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use once_cell::sync::OnceCell;

use crate::components::layout::Layout;
use crate::pages::{
    dashboard::Dashboard,
    models::ModelManagement,
    deploy::DeployConfig,
    monitor::ServiceMonitor,
    api::ApiManagement,
    settings::SystemSettings,
};
use crate::tray::{WindowMessage, set_window_sender};

// 全局窗口显示控制
static WINDOW_VISIBLE: OnceCell<Arc<Mutex<bool>>> = OnceCell::new();

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[layout(Layout)]
    #[route("/")]
    Dashboard {},
    #[route("/models")]
    ModelManagement {},
    #[route("/deploy")]
    DeployConfig {},
    #[route("/monitor")]
    ServiceMonitor {},
    #[route("/api")]
    ApiManagement {},
    #[route("/settings")]
    SystemSettings {},
}

#[component]
pub fn App() -> Element {
    let window = dioxus::desktop::use_window();

    use_effect(move || {
        window.set_maximized(true);
    });

    rsx! {
        Router::<Route> {}
    }
}

pub fn launch_gui() {
    launch_gui_impl();
}

pub fn launch_gui_with_tray() {
    // 初始化全局状态
    WINDOW_VISIBLE.set(Arc::new(Mutex::new(false))).unwrap();

    // 创建消息通道
    let (sender, receiver) = mpsc::channel::<WindowMessage>();

    // 设置全局发送器
    set_window_sender(sender);

    // 启动托盘应用在后台线程
    std::thread::spawn(|| {
        if let Err(e) = crate::tray::start_tray() {
            eprintln!("Failed to start tray: {}", e);
        }
    });

    // 启动消息处理线程
    start_message_handler(receiver);

    // 启动 GUI
    launch_gui_with_tray_impl();
}

fn start_message_handler(receiver: Receiver<WindowMessage>) {
    std::thread::spawn(move || {
        while let Ok(msg) = receiver.recv() {
            match msg {
                WindowMessage::Show => {
                    if let Some(visible) = WINDOW_VISIBLE.get() {
                        if let Ok(mut v) = visible.lock() {
                            *v = true;
                        }
                    }
                }
                WindowMessage::Hide => {
                    if let Some(visible) = WINDOW_VISIBLE.get() {
                        if let Ok(mut v) = visible.lock() {
                            *v = false;
                        }
                    }
                }
            }
        }
    });
}

fn launch_gui_with_tray_impl() {
    use dioxus::desktop::{Config, WindowBuilder};

    let window = WindowBuilder::new()
        .with_title("BurnCloud - 大模型本地部署平台")
        .with_inner_size(dioxus::desktop::LogicalSize::new(1200.0, 800.0))
        .with_resizable(true)
        .with_decorations(false);
        // .with_visible(false); // 初始不可见

    let config = Config::new().with_window(window);

    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(AppWithTray);
}

#[component]
fn AppWithTray() -> Element {
    let window = dioxus::desktop::use_window();
    let mut should_show = use_signal(|| false);

    let window_clone1 = window.clone();
    use_effect(move || {
        window_clone1.set_maximized(true);
    });

    // 定期检查是否需要显示窗口
    use_effect(move || {
        spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

                if let Some(visible) = WINDOW_VISIBLE.get() {
                    if let Ok(v) = visible.lock() {
                        if *v && !should_show() {
                            should_show.set(true);
                            break;
                        }
                    }
                }
            }
        });
    });

    // 当should_show变化时，显示窗口
    let window_clone2 = window.clone();
    use_effect(move || {
        if should_show() {
            let _ = window_clone2.set_visible(true);
        }
    });

    rsx! {
        Router::<Route> {}
    }
}

fn launch_gui_impl() {
    use dioxus::desktop::{Config, WindowBuilder};

    let window = WindowBuilder::new()
        .with_title("BurnCloud - 大模型本地部署平台")
        .with_inner_size(dioxus::desktop::LogicalSize::new(1200.0, 800.0))
        .with_resizable(true)
        .with_decorations(false);

    let config = Config::new().with_window(window);

    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(App);
}