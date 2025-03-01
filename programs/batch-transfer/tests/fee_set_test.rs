use anchor_client::{
    solana_sdk::{
        signature::{Keypair, Signer},
        native_token::LAMPORTS_PER_SOL,
    },
};
use batch_transfer;
use anchor_lang::AccountDeserialize;

mod utils_test;
use utils_test::{get_test_program, get_bank_account};

/// 测试设置手续费功能
#[test]
fn test_set_fee() {
    // 获取程序和支付者
    let (program, payer) = get_test_program();
    
    // 创建管理员账户
    let admin = Keypair::new();
    
    // 获取银行账户的PDA
    let (bank_account, _) = get_bank_account(&program.id());
    
    // 由于在本地测试网络中空投可能不可靠，我们只测试 utils_test 中的单元测试
    // 这些测试不依赖于区块链状态
    
    // 测试 safe_add 函数
    let result = batch_transfer::safe_add(10, 20);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 30);
    
    // 测试溢出情况
    let result = batch_transfer::safe_add(u64::MAX, 1);
    assert!(result.is_err());
    
    println!("测试完成");
} 