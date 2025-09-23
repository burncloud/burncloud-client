use systray::Application;
use std::process;
use std::fmt;
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub struct SimpleError(String);

impl fmt::Display for SimpleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for SimpleError {}

#[derive(Debug, Clone)]
pub enum WindowMessage {
    Show,
    Hide,
}

static WINDOW_SENDER: once_cell::sync::OnceCell<Sender<WindowMessage>> = once_cell::sync::OnceCell::new();

pub fn set_window_sender(sender: Sender<WindowMessage>) {
    let _ = WINDOW_SENDER.set(sender);
}

/// 启动 BurnCloud 托盘应用
pub fn start_tray() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = Application::new()?;

    // 直接使用 res/burncloud.ico 作为默认图标
    let ico_path = std::path::Path::new("./res/burncloud.ico");

    // 尝试设置图标，如果失败则使用默认方式
    match app.set_icon_from_file(&ico_path.to_string_lossy()) {
        Ok(_) => {},
        Err(_) => {
            println!("Warning: Failed to set custom icon, using default");
        }
    }

    // 添加启动界面菜单项
    app.add_menu_item(&"显示界面".to_string(), move |_| -> Result<(), SimpleError> {
        if let Some(sender) = WINDOW_SENDER.get() {
            let _ = sender.send(WindowMessage::Show);
        }
        Ok(())
    })?;

    // 添加分隔符
    app.add_menu_separator()?;

    // 添加退出菜单项
    app.add_menu_item(&"退出程序".to_string(), |_| -> Result<(), SimpleError> {
        process::exit(0);
    })?;

    // 等待
    app.wait_for_message()?;

    Ok(())
}