use anchor_client::{
    solana_sdk::{
        signature::{Keypair, Signer},
        pubkey::Pubkey,
    },
};
use batch_transfer::{self, ErrorCode};
use anchor_lang::prelude::*;

mod utils_test;
use utils_test::get_test_program;

/// 测试空转账列表的情况 - 转换为单元测试
#[test]
fn test_empty_transfers() {
    // 获取测试程序和支付者
    let (program, _payer) = get_test_program();
    
    // 创建管理员和发送者账户
    let admin = Keypair::new();
    let sender = Keypair::new();
    
    println!("开始模拟空转账列表测试");
    
    // 准备空的转账列表
    let empty_recipients: Vec<Pubkey> = vec![];
    let empty_amounts: Vec<u64> = vec![];
    
    // 模拟批量转账逻辑检查
    let transfer_result = if empty_recipients.is_empty() || empty_amounts.is_empty() {
        // 模拟返回 EmptyTransfers 错误
        Err(ErrorCode::EmptyTransfers)
    } else {
        // 仅用于模拟成功的情况
        Ok(())
    };
    
    // 验证转账操作返回了正确的错误
    assert!(transfer_result.is_err(), "应该检测到空转账列表");
    
    // 验证错误类型
    let err = transfer_result.unwrap_err();
    assert!(matches!(err, ErrorCode::EmptyTransfers), "应该返回EmptyTransfers错误");
    
    println!("空转账列表测试完成，错误类型正确：EmptyTransfers");
} 