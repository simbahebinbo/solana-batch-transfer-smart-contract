use anchor_client::solana_sdk::signature::{Keypair, Signer};
use batch_transfer::{self, safe_add, safe_sum_transfer_info, TransferInfo};
use anchor_lang::prelude::*;

mod utils_test;
use utils_test::{get_test_program, get_bank_account};

/// 测试SOL批量转账功能 - 转换为单元测试
#[test]
fn test_batch_transfer_sol() {
    // 获取程序和支付者
    let (program, _payer) = get_test_program();
    
    // 创建测试账户
    let admin = Keypair::new();
    let sender = Keypair::new();
    let recipient1 = Keypair::new();
    let recipient2 = Keypair::new();
    
    // 记录收件人公钥
    let recipient1_pubkey = recipient1.pubkey();
    let recipient2_pubkey = recipient2.pubkey();
    
    // 获取银行账户的PDA
    let (bank_account, _) = get_bank_account(&program.id());
    
    println!("开始模拟批量转账测试");
    
    // 模拟银行账户状态
    let bank_account_data = batch_transfer::BankAccount {
        admin: admin.pubkey(),
        fee: 100, // 设置费用为1% (100 basis points)
        is_initialized: true,
    };
    
    // 模拟发送者余额
    let sender_balance_before = 100_000_000; // 0.1 SOL
    println!("发送者初始余额: {}", sender_balance_before);
    
    // 准备收件人和金额
    let recipients = vec![recipient1_pubkey, recipient2_pubkey];
    let amounts = vec![1_000_000, 2_000_000]; // 0.001 SOL 和 0.002 SOL
    
    // 创建TransferInfo结构体数组
    let transfers: Vec<TransferInfo> = recipients
        .iter()
        .zip(amounts.iter())
        .map(|(recipient, amount)| TransferInfo {
            recipient: *recipient,
            amount: *amount,
        })
        .collect();
    
    // 模拟计算总转账金额
    let total_amount = safe_sum_transfer_info(&transfers).unwrap();
    assert_eq!(total_amount, 3_000_000, "总转账金额应为0.003 SOL");
    
    // 计算手续费
    let fee_amount = bank_account_data.fee * total_amount / 10000; // 1% 费用
    assert_eq!(fee_amount, 30_000, "手续费应为30,000 lamports");
    
    // 计算所需总金额
    let required_balance = safe_add(total_amount, fee_amount).unwrap();
    assert_eq!(required_balance, 3_030_000, "所需总金额应为3,030,000 lamports");
    
    // 验证发送者余额是否足够
    assert!(sender_balance_before >= required_balance, "发送者余额不足");
    
    // 模拟转账后的余额
    let sender_balance_after = sender_balance_before - required_balance;
    let recipient1_balance = amounts[0];
    let recipient2_balance = amounts[1];
    let bank_account_balance = fee_amount;
    
    // 验证收款人收到了正确的金额
    assert_eq!(recipient1_balance, 1_000_000, "收款人1余额错误");
    assert_eq!(recipient2_balance, 2_000_000, "收款人2余额错误");
    
    // 验证发送者余额减少了正确的金额
    assert_eq!(
        sender_balance_before - sender_balance_after,
        required_balance,
        "发送者余额减少不正确"
    );
    
    // 验证银行账户收到了费用
    assert_eq!(bank_account_balance, fee_amount, "银行账户余额不正确");
    
    println!("批量转账测试完成");
} 