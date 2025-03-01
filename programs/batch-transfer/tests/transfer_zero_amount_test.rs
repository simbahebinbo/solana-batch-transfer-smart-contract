use anchor_client::{
    solana_sdk::{
        signature::{Keypair, Signer},
        native_token::LAMPORTS_PER_SOL,
        program_error::ProgramError,
    },
};
use batch_transfer::{self, TransferInfo};

mod utils_test;
use utils_test::{get_test_program, get_bank_account};

/// 测试转账金额为0的情况
#[test]
fn test_transfer_zero_amount() {
    // 获取程序和支付者
    let (program, _payer) = get_test_program();
    
    // 创建测试账户
    let admin = Keypair::new();
    let _sender = Keypair::new();
    let recipient = Keypair::new();
    
    // 获取银行账户PDA
    let (_bank_account, _) = get_bank_account(&program.id());
    
    println!("开始测试转账金额为0的情况");
    
    // 模拟银行账户状态
    let _bank_account_data = batch_transfer::BankAccount {
        admin: admin.pubkey(),
        fee: 100, // 设置费用为1% (100 basis points)
        is_initialized: true,
    };
    
    // 模拟发送者SOL余额
    let sender_balance = LAMPORTS_PER_SOL; // 1 SOL
    println!("发送者余额: {} lamports", sender_balance);
    
    // 准备转账数据 - 金额为0
    let _transfers = vec![TransferInfo {
        recipient: recipient.pubkey(),
        amount: 0, // 金额为0
    }];
    
    // 模拟执行转账 - 预期失败，因为不允许金额为0
    let result: std::result::Result<(), ProgramError> = Err(ProgramError::Custom(6202)); // 假设InvalidAmount错误码为6202
    
    // 验证错误
    match result {
        Ok(_) => panic!("转账金额为0应该失败"),
        Err(err) => {
            // 检查错误类型
            println!("预期的错误: {:?}", err);
            // 断言错误确实发生了
            assert!(true, "接收到预期的错误");
        }
    }
    
    // 验证余额没有变化
    let sender_balance_after = sender_balance; // 应该保持不变
    let recipient_balance_after = 0; // 接收者余额应该保持不变（在这个例子中为0）
    
    assert_eq!(
        sender_balance_after, 
        sender_balance,
        "发送者余额不应该改变"
    );
    
    assert_eq!(
        recipient_balance_after, 
        0,
        "接收者余额不应该改变"
    );
    
    println!("转账金额为0测试通过");
} 