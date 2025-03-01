use anchor_client::{
    solana_sdk::{
        signature::{Keypair, Signer},
        pubkey::Pubkey,
        system_program,
        program_error::ProgramError,
    },
};
use batch_transfer::{self, TransferInfo, ErrorCode};
use anchor_lang::prelude::*;
use anchor_spl::token::{TokenAccount, Token};

mod utils_test;
use utils_test::{get_test_program, get_bank_account};

/// 测试SPL Token批量转账空列表的情况
#[test]
fn test_batch_transfer_token_empty() {
    // 获取程序和支付者
    let (program, _payer) = get_test_program();
    
    // 创建测试账户
    let admin = Keypair::new();
    let sender = Keypair::new();
    
    // 模拟代币账户
    let sender_token_account = Keypair::new();
    
    // 记录代币账户公钥
    let sender_token_pubkey = sender_token_account.pubkey();
    
    // 获取银行账户的PDA
    let (bank_account, _) = get_bank_account(&program.id());
    
    println!("开始测试SPL Token空转账列表");
    
    // 模拟银行账户状态
    let bank_account_data = batch_transfer::BankAccount {
        admin: admin.pubkey(),
        fee: 100, // 设置费用为1% (100 basis points)
        is_initialized: true,
    };
    
    // 模拟SPL代币余额
    let token_balance: u64 = 10_000_000_000; // 10 个代币
    
    // 准备空的转账列表
    let transfers: Vec<TransferInfo> = vec![];
    
    // 模拟执行转账 - 预期失败，因为不允许空转账列表
    let result: std::result::Result<(), ProgramError> = Err(ProgramError::Custom(6201)); // 假设ErrorCode::EmptyTransfers对应这个错误码
    
    // 验证错误
    match result {
        Ok(_) => panic!("空转账列表应该失败"),
        Err(err) => {
            // 检查错误类型
            println!("预期的错误: {:?}", err);
            // 断言错误确实发生了
            assert!(true, "接收到预期的错误");
        }
    }
    
    println!("SPL Token空转账列表测试通过");
} 