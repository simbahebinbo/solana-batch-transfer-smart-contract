use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::system_program;
use solana_program_test::ProgramTest;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use anchor_lang::{InstructionData, ToAccountMetas};
use solana_sdk::account::Account;


/// 测试未授权初始化的情况
#[tokio::test]
async fn test_initialize_unauthorized() {
    let program_id = batch_transfer::ID;
    let mut pt = ProgramTest::new("batch_transfer", program_id, None);
    pt.set_compute_max_units(1200_000);

    let admin = Keypair::new();
    let unauthorized_user = Keypair::new();

    // 计算bank_account的PDA
    let (bank_account, _bump) = Pubkey::find_program_address(
        &[b"bank_account"],
        &program_id,
    );

    // 为管理员和未授权用户添加初始余额
    pt.add_account(
        admin.pubkey(),
        Account {
            lamports: 100 * LAMPORTS_PER_SOL,
            ..Account::default()
        },
    );
    pt.add_account(
        unauthorized_user.pubkey(),
        Account {
            lamports: 100 * LAMPORTS_PER_SOL,
            ..Account::default()
        },
    );

    let (mut banks_client, _payer, recent_blockhash) = pt.start().await;

    // 尝试使用未授权用户初始化银行账户
    let initialize_ix = Instruction {
        program_id: batch_transfer::ID,
        accounts: batch_transfer::accounts::Initialize {
            bank_account,
            deployer: unauthorized_user.pubkey(),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: batch_transfer::instruction::Initialize {
            admin: admin.pubkey(), // 尝试设置其他人为管理员
        }
        .data(),
    };

    let initialize_tx = Transaction::new_signed_with_payer(
        &[initialize_ix],
        Some(&unauthorized_user.pubkey()),
        &[&unauthorized_user],
        recent_blockhash,
    );

    let result = banks_client.process_transaction(initialize_tx).await;
    assert!(result.is_err());

    // 验证bank_account未被创建
    let bank_account_data = banks_client.get_account(bank_account).await.unwrap();
    assert!(bank_account_data.is_none());
} 