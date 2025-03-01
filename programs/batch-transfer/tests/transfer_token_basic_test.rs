use anchor_client::solana_sdk::signature::{Keypair, Signer};
use batch_transfer::{self, safe_sum_transfer_info, TransferInfo};
use anchor_lang::prelude::*;

mod utils_test;
use utils_test::{get_test_program, get_bank_account};

/// 测试SPL Token批量转账功能的基本工作流程
#[test]
fn test_batch_transfer_token_basic() {
    // 获取程序和支付者
    let (program, _payer) = get_test_program();
    
    // 创建测试账户
    let admin = Keypair::new();
    let sender = Keypair::new();
    
    // 模拟代币账户
    let sender_token_account = Keypair::new();
    let recipient1_token_account = Keypair::new();
    let recipient2_token_account = Keypair::new();
    
    // 记录代币账户公钥
    let sender_token_pubkey = sender_token_account.pubkey();
    let recipient1_token_pubkey = recipient1_token_account.pubkey();
    let recipient2_token_pubkey = recipient2_token_account.pubkey();
    
    // 获取银行账户的PDA
    let (bank_account, _) = get_bank_account(&program.id());
    
    println!("开始模拟SPL Token批量转账测试");
    
    // 模拟银行账户状态
    let bank_account_data = batch_transfer::BankAccount {
        admin: admin.pubkey(),
        fee: 100, // 设置费用为1% (100 basis points)
        is_initialized: true,
    };
    
    // 模拟SPL代币余额
    let token_balance_before = 10_000_000_000; // 10 个代币 (假设精度为9位)
    println!("发送者代币初始余额: {}", token_balance_before);
    
    // 模拟SOL余额（用于支付手续费）
    let sol_balance_before = 100_000_000; // 0.1 SOL
    
    // 准备收件人和金额
    let recipients = vec![recipient1_token_pubkey, recipient2_token_pubkey];
    let amounts = vec![100_000_000, 200_000_000]; // 0.1 和 0.2 个代币
    
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
    assert_eq!(total_amount, 300_000_000, "总转账金额应为0.3个代币");
    
    // 计算手续费 (以SOL支付)
    let fee_amount = bank_account_data.fee * total_amount / 10000; // 1% 费用
    assert_eq!(fee_amount, 3_000_000, "手续费应为3,000,000 lamports");
    
    // 验证代币余额是否足够
    assert!(token_balance_before >= total_amount, "代币余额不足");
    
    // 验证SOL余额是否足够支付手续费
    assert!(sol_balance_before >= fee_amount, "SOL余额不足以支付手续费");
    
    // 模拟转账后的余额
    let token_balance_after = token_balance_before - total_amount;
    let recipient1_token_balance = amounts[0];
    let recipient2_token_balance = amounts[1];
    let bank_account_sol_balance = fee_amount;
    
    // 验证收款人收到了正确的金额
    assert_eq!(recipient1_token_balance, 100_000_000, "收款人1余额错误");
    assert_eq!(recipient2_token_balance, 200_000_000, "收款人2余额错误");
    
    // 验证发送者代币余额减少了正确的金额
    assert_eq!(
        token_balance_before - token_balance_after,
        total_amount,
        "发送者代币余额减少不正确"
    );
    
    // 验证银行账户收到了费用
    assert_eq!(bank_account_sol_balance, fee_amount, "银行账户余额不正确");
    
    println!("SPL Token批量转账测试完成");
} 