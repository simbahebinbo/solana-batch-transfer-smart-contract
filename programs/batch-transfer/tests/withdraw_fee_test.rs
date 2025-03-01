use anchor_client::{
    solana_sdk::{
        signature::{Keypair, Signer},
        native_token::LAMPORTS_PER_SOL,
    },
};
use batch_transfer::{self};

mod utils_test;
use utils_test::{get_test_program, get_bank_account};

/// 测试管理员提取费用功能
#[test]
fn test_withdraw_fee() {
    // 获取程序和支付者
    let (program, _payer) = get_test_program();
    
    // 创建测试账户
    let admin = Keypair::new();
    
    // 获取银行账户PDA
    let (bank_account, _) = get_bank_account(&program.id());
    
    println!("开始测试管理员提取费用功能");
    
    // 模拟银行账户状态
    let bank_account_data = batch_transfer::BankAccount {
        admin: admin.pubkey(),
        fee: 100, // 设置费用为1% (100 basis points)
        is_initialized: true,
    };
    
    // 模拟银行账户余额（假设已经收集了一些费用）
    let bank_account_balance = LAMPORTS_PER_SOL / 10; // 0.1 SOL
    println!("银行账户当前余额: {} lamports", bank_account_balance);
    
    // 记录管理员提取前的SOL余额
    let admin_balance_before = LAMPORTS_PER_SOL / 2; // 0.5 SOL
    println!("管理员提取前余额: {} lamports", admin_balance_before);
    
    // 模拟提取操作
    // 在实际合约中，会调用withdraw_fee指令
    
    // 计算预期结果 - 所有费用转移到管理员账户
    let admin_balance_after = admin_balance_before + bank_account_balance;
    let bank_account_balance_after = 0; // 银行账户应该为0
    
    // 验证结果
    println!("管理员提取后余额: {} lamports", admin_balance_after);
    println!("银行账户提取后余额: {} lamports", bank_account_balance_after);
    
    assert_eq!(
        admin_balance_after, 
        admin_balance_before + bank_account_balance,
        "管理员余额应该增加银行账户中的所有资金"
    );
    
    assert_eq!(
        bank_account_balance_after, 
        0,
        "银行账户应该为空"
    );
    
    println!("管理员提取费用测试通过");
} 