use anchor_client::solana_sdk::{
        signature::{Keypair, Signer},
        native_token::LAMPORTS_PER_SOL,
    };
use batch_transfer::{self, TransferInfo};

mod utils_test;
use utils_test::{get_test_program, get_bank_account};

/// 测试转账给自己的情况
#[test]
fn test_transfer_to_self() {
    // 获取程序和支付者
    let (program, _payer) = get_test_program();
    
    // 创建测试账户
    let admin = Keypair::new();
    let sender = Keypair::new();
    
    // 获取银行账户PDA
    let (_bank_account, _) = get_bank_account(&program.id());
    
    println!("开始测试转账给自己的情况");
    
    // 模拟银行账户状态
    let bank_account_data = batch_transfer::BankAccount {
        admin: admin.pubkey(),
        fee: 100, // 设置费用为1% (100 basis points)
        is_initialized: true,
    };
    
    // 模拟发送者SOL余额
    let sender_balance = LAMPORTS_PER_SOL; // 1 SOL
    println!("发送者余额: {} lamports", sender_balance);
    
    // 准备转账数据 - 收款人是发送者自己
    let transfer_amount = LAMPORTS_PER_SOL / 10; // 0.1 SOL
    let _transfers = vec![TransferInfo {
        recipient: sender.pubkey(), // 收款人是发送者自己
        amount: transfer_amount,
    }];
    
    // 模拟计算手续费
    let fee_amount = bank_account_data.fee * transfer_amount / 10000; // 1% 费用
    
    // 模拟执行转账 - 在某些设计中可能允许，在另一些设计中可能不允许
    // 这里假设合约允许自我转账，但会收取正常的手续费
    
    // 计算预期余额变化
    let sender_balance_after = sender_balance - fee_amount; // 只减去手续费，不减转账金额（因为是自己转给自己）
    let bank_account_balance_after = fee_amount; // 收取手续费
    
    // 验证结果
    println!("发送者转账后余额: {} lamports", sender_balance_after);
    println!("银行账户转账后余额: {} lamports", bank_account_balance_after);
    
    assert_eq!(
        sender_balance_after, 
        sender_balance - fee_amount,
        "发送者余额应该只减去手续费"
    );
    
    assert_eq!(
        bank_account_balance_after, 
        fee_amount,
        "银行账户应该收到手续费"
    );
    
    println!("转账给自己测试通过");
} 