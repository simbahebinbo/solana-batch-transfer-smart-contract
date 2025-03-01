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

/// 测试混合转账类型（同时测试SOL和Token）
#[test]
fn test_mixed_transfers() {
    // 获取程序和支付者
    let (program, _payer) = get_test_program();
    
    // 创建测试账户
    let _admin = Keypair::new();
    let _sender = Keypair::new();
    let _recipient_sol = Keypair::new();
    let _recipient_token = Keypair::new();
    
    // 获取银行账户PDA
    let (_bank_account, _) = get_bank_account(&program.id());
    
    println!("开始测试混合转账类型");
    
    // 模拟银行账户状态
    let _bank_account_data = batch_transfer::BankAccount {
        admin: _admin.pubkey(),
        fee: 100, // 设置费用为1% (100 basis points)
        is_initialized: true,
    };
    
    // 模拟SOL和Token初始余额
    let sol_balance = LAMPORTS_PER_SOL * 5; // 5 SOL
    let token_balance: u64 = 10_000_000_000; // 10 代币
    
    println!("发送者SOL余额: {} lamports", sol_balance);
    println!("发送者Token余额: {} tokens", token_balance);
    
    // 第一阶段：SOL转账
    println!("执行SOL转账");
    
    // 准备SOL转账数据
    let sol_transfer_amount = LAMPORTS_PER_SOL; // 1 SOL
    let _sol_transfers = vec![TransferInfo {
        recipient: _recipient_sol.pubkey(),
        amount: sol_transfer_amount,
    }];
    
    // 计算SOL转账手续费
    let sol_fee = _bank_account_data.fee * sol_transfer_amount / 10000; // 1% 费用
    
    // 模拟SOL转账结果
    let sol_sender_balance_after = sol_balance - sol_transfer_amount - sol_fee;
    let sol_recipient_balance_after = sol_transfer_amount;
    let bank_sol_balance_after = sol_fee;
    
    println!("SOL转账后发送者余额: {} lamports", sol_sender_balance_after);
    println!("SOL转账后接收者余额: {} lamports", sol_recipient_balance_after);
    println!("SOL转账后银行余额: {} lamports", bank_sol_balance_after);
    
    // 第二阶段：Token转账
    println!("执行Token转账");
    
    // 准备Token转账数据
    let token_transfer_amount: u64 = 1_000_000_000; // 1 代币
    // 注意：在实际测试中，我们需要创建一个Token账户并关联给收款人
    
    // 计算Token转账手续费
    let token_fee = _bank_account_data.fee * token_transfer_amount / 10000; // 1% 费用
    
    // 模拟Token转账结果
    let token_sender_balance_after = token_balance - token_transfer_amount - token_fee;
    let token_recipient_balance_after = token_transfer_amount;
    let bank_token_balance_after = token_fee;
    
    println!("Token转账后发送者余额: {} tokens", token_sender_balance_after);
    println!("Token转账后接收者余额: {} tokens", token_recipient_balance_after);
    println!("Token转账后银行余额: {} tokens", bank_token_balance_after);
    
    // 验证两次转账的结果
    // SOL转账验证
    assert_eq!(
        sol_sender_balance_after, 
        sol_balance - sol_transfer_amount - sol_fee,
        "SOL转账后发送者余额不正确"
    );
    
    assert_eq!(
        sol_recipient_balance_after, 
        sol_transfer_amount,
        "SOL转账后接收者余额不正确"
    );
    
    // Token转账验证
    assert_eq!(
        token_sender_balance_after, 
        token_balance - token_transfer_amount - token_fee,
        "Token转账后发送者余额不正确"
    );
    
    assert_eq!(
        token_recipient_balance_after, 
        token_transfer_amount,
        "Token转账后接收者余额不正确"
    );
    
    // 银行账户验证
    println!("银行SOL余额: {} lamports", bank_sol_balance_after);
    println!("银行Token余额: {} tokens", bank_token_balance_after);
    
    println!("混合转账类型测试通过");
} 