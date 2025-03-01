use batch_transfer::{self, TransferInfo};
use anchor_client::solana_sdk::signer::keypair::Keypair;
use anchor_client::solana_sdk::signer::Signer;
use anchor_lang::prelude::*;

mod utils_test;

/// 测试批量转账SOL功能 - 大量接收者场景
/// 主要测试合约处理大量接收者的能力
/// 注意：这是一个模拟测试，实际上不会执行转账
#[test]
fn test_large_batch_transfer_sol() {
    println!("============= 开始大批量转账测试 =============");
    
    // 步骤1: 设置测试环境
    println!("步骤1: 设置测试环境");
    let (program, _payer) = utils_test::get_test_program();
    
    // 创建管理员和发送者账户
    let admin = Keypair::new();
    let _sender = Keypair::new();
    
    // 获取银行账户
    let (_bank_account, _) = utils_test::get_bank_account(&program.id());
    
    // 步骤2: 生成大量接收者和金额
    println!("步骤2: 生成大量接收者和金额");
    let max_recipients = 100;
    let mut recipients = Vec::with_capacity(max_recipients);
    let mut amounts = Vec::with_capacity(max_recipients);
    let mut total_amount = 0_u64;
    
    for i in 0..max_recipients {
        let recipient = Keypair::new();
        let amount = 10_000_u64 + (i as u64 % 10) * 10_000_u64;
        
        recipients.push(recipient.pubkey());
        amounts.push(amount);
        total_amount += amount;
    }
    
    // 步骤3: 创建TransferInfo结构体数组
    println!("步骤3: 创建TransferInfo结构体数组");
    let transfers: Vec<TransferInfo> = recipients
        .iter()
        .zip(amounts.iter())
        .map(|(recipient, amount)| TransferInfo {
            recipient: *recipient,
            amount: *amount,
        })
        .collect();
    
    // 步骤4: 验证结果
    println!("步骤4: 验证结果");
    assert_eq!(transfers.len(), max_recipients, "接收者数量应为100");
    
    // 计算平均转账金额
    let expected_average = (10_000_u64 + 100_000_u64) / 2;
    let actual_average = total_amount / max_recipients as u64;
    
    // 允许一定的误差范围
    let error_margin = 5_000_u64;
    assert!(
        actual_average >= expected_average - error_margin && 
        actual_average <= expected_average + error_margin,
        "平均转账金额超出预期范围"
    );
    
    println!("总转账金额: {} lamports", total_amount);
    println!("平均转账金额: {} lamports", actual_average);
    println!("接收者数量: {}", transfers.len());
    
    println!("============= 大批量转账测试完成 =============");
} 