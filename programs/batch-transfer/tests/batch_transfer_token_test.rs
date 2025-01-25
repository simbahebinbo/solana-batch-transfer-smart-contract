// use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
// use anchor_spl::token;
// use anchor_spl::token::{Mint, TokenAccount};
// use solana_program::instruction::Instruction;
// use solana_program::pubkey::Pubkey;
// use solana_program::system_program;
// use solana_program_test::{BanksClient, ProgramTest};
// use solana_sdk::account::Account;
// use solana_sdk::signature::Keypair;
// use solana_sdk::signer::Signer;
// use solana_sdk::transaction::Transaction;
//
// use batch_transfer::UserAccount;
//
// #[tokio::test]
// async fn test_deposit() {
//     let SetUpTest {
//         program_id,
//         pt,
//         admin,
//         user_account,
//         staking_token_account,
//         mint,
//         user_token_account,
//     } = SetUpTest::new();
//
//     let (mut banks_client, payer, recent_blockhash) = pt.start().await;
//
//     // 调用 reset_user_account 指令来初始化 user_account
//     let reset_user_account_ix = Instruction {
//         program_id,
//         accounts: batch_transfer::accounts::ResetUserAccount {
//             user_account: user_account.pubkey(),
//             admin: admin.pubkey(),
//             system_program: system_program::ID,
//         }
//             .to_account_metas(None),
//         data: batch_transfer::instruction::ResetUserAccount {}.data(),
//     };
//
//     let reset_user_account_tx = Transaction::new_signed_with_payer(
//         &[reset_user_account_ix],
//         Some(&admin.pubkey()),
//         &[&admin, &user_account],
//         recent_blockhash,
//     );
//
//     banks_client.process_transaction(reset_user_account_tx).await;
//
//     // 反序列化并检查用户账户
//     let user_account_data: UserAccount = load_and_deserialize(
//         banks_client.clone(),
//         user_account.pubkey(),
//     ).await;
//
//     // 确保用户账户的余额已更新
//     assert_eq!(user_account_data.amount, 0);
//
//     // 创建并发送初始化 mint 的指令
//     let init_mint_ix = token::spl_token::instruction::initialize_mint(
//         &token::ID,
//         &mint,
//         &admin.pubkey(),
//         None,
//         9, // decimals
//     ).unwrap();
//
//     let init_mint_tx = Transaction::new_signed_with_payer(
//         &[init_mint_ix],
//         Some(&admin.pubkey()),
//         &[&admin],
//         recent_blockhash,
//     );
//
//     banks_client.process_transaction(init_mint_tx).await.unwrap();
//
//     // 创建并发送初始化 token account 的指令
//     let init_user_token_account_ix = token::spl_token::instruction::initialize_account(
//         &token::ID,
//         &user_token_account,
//         &mint,
//         &admin.pubkey(),
//     ).unwrap();
//
//     let init_user_token_account_tx = Transaction::new_signed_with_payer(
//         &[init_user_token_account_ix],
//         Some(&admin.pubkey()),
//         &[&admin],
//         recent_blockhash,
//     );
//
//     banks_client.process_transaction(init_user_token_account_tx).await.unwrap();
//
//     // 创建并发送初始化 staking token account 的指令
//     let init_staking_token_account_ix = token::spl_token::instruction::initialize_account(
//         &token::ID,
//         &staking_token_account,
//         &mint,
//         &admin.pubkey(),
//     ).unwrap();
//
//     let init_staking_token_account_tx = Transaction::new_signed_with_payer(
//         &[init_staking_token_account_ix],
//         Some(&admin.pubkey()),
//         &[&admin],
//         recent_blockhash,
//     );
//
//     banks_client.process_transaction(init_staking_token_account_tx).await.unwrap();
//
//
//     // 设置铸币金额
//     let mint_amount: u64 = 1_000_000_000;
//
//     // 调用 faucet 函数
//     let faucet_ix = Instruction {
//         program_id: program_id,
//         accounts: batch_transfer::accounts::Faucet {
//             mint: mint,
//             user_token_account: user_token_account,
//             admin: admin.pubkey(),
//             token_program: token::ID,
//         }
//             .to_account_metas(None),
//         data: batch_transfer::instruction::Faucet { amount: mint_amount }.data(),
//     };
//
//     let faucet_tx = Transaction::new_signed_with_payer(
//         &[faucet_ix],
//         Some(&admin.pubkey()),
//         &[&admin],
//         recent_blockhash,
//     );
//     banks_client.process_transaction(faucet_tx).await.unwrap();
//
//     // Check the amount of the user token account
//     let token_account_data: TokenAccount = load_and_deserialize(
//         banks_client.clone(),
//         user_token_account,
//     ).await;
//
//     assert_eq!(token_account_data.amount, mint_amount);
//
//     // 设置存款金额
//     let deposit_amount: u64 = 1000;
//
//     // 调用存款函数
//     let deposit_ix = Instruction {
//         program_id: program_id,
//         accounts: batch_transfer::accounts::Deposit {
//             user_account: user_account.pubkey(),
//             user: admin.pubkey(),
//             user_token_account: user_token_account,
//             staking_token_account: staking_token_account,
//             token_program: token::ID,
//         }
//             .to_account_metas(None),
//         data: batch_transfer::instruction::Deposit { amount: deposit_amount }.data(),
//     };
//
//     let deposit_tx = Transaction::new_signed_with_payer(
//         &[deposit_ix],
//         Some(&admin.pubkey()),
//         &[&admin],
//         recent_blockhash,
//     );
//
//     banks_client.process_transaction(deposit_tx).await.unwrap();
//
//     // 反序列化并检查用户账户
//     let user_account_data: UserAccount = load_and_deserialize(
//         banks_client.clone(),
//         user_account.pubkey(),
//     ).await;
//
//     // 确保用户账户的余额已更新
//     assert_eq!(user_account_data.amount, deposit_amount);
// }
//
// pub struct SetUpTest {
//     pub program_id: Pubkey,
//     pub pt: ProgramTest,
//     pub admin: Keypair,
//     pub user_account: Keypair,
//     pub staking_token_account: Pubkey,
//     pub mint: Pubkey,
//     pub user_token_account: Pubkey,
// }
//
// impl SetUpTest {
//     pub fn new() -> Self {
//         let program_id = batch_transfer::ID;
//         let mut pt: ProgramTest = ProgramTest::new("batch_transfer", program_id, None);
//         pt.set_compute_max_units(1200_000);
//
//         let mut accounts: Vec<Keypair> = Vec::new();
//         let admin = Keypair::new();
//         accounts.push(admin.insecure_clone());
//         // 创建用户和质押账户
//         let user_account = Keypair::new();
//         accounts.push(user_account.insecure_clone());
//
//         for account in accounts {
//             //create a new account and fund with 1 SOL
//             pt.add_account(
//                 account.pubkey(),
//                 Account {
//                     lamports: 1_000_000_000,
//                     ..Account::default()
//                 },
//             );
//         }
//
//         // 添加 mint 账户
//         let mint = Pubkey::new_unique();
//         pt.add_account(
//             mint,
//             Account {
//                 lamports: 1000000000000,
//                 data: vec![0; Mint::LEN],
//                 owner: token::ID,
//                 executable: false,
//                 rent_epoch: 0,
//             },
//         );
//
//         // 添加 user token 账户
//         let user_token_account = Pubkey::new_unique();
//         pt.add_account(
//             user_token_account,
//             Account {
//                 lamports: 10000000000000,
//                 data: vec![0; TokenAccount::LEN],
//                 owner: token::ID,
//                 executable: false,
//                 rent_epoch: 0,
//             },
//         );
//
//         // 添加 staking token 账户
//         let staking_token_account = Pubkey::new_unique();
//         pt.add_account(
//             staking_token_account,
//             Account {
//                 lamports: 10000000000000,
//                 data: vec![0; TokenAccount::LEN],
//                 owner: token::ID,
//                 executable: false,
//                 rent_epoch: 0,
//             },
//         );
//
//         Self {
//             program_id,
//             pt,
//             admin,
//             user_account,
//             staking_token_account,
//             mint,
//             user_token_account,
//         }
//     }
// }
//
//
// pub async fn load_and_deserialize<T: AccountDeserialize>(
//     mut banks_client: BanksClient,
//     address: Pubkey,
// ) -> T {
//     let account = banks_client
//         .get_account(address)
//         .await
//         .unwrap()
//         .unwrap();
//
//     T::try_deserialize(&mut account.data.as_slice()).unwrap()
// }
