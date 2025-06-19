use clap::{Parser, Subcommand};
use anstyle::{AnsiColor, Color, Style};



const FULL_HELP_MESSAGE: &str = "For more information, visit: https://github.com/zhuima/rnm\n\
    \n\
    \x1b[38;5;208m┌──────────────────────────────────────────────────────────────────────────────────────┐\x1b[0m\n\
    \x1b[38;5;208m│\x1b[0m               \x1b[1;38;5;226m✨ 本工具由 oomol.com 赞助的 OOMOL_Studio 强力驱动 ✨\x1b[0m\x1b[38;5;208m│\x1b[0m\n\
    \x1b[38;5;208m│\x1b[0m                        \x1b[1;38;5;219mhttps://oomol.com/！\x1b[0m\x1b[38;5;208m│\x1b[0m\n\
    \x1b[38;5;208m└──────────────────────────────────────────────────────────────────────────────────────┘\x1b[0m\n";

#[derive(Parser, Debug)]
#[command(
    author = "zhuima <zhuima314@gmail.com>",
    version,
    about = "A simple Node.js version manager, written in Rust",
    long_about = "A fast and user-friendly Node.js version manager written in Rust, supporting multiple Node.js versions.",
    before_help = "💫 RNM - Rust Node Manager",
    after_help = FULL_HELP_MESSAGE,
    styles = get_styles()
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .header(Style::new().bold().underline().fg_color(Some(Color::Ansi(AnsiColor::Green))))
        .usage(Style::new().bold().fg_color(Some(Color::Ansi(AnsiColor::Yellow))))
        .literal(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))))
        .placeholder(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan))))
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    // 列出远程版本
    #[command(name = "ls-remote", alias = "list-remote", about = "list remote versions")]
    LsRemote {
        // 可选的版本号，如果提供则只显示匹配的版本
        #[arg(help = "指定版本号，例如: 18 或 18.15 或 18.15.0")]
        version: Option<String>,
    },

    // 列出本地已安装版本
    #[command(name = "ls", alias = "list", about = "list installed versions")]
    Ls,

    // 下载安装指定版本
    #[command(name = "install", alias = "i", about = "install specified version")]
    Install {
        // 版本号
        version: String,
    },

    // 卸载指定版本
    #[command(name = "uninstall", alias = "u", about = "uninstall specified version")]
    Uninstall {
        // 版本号
        version: String,
    },

    // 设置默认版本
    #[command(name = "use", alias = "u", about = "set default version")]
    Use {
        // 版本号
        version: String,
    },

    // 查看当前版本
    #[command(name = "current", alias = "c", about = "show current version")]
    Current,
}