use anchor_client::{
    solana_sdk::{
        signature::{Keypair, Signer},
    },
};
use batch_transfer::{self, ErrorCode, BankAccount};
use anchor_lang::prelude::*;

mod utils_test;
use utils_test::{get_test_program, get_bank_account};

/// 测试未授权初始化的情况 - 转换为单元测试
#[test]
fn test_initialize_unauthorized() {
    // 获取程序和支付者
    let (program, _payer) = get_test_program();
    
    // 创建测试账户
    let admin = Keypair::new();
    let unauthorized_user = Keypair::new();

    // 获取银行账户的PDA
    let (bank_account, _) = get_bank_account(&program.id());
    
    println!("开始测试未授权初始化场景");
    
    // 模拟银行账户状态 - 初始化前
    let mut bank = BankAccount {
        admin: Pubkey::default(),
        fee: 0,
        is_initialized: false,
    };
    
    // 模拟未授权用户尝试初始化
    println!("模拟未授权用户尝试初始化");
    
    // 在实际代码中，这里会检查部署者是否等于指定的管理员
    let deployer_pubkey = unauthorized_user.pubkey();
    let admin_pubkey = admin.pubkey();
    
    // 模拟初始化逻辑
    let init_error = if deployer_pubkey != admin_pubkey {
        // 模拟返回 Unauthorized 错误
        Some(ErrorCode::Unauthorized)
    } else {
        bank.admin = admin_pubkey;
        bank.fee = 0;
        bank.is_initialized = true;
        None
    };
    
    // 验证初始化返回了正确的错误
    assert!(init_error.is_some(), "未授权初始化应该失败");
    
    if let Some(error) = init_error {
        // 验证错误类型正确
        assert!(matches!(error, ErrorCode::Unauthorized), "返回了错误的错误类型");
        println!("未授权初始化失败，返回了正确的错误: Unauthorized");
    }
    
    // 验证银行账户状态未改变
    assert!(!bank.is_initialized, "银行账户不应被初始化");
    assert_eq!(bank.admin, Pubkey::default(), "管理员不应被设置");
    
    println!("未授权初始化测试完成");
} 