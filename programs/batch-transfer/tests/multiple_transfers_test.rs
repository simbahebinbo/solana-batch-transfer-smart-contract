use anchor_client::{
    solana_sdk::{
        signature::{Keypair, Signer},
        native_token::LAMPORTS_PER_SOL,
    },
};
use batch_transfer::{self, TransferInfo};

mod utils_test;
use utils_test::{get_test_program, get_bank_account};

/// 测试连续执行多次批量转账
#[test]
fn test_multiple_transfers() {
    // 获取程序和支付者
    let (program, _payer) = get_test_program();
    
    // 创建测试账户
    let admin = Keypair::new();
    let sender = Keypair::new();
    let recipient1 = Keypair::new();
    let recipient2 = Keypair::new();
    let recipient3 = Keypair::new();
    
    // 获取银行账户PDA
    let (bank_account, _) = get_bank_account(&program.id());
    
    println!("开始测试连续执行多次批量转账");
    
    // 模拟银行账户状态
    let bank_account_data = batch_transfer::BankAccount {
        admin: admin.pubkey(),
        fee: 100, // 设置费用为1% (100 basis points)
        is_initialized: true,
    };
    
    // 模拟发送者初始SOL余额
    let initial_sender_balance = LAMPORTS_PER_SOL * 10; // 10 SOL
    let mut current_sender_balance = initial_sender_balance;
    println!("发送者初始余额: {} lamports", current_sender_balance);
    
    // 模拟接收者初始余额
    let mut recipient1_balance = 0;
    let mut recipient2_balance = 0;
    let mut recipient3_balance = 0;
    let mut bank_balance = 0;
    
    // 第一次批量转账
    println!("执行第一次批量转账");
    let first_transfers = vec![
        TransferInfo {
            recipient: recipient1.pubkey(),
            amount: LAMPORTS_PER_SOL, // 1 SOL
        },
        TransferInfo {
            recipient: recipient2.pubkey(),
            amount: LAMPORTS_PER_SOL / 2, // 0.5 SOL
        },
    ];
    
    // 计算第一次转账总金额
    let first_total_amount = first_transfers.iter().fold(0, |acc, t| acc + t.amount);
    println!("第一次转账总金额: {} lamports", first_total_amount);
    
    // 计算第一次手续费
    let first_fee = bank_account_data.fee * first_total_amount / 10000; // 1% 费用
    println!("第一次转账手续费: {} lamports", first_fee);
    
    // 更新余额
    current_sender_balance -= (first_total_amount + first_fee);
    recipient1_balance += LAMPORTS_PER_SOL;
    recipient2_balance += LAMPORTS_PER_SOL / 2;
    bank_balance += first_fee;
    
    println!("第一次转账后发送者余额: {} lamports", current_sender_balance);
    println!("第一次转账后接收者1余额: {} lamports", recipient1_balance);
    println!("第一次转账后接收者2余额: {} lamports", recipient2_balance);
    println!("第一次转账后银行余额: {} lamports", bank_balance);
    
    // 第二次批量转账
    println!("执行第二次批量转账");
    let second_transfers = vec![
        TransferInfo {
            recipient: recipient2.pubkey(),
            amount: LAMPORTS_PER_SOL / 4, // 0.25 SOL
        },
        TransferInfo {
            recipient: recipient3.pubkey(),
            amount: LAMPORTS_PER_SOL * 2, // 2 SOL
        },
    ];
    
    // 计算第二次转账总金额
    let second_total_amount = second_transfers.iter().fold(0, |acc, t| acc + t.amount);
    println!("第二次转账总金额: {} lamports", second_total_amount);
    
    // 计算第二次手续费
    let second_fee = bank_account_data.fee * second_total_amount / 10000; // 1% 费用
    println!("第二次转账手续费: {} lamports", second_fee);
    
    // 更新余额
    current_sender_balance -= (second_total_amount + second_fee);
    recipient2_balance += LAMPORTS_PER_SOL / 4;
    recipient3_balance += LAMPORTS_PER_SOL * 2;
    bank_balance += second_fee;
    
    println!("第二次转账后发送者余额: {} lamports", current_sender_balance);
    println!("第二次转账后接收者1余额: {} lamports", recipient1_balance);
    println!("第二次转账后接收者2余额: {} lamports", recipient2_balance);
    println!("第二次转账后接收者3余额: {} lamports", recipient3_balance);
    println!("第二次转账后银行余额: {} lamports", bank_balance);
    
    // 验证最终余额
    let expected_sender_balance = initial_sender_balance - (first_total_amount + first_fee) - (second_total_amount + second_fee);
    let expected_recipient1_balance = LAMPORTS_PER_SOL;
    let expected_recipient2_balance = LAMPORTS_PER_SOL / 2 + LAMPORTS_PER_SOL / 4;
    let expected_recipient3_balance = LAMPORTS_PER_SOL * 2;
    let expected_bank_balance = first_fee + second_fee;
    
    assert_eq!(
        current_sender_balance,
        expected_sender_balance,
        "发送者最终余额不正确"
    );
    
    assert_eq!(
        recipient1_balance,
        expected_recipient1_balance,
        "接收者1最终余额不正确"
    );
    
    assert_eq!(
        recipient2_balance,
        expected_recipient2_balance,
        "接收者2最终余额不正确"
    );
    
    assert_eq!(
        recipient3_balance,
        expected_recipient3_balance,
        "接收者3最终余额不正确"
    );
    
    assert_eq!(
        bank_balance,
        expected_bank_balance,
        "银行最终余额不正确"
    );
    
    println!("连续执行多次批量转账测试通过");
} 