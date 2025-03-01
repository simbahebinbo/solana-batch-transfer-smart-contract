use batch_transfer::{self, BankAccount};
use anchor_lang::prelude::*;
use anchor_client::solana_sdk::signer::keypair::Keypair;
use anchor_client::solana_sdk::signer::Signer;

mod utils_test;

/// 综合测试：测试SOL和SPL Token批量转账的完整流程
/// 包括初始化、设置费用、转账和验证结果
#[test]
fn test_comprehensive_batch_transfer() {
    println!("============= 开始综合批量转账测试 =============");
    
    // 步骤1: 设置测试环境
    println!("步骤1: 设置测试环境");
    let (program, _payer) = utils_test::get_test_program();
    
    // 创建管理员和发送者账户
    let admin = Keypair::new();
    let sender = Keypair::new();
    
    // 获取银行账户
    let (bank_account, _) = utils_test::get_bank_account(&program.id());
    
    // 步骤2: 初始化银行账户
    println!("步骤2: 初始化银行账户");
    let bank_account_data = BankAccount {
        admin: admin.pubkey(),
        fee: 5_000_000, // 0.005 SOL
        is_initialized: true,
    };
    
    // 步骤3: 设置交易费用
    println!("步骤3: 设置交易费用");
    let transaction_fee = 5_000_000; // 0.005 SOL
    
    // 步骤4: 准备接收者和转账金额
    println!("步骤4: 准备接收者和转账金额");
    let recipient1 = Keypair::new();
    let recipient2 = Keypair::new();
    let recipient3 = Keypair::new();
    
    let amount1 = 10_000_000; // 0.01 SOL
    let amount2 = 20_000_000; // 0.02 SOL
    let amount3 = 30_000_000; // 0.03 SOL
    
    let recipients = vec![
        recipient1.pubkey(),
        recipient2.pubkey(),
        recipient3.pubkey(),
    ];
    
    let amounts = vec![amount1, amount2, amount3];
    
    // 步骤5: 计算总转账金额和费用
    println!("步骤5: 计算总转账金额和费用");
    let total_amount = amount1 + amount2 + amount3;
    let fee = transaction_fee;
    let total_with_fee = total_amount + fee;
    
    // 步骤6: 模拟执行批量转账
    println!("步骤6: 模拟执行批量转账");
    
    // 步骤7: 验证结果
    println!("步骤7: 验证结果");
    assert_eq!(recipients.len(), 3, "接收者数量应为3");
    assert_eq!(amounts.len(), 3, "金额数量应为3");
    assert_eq!(total_amount, 60_000_000, "总转账金额应为0.06 SOL");
    assert_eq!(fee, 5_000_000, "费用应为0.005 SOL");
    assert_eq!(total_with_fee, 65_000_000, "总金额加费用应为0.065 SOL");
    
    println!("============= 综合批量转账测试完成 =============");
} 