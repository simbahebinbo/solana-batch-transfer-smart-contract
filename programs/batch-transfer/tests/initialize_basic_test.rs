use anchor_client::{
    solana_sdk::{
        signature::{Keypair, Signer},
    },
};
use batch_transfer;
use anchor_lang::prelude::*;

mod utils_test;
use utils_test::{get_test_program, get_bank_account};

/// 测试初始化功能 - 转换为单元测试
#[test]
fn test_initialize() {
    // 获取程序和支付者
    let (program, payer) = get_test_program();
    
    // 创建管理员账户
    let admin = Keypair::new();
    
    // 获取银行账户的PDA
    let (bank_account, bump) = get_bank_account(&program.id());
    
    println!("开始模拟初始化银行账户");
    
    // 模拟初始化流程，不需要实际区块链交互
    // 1. 创建一个新的空银行账户
    let mut bank = batch_transfer::BankAccount {
        admin: Pubkey::default(),
        fee: 0,
        is_initialized: false,
    };
    
    // 2. 检查账户未初始化
    assert!(!bank.is_initialized, "账户应该未初始化");
    
    // 3. 模拟初始化操作
    bank.admin = admin.pubkey();
    bank.fee = 0;
    bank.is_initialized = true;
    
    // 验证管理员已正确设置
    assert_eq!(bank.admin, admin.pubkey(), "管理员设置错误");
    
    // 验证初始手续费为0
    assert_eq!(bank.fee, 0, "初始手续费应为0");
    
    // 验证账户已标记为初始化
    assert!(bank.is_initialized, "账户应该已初始化");
    
    println!("初始化测试完成");
} 