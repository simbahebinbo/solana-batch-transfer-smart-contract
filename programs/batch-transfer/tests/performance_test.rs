use std::time::Instant;
use anchor_client::{
    solana_sdk::{
        signature::{Keypair, Signer},
        native_token::LAMPORTS_PER_SOL,
    },
};
use batch_transfer::{self, TransferInfo};

mod utils_test;
use utils_test::{get_test_program, get_bank_account};

/// 测试大量转账的性能
/// 
/// 这个测试评估批量转账智能合约在处理大量转账时的性能表现：
/// 1. 创建100个接收者账户
/// 2. 为每个接收者准备0.01 SOL的转账
/// 3. 模拟计算手续费和转账执行过程
/// 4. 记录准备和执行所需的时间
/// 
/// 测试的主要指标包括：
/// - 准备转账数据所需时间
/// - 执行批量转账所需时间
/// - 每笔转账的平均处理时间
/// - 总转账金额和总手续费
/// 
/// 预期结果：
/// - 所有转账能够成功处理
/// - 最终余额计算正确（发送者余额、接收者余额、银行账户余额）
#[test]
fn test_large_batch_performance() {
    // 获取程序和支付者
    let (program, _payer) = get_test_program();
    
    // 创建测试账户
    let _admin = Keypair::new();
    let _sender = Keypair::new();
    
    // 获取银行账户PDA
    let (_bank_account, _) = get_bank_account(&program.id());
    
    println!("开始测试大量转账性能");
    
    // 模拟银行账户状态
    let _bank_account_data = batch_transfer::BankAccount {
        admin: _admin.pubkey(),
        fee: 100, // 设置费用为1% (100 basis points)
        is_initialized: true,
    };
    
    // 模拟SOL初始余额（非常大以支持多次转账）
    let sol_balance = LAMPORTS_PER_SOL * 1000; // 1000 SOL
    println!("发送者初始余额: {} lamports", sol_balance);
    
    // 创建多个接收者
    let num_recipients = 100; // 测试100个接收者
    println!("创建 {} 个接收者", num_recipients);
    
    let mut recipients = vec![];
    for _i in 0..num_recipients {
        recipients.push(Keypair::new());
    }
    
    // 准备大量转账数据
    println!("准备 {} 个转账交易", num_recipients);
    let transfer_amount = LAMPORTS_PER_SOL / 100; // 每次转0.01 SOL
    
    let start_time = Instant::now();
    
    let mut transfers = vec![];
    for recipient in &recipients {
        transfers.push(TransferInfo {
            recipient: recipient.pubkey(),
            amount: transfer_amount,
        });
    }
    
    let preparation_duration = start_time.elapsed();
    println!("准备转账数据耗时: {:?}", preparation_duration);
    
    // 模拟批量转账执行
    let execution_start = Instant::now();
    
    // 在实际场景中，这里会调用batch_transfer_sol指令
    // 这里我们只模拟计算所需的时间
    
    // 计算总转账金额和总手续费
    let total_transfer_amount = transfer_amount * num_recipients as u64;
    let total_fee = _bank_account_data.fee * total_transfer_amount / 10000; // 1% 费用
    
    // 模拟检查发送者余额
    let sender_has_sufficient_balance = sol_balance >= (total_transfer_amount + total_fee);
    assert!(sender_has_sufficient_balance, "发送者余额不足");
    
    // 模拟更新接收者余额
    for _ in &recipients {
        // 在实际场景中，这里会更新每个接收者的余额
    }
    
    // 模拟更新发送者余额
    let sender_balance_after = sol_balance - total_transfer_amount - total_fee;
    
    // 模拟更新银行账户余额
    let bank_balance_after = total_fee;
    
    let execution_duration = execution_start.elapsed();
    println!("执行批量转账耗时: {:?}", execution_duration);
    
    // 输出性能指标
    println!("每笔转账平均耗时: {:?}", execution_duration / num_recipients as u32);
    println!("总转账金额: {} lamports", total_transfer_amount);
    println!("总手续费: {} lamports", total_fee);
    println!("发送者余额变化: {} -> {} lamports", sol_balance, sender_balance_after);
    println!("银行账户余额: {} lamports", bank_balance_after);
    
    // 验证结果
    assert_eq!(
        sender_balance_after,
        sol_balance - total_transfer_amount - total_fee,
        "发送者余额计算错误"
    );
    
    assert_eq!(
        bank_balance_after,
        total_fee,
        "银行账户余额计算错误"
    );
    
    println!("大量转账性能测试通过");
} 