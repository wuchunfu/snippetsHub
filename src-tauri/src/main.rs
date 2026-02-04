/*
 * SnippetsHub - 代码片段管理工具
 *
 * @file main.rs - 应用二进制入口
 * @author Noah
 * @description Rust 应用程序的启动入口点
 * @version 1.0.0
 */

// Prevents additional console window on Windows in all builds
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

fn main() {
    snippets_hub_lib::run()
}
