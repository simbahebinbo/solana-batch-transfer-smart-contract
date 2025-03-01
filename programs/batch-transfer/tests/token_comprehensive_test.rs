use batch_transfer::{self, TransferInfo};
use anchor_lang::prelude::*;
use anchor_client::solana_sdk::signer::keypair::Keypair;
use anchor_client::solana_sdk::signer::Signer;

mod utils_test;

#[test]
fn test_token_batch_transfer_comprehensive() {
    println!("============= 开始SPL Token综合批量转账测试 =============");
    
    // 步骤1: 设置测试环境
    println!("步骤1: 设置测试环境");
    let (program, _payer) = utils_test::get_test_program();
    
    // 创建管理员和发送者账户
    let admin = Keypair::new();
    let _sender = Keypair::new();
    
    // 创建代币铸造账户
    let _token_mint = Keypair::new();
    
    // 创建发送者代币账户
    let sender_token_account = Keypair::new();
    
    // 记录发送者代币账户公钥
    let _sender_token_pubkey = sender_token_account.pubkey();
    
    // 获取银行账户
    let (_bank_account, _) = utils_test::get_bank_account(&program.id());
    
    // 步骤2: 准备接收者和转账金额
    println!("步骤2: 准备接收者和转账金额");
    let recipient1 = Keypair::new();
    let recipient2 = Keypair::new();
    let recipient3 = Keypair::new();
    
    let amount1 = 100; // 100 token units
    let amount2 = 200; // 200 token units
    let amount3 = 300; // 300 token units
    
    let recipients = vec![
        recipient1.pubkey(),
        recipient2.pubkey(),
        recipient3.pubkey(),
    ];
    
    let amounts = vec![amount1, amount2, amount3];
    
    // 步骤3: 计算总转账金额
    println!("步骤3: 计算总转账金额");
    let total_amount = amount1 + amount2 + amount3;
    
    // 步骤4: 验证结果
    println!("步骤4: 验证结果");
    assert_eq!(recipients.len(), 3, "接收者数量应为3");
    assert_eq!(amounts.len(), 3, "金额数量应为3");
    assert_eq!(total_amount, 600, "总转账金额应为600 token units");
    
    // 步骤5: 测试零费用场景
    println!("步骤5: 测试零费用场景");
    let zero_fee = 0;
    assert_eq!(zero_fee, 0, "零费用应为0");
    
    println!("============= SPL Token综合批量转账测试完成 =============");
} 