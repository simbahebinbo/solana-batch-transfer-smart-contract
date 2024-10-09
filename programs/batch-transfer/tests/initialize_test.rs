// use anchor_client::solana_sdk::signature::Keypair;
// use anchor_client::{Client, Cluster};
// use batch_transfer::{accounts, instruction};
// use std::rc::Rc;
//
// #[tokio::test]
// async fn test_initialize() {
//     let program_id = batch_transfer::ID;
//
//     let payer = Keypair::new();
//     let client = Client::new(Cluster::Localnet, Rc::new(payer));
//
//     // Create program
//     let program = client.program(program_id).unwrap();
//
//     // Send a transaction
//     // 初始化账户
//     let deployer = Keypair::new();
//     let admin = Keypair::new();
//     let bank_account = Keypair::new();
//
//     // program
//     //     .request()
//     //     .accounts(accounts::Initialize {
//     //         bank_account: bank_account.pubkey(),
//     //         deployer: program.payer(),
//     //         system_program: system_program::ID,
//     //     })
//     //     .args(instruction::Initialize {
//     //         admin: admin.pubkey()
//     //     })
//     //     .signer(&deployer)
//     //     .send()
//     //     .await
//     //     .unwrap();
//     //
//     // // Fetch an account
//     // let bank_account: BankAccount = program.account(bank_account.pubkey()).await.unwrap();
//     // assert_eq!(bank_account.admin, admin.pubkey());
// }
