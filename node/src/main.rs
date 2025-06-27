//! # BitNice 节点
//! 
//! 基于 PoW 共识的 BitNice 区块链节点主程序

mod chain_spec;
mod cli;
mod command;
mod rpc;
mod service;

fn main() -> sc_cli::Result<()> {
    command::run()
} 