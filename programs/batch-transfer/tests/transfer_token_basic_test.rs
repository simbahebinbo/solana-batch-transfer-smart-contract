use solana_sdk::instruction::{Instruction, AccountMeta};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::system_program;
use solana_program_test::ProgramTest;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use anchor_lang::{InstructionData, ToAccountMetas};
use solana_sdk::account::Account;
use anchor_spl::token::{self};
use spl_token::state::Account as TokenAccount;
use spl_token::state::Mint;
use spl_token::instruction as token_instruction;
use spl_token::solana_program::program_pack::Pack;
use batch_transfer::accounts;
use batch_transfer::TransferInfo;
use solana_program::{
    rent::Rent,
    system_instruction,
    sysvar::SysvarId,
};

/// 将 Solana SDK 的 Pubkey 转换为 SPL Token 的 Pubkey
fn to_token_pubkey(pubkey: &Pubkey) -> spl_token::solana_program::pubkey::Pubkey {
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(pubkey.as_ref());
    spl_token::solana_program::pubkey::Pubkey::new_from_array(bytes)
}

/// 将 SPL Token 的 Instruction 转换为 Solana SDK 的 Instruction
fn to_sdk_instruction(ix: spl_token::solana_program::instruction::Instruction) -> Instruction {
    Instruction {
        program_id: Pubkey::new_from_array(ix.program_id.to_bytes()),
        accounts: ix.accounts.into_iter().map(|meta| AccountMeta {
            pubkey: Pubkey::new_from_array(meta.pubkey.to_bytes()),
            is_signer: meta.is_signer,
            is_writable: meta.is_writable,
        }).collect(),
        data: ix.data,
    }
}

/// 初始化代币铸币账户
async fn initialize_token_mint(
    banks_client: &mut solana_program_test::BanksClient,
    mint: &Keypair,
    authority: &Pubkey,
    decimals: u8,
    recent_blockhash: solana_sdk::hash::Hash,
) {
    let rent = banks_client.get_rent().await.unwrap();
    let mint_rent = rent.minimum_balance(Mint::LEN);

    let space = Mint::LEN;
    let create_mint_account_ix = solana_sdk::system_instruction::create_account(
        authority,
        &mint.pubkey(),
        mint_rent,
        space as u64,
        &token::ID,
    );

    let initialize_mint_ix = token_instruction::initialize_mint(
        &to_token_pubkey(&token::ID),
        &to_token_pubkey(&mint.pubkey()),
        &to_token_pubkey(authority),
        Some(&to_token_pubkey(authority)),
        decimals,
    )
    .unwrap();

    let transaction = Transaction::new_signed_with_payer(
        &[create_mint_account_ix, to_sdk_instruction(initialize_mint_ix)],
        Some(authority),
        &[mint],
        recent_blockhash,
    );

    banks_client.process_transaction(transaction).await.unwrap();
}

/// 创建代币账户
async fn create_token_account(
    banks_client: &mut solana_program_test::BanksClient,
    account: &Keypair,
    mint: &Pubkey,
    owner: &Keypair,
    recent_blockhash: solana_sdk::hash::Hash,
) {
    let rent = banks_client.get_rent().await.unwrap();
    let account_rent = rent.minimum_balance(TokenAccount::LEN);

    let space = TokenAccount::LEN;
    let create_account_ix = solana_sdk::system_instruction::create_account(
        &owner.pubkey(),
        &account.pubkey(),
        account_rent,
        space as u64,
        &token::ID,
    );

    let initialize_account_ix = token_instruction::initialize_account(
        &to_token_pubkey(&token::ID),
        &to_token_pubkey(&account.pubkey()),
        &to_token_pubkey(mint),
        &to_token_pubkey(&owner.pubkey()),
    )
    .unwrap();

    let transaction = Transaction::new_signed_with_payer(
        &[create_account_ix, to_sdk_instruction(initialize_account_ix)],
        Some(&owner.pubkey()),
        &[owner, account],
        recent_blockhash,
    );

    banks_client.process_transaction(transaction).await.unwrap();
}

/// 铸造代币
async fn mint_token(
    banks_client: &mut solana_program_test::BanksClient,
    mint: &Pubkey,
    account: &Pubkey,
    authority: &Keypair,
    amount: u64,
    recent_blockhash: solana_sdk::hash::Hash,
) {
    let mint_ix = token_instruction::mint_to(
        &to_token_pubkey(&token::ID),
        &to_token_pubkey(mint),
        &to_token_pubkey(account),
        &to_token_pubkey(&authority.pubkey()),
        &[],
        amount,
    )
    .unwrap();

    let transaction = Transaction::new_signed_with_payer(
        &[to_sdk_instruction(mint_ix)],
        Some(&authority.pubkey()),
        &[authority],
        recent_blockhash,
    );

    banks_client.process_transaction(transaction).await.unwrap();
}

/// 测试代币批量转账功能
#[test]
fn test_batch_transfer_token() {
    solana_program_test::tokio::runtime::Runtime::new().unwrap().block_on(async {
        let program_id = batch_transfer::ID;
        let mut pt = ProgramTest::new("batch_transfer", program_id, None);
        pt.set_compute_max_units(1200_000);

        let admin = Keypair::new();
        let sender = Keypair::new();
        let mint = Keypair::new();
        let sender_token = Keypair::new();
        let recipient1 = Keypair::new();
        let recipient2 = Keypair::new();
        let recipient1_token = Keypair::new();
        let recipient2_token = Keypair::new();

        // 计算bank_account的PDA
        let (bank_account, _bump) = Pubkey::find_program_address(
            &[b"bank_account"],
            &program_id,
        );

        // 为管理员和发送者添加初始余额
        pt.add_account(
            admin.pubkey(),
            Account {
                lamports: 100 * LAMPORTS_PER_SOL,
                ..Account::default()
            },
        );
        pt.add_account(
            sender.pubkey(),
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

        // 设置手续费
        let set_fee_ix = Instruction {
            program_id: batch_transfer::ID,
            accounts: batch_transfer::accounts::SetFee {
                bank_account,
                admin: admin.pubkey(),
            }
            .to_account_metas(None),
            data: batch_transfer::instruction::SetFee {
                fee: 1000,
            }
            .data(),
        };

        let set_fee_tx = Transaction::new_signed_with_payer(
            &[set_fee_ix],
            Some(&admin.pubkey()),
            &[&admin],
            recent_blockhash,
        );

        banks_client.process_transaction(set_fee_tx).await.unwrap();

        // 初始化代币
        initialize_token_mint(&mut banks_client, &mint, &admin.pubkey(), 6, recent_blockhash).await;
        create_token_account(&mut banks_client, &sender_token, &mint.pubkey(), &sender, recent_blockhash).await;
        create_token_account(&mut banks_client, &recipient1_token, &mint.pubkey(), &recipient1, recent_blockhash).await;
        create_token_account(&mut banks_client, &recipient2_token, &mint.pubkey(), &recipient2, recent_blockhash).await;

        // 铸造代币给发送者
        mint_token(&mut banks_client, &mint.pubkey(), &sender_token.pubkey(), &admin, 100_000_000, recent_blockhash).await;

        // 执行批量转账
        let transfers = vec![
            TransferInfo {
                recipient: recipient1_token.pubkey(),
                amount: 1_000_000,
            },
            TransferInfo {
                recipient: recipient2_token.pubkey(),
                amount: 2_000_000,
            },
        ];

        let mut accounts = batch_transfer::accounts::BatchTransferToken {
            bank_account,
            sender: sender.pubkey(),
            token_account: sender_token.pubkey(),
            token_program: token::ID,
            system_program: system_program::ID,
        }.to_account_metas(None);

        // 添加接收者账户到剩余账户列表
        for info in transfers.iter() {
            accounts.push(AccountMeta::new(info.recipient, false));
        }

        let batch_transfer_ix = Instruction {
            program_id: batch_transfer::ID,
            accounts,
            data: batch_transfer::instruction::BatchTransferToken {
                transfers: transfers.clone(),
            }.data(),
        };

        let batch_transfer_tx = Transaction::new_signed_with_payer(
            &[batch_transfer_ix],
            Some(&sender.pubkey()),
            &[&sender],
            recent_blockhash,
        );

        banks_client.process_transaction(batch_transfer_tx).await.unwrap();

        // 验证发送者代币账户余额
        let sender_token_account = banks_client.get_account(sender_token.pubkey()).await.unwrap().unwrap();
        let sender_token_data = TokenAccount::unpack(&sender_token_account.data).unwrap();
        assert_eq!(sender_token_data.amount, 97_000_000);

        let recipient1_token_account = banks_client.get_account(recipient1_token.pubkey()).await.unwrap().unwrap();
        let recipient1_token_data = TokenAccount::unpack(&recipient1_token_account.data).unwrap();
        assert_eq!(recipient1_token_data.amount, 1_000_000);

        let recipient2_token_account = banks_client.get_account(recipient2_token.pubkey()).await.unwrap().unwrap();
        let recipient2_token_data = TokenAccount::unpack(&recipient2_token_account.data).unwrap();
        assert_eq!(recipient2_token_data.amount, 2_000_000);
    });
} 