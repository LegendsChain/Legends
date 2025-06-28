use substrate_build_script_utils::{generate_cargo_keys, rerun_if_git_head_changed};

fn main() {
    // 生成 Cargo 键值对，用于在运行时获取构建信息
    // 这将生成如 CARGO_PKG_VERSION 等环境变量
    generate_cargo_keys();

    // 当 Git HEAD 改变时重新运行构建脚本
    // 这确保版本信息在每次提交后都会更新
    rerun_if_git_head_changed();
}
