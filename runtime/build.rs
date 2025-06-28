use substrate_wasm_builder::WasmBuilder;

fn main() {
    // 使用 WasmBuilder 构建 WebAssembly 运行时
    // 这将生成用于链上执行的 WASM 二进制文件
    WasmBuilder::new()
        // 设置目标目录
        .with_current_project()
        // 导出堆分配器，用于 WASM 运行时的内存管理
        .export_heap_base()
        // 导入内存，允许主机访问 WASM 内存
        .import_memory()
        // 构建项目
        .build();
}
