use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::system_program;
use solana_program_test::ProgramTest;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use anchor_lang::{InstructionData, ToAccountMetas, AccountDeserialize};
use solana_sdk::account::Account;


/// 测试未授权设置手续费的情况
#[tokio::test]
async fn test_set_fee_unauthorized() {
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

    // 初始化银行账户
    let initialize_ix = Instruction {
        program_id: batch_transfer::ID,
        accounts: batch_transfer::accounts::Initialize {
            bank_account,
            deployer: admin.pubkey(),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: batch_transfer::instruction::Initialize {
            admin: admin.pubkey(),
        }
        .data(),
    };

    let initialize_tx = Transaction::new_signed_with_payer(
        &[initialize_ix],
        Some(&admin.pubkey()),
        &[&admin],
        recent_blockhash,
    );

    banks_client.process_transaction(initialize_tx).await.unwrap();

    // 尝试使用未授权用户设置手续费
    let fee = LAMPORTS_PER_SOL / 100; // 0.01 SOL
    let set_fee_ix = Instruction {
        program_id: batch_transfer::ID,
        accounts: batch_transfer::accounts::SetFee {
            bank_account,
            admin: unauthorized_user.pubkey(),
        }
        .to_account_metas(None),
        data: batch_transfer::instruction::SetFee {
            fee,
        }
        .data(),
    };

    let set_fee_tx = Transaction::new_signed_with_payer(
        &[set_fee_ix],
        Some(&unauthorized_user.pubkey()),
        &[&unauthorized_user],
        recent_blockhash,
    );

    let result = banks_client.process_transaction(set_fee_tx).await;
    assert!(result.is_err());

    // 验证手续费未被修改
    let bank_account_data = banks_client
        .get_account(bank_account)
        .await
        .unwrap()
        .unwrap();
    let bank = batch_transfer::BankAccount::try_deserialize(&mut bank_account_data.data.as_ref()).unwrap();
    assert_eq!(bank.fee, 0); // 初始手续费为0
} 