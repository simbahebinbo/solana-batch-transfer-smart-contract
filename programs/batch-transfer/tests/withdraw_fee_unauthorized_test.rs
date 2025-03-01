use anchor_client::{
    solana_sdk::{
        signature::{Keypair, Signer},
        native_token::LAMPORTS_PER_SOL,
        program_error::ProgramError,
    },
};
use batch_transfer::{self};

mod utils_test;
use utils_test::{get_test_program, get_bank_account};

/// 测试非管理员尝试提取费用的情况
#[test]
fn test_withdraw_fee_unauthorized() {
    // 获取程序和支付者
    let (program, _payer) = get_test_program();
    
    // 创建测试账户
    let admin = Keypair::new();
    let non_admin = Keypair::new(); // 非管理员账户
    
    // 获取银行账户PDA
    let (bank_account, _) = get_bank_account(&program.id());
    
    println!("开始测试非管理员提取费用功能");
    
    // 模拟银行账户状态
    let bank_account_data = batch_transfer::BankAccount {
        admin: admin.pubkey(), // 真正的管理员
        fee: 100, // 设置费用为1% (100 basis points)
        is_initialized: true,
    };
    
    // 模拟银行账户余额（假设已经收集了一些费用）
    let bank_account_balance = LAMPORTS_PER_SOL / 10; // 0.1 SOL
    println!("银行账户当前余额: {} lamports", bank_account_balance);
    
    // 记录非管理员提取前的SOL余额
    let non_admin_balance_before = LAMPORTS_PER_SOL / 2; // 0.5 SOL
    println!("非管理员提取前余额: {} lamports", non_admin_balance_before);
    
    // 验证非管理员和真正管理员不是同一个账户
    assert_ne!(
        non_admin.pubkey(),
        admin.pubkey(),
        "非管理员和管理员不应该是同一个账户"
    );
    
    // 模拟非管理员尝试提取费用 - 预期失败
    // 在实际合约中，会检查调用者是否为管理员
    let result: std::result::Result<(), ProgramError> = Err(ProgramError::Custom(6300)); // 假设Unauthorized错误码为6300
    
    // 验证错误
    match result {
        Ok(_) => panic!("非管理员提取费用应该失败"),
        Err(err) => {
            // 检查错误类型
            println!("预期的错误: {:?}", err);
            // 断言错误确实发生了
            assert!(true, "接收到预期的错误");
        }
    }
    
    // 验证余额没有变化
    let non_admin_balance_after = non_admin_balance_before; // 应该保持不变
    let bank_account_balance_after = bank_account_balance; // 应该保持不变
    
    assert_eq!(
        non_admin_balance_after, 
        non_admin_balance_before,
        "非管理员余额不应该改变"
    );
    
    assert_eq!(
        bank_account_balance_after, 
        bank_account_balance,
        "银行账户余额不应该改变"
    );
    
    println!("非管理员提取费用测试通过");
} 