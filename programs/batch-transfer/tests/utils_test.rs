use anchor_client::{
    Client,
    Cluster,
    Program,
    anchor_lang::{
        prelude::ProgramError,
        system_program,
        AccountDeserialize,
    },
    solana_sdk::{
        signature::{Keypair, Signer},
        pubkey::Pubkey,
        instruction::AccountMeta,
    },
};
use std::rc::Rc;
use batch_transfer::{self as batch_transfer, TransferInfo};
use tokio;

#[test]
fn test_safe_add() {
    // 正常情况
    let result = batch_transfer::safe_add(10, 20);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 30);

    // 溢出情况
    let result = batch_transfer::safe_add(u64::MAX, 1);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), ProgramError::InvalidArgument);
}

#[test]
fn test_safe_sum_transfer_info() {
    // 创建测试数据
    let transfer1 = TransferInfo {
        recipient: Pubkey::new_unique(),
        amount: 100,
    };
    let transfer2 = TransferInfo {
        recipient: Pubkey::new_unique(),
        amount: 200,
    };
    let transfers = vec![transfer1.clone(), transfer2]; // 添加clone调用

    // 测试正常情况
    let result = batch_transfer::safe_sum_transfer_info(&transfers);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 300);

    // 创建可能导致溢出的测试数据
    let transfer_overflow = TransferInfo {
        recipient: Pubkey::new_unique(),
        amount: u64::MAX,
    };
    let transfers_overflow = vec![transfer1, transfer_overflow];

    // 测试溢出情况
    let result = batch_transfer::safe_sum_transfer_info(&transfers_overflow);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), ProgramError::InvalidArgument);
}

// 获取测试程序和支付者
pub fn get_test_program() -> (Program<Rc<Keypair>>, Rc<Keypair>) {
    // 程序ID
    let program_id = batch_transfer::ID;
    
    // 创建一个随机的支付者
    let payer = Rc::new(Keypair::new());
    let payer_clone = payer.clone();
    
    // 创建客户端
    let client = Client::new(
        Cluster::Localnet,
        payer.clone(),
    );
    
    // 获取程序
    let program = client.program(program_id).expect("无法获取程序");
    
    (program, payer_clone)
}

pub fn get_bank_account(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"bank_account"], program_id)
}

// 辅助函数：用于创建和初始化测试环境
#[cfg(test)]
pub mod test_utils {
    use super::*;
    use batch_transfer::accounts as batch_accounts;
    use batch_transfer::instruction as batch_instructions;
    use batch_transfer::TransferInfo;
    
    // 初始化银行账户
    pub async fn initialize_bank_account(
        program: &Program<Rc<Keypair>>,
        admin: &Keypair,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (bank_account, _) = get_bank_account(&program.id());
        
        let sig = program
            .request()
            .accounts(batch_accounts::Initialize {
                bank_account,
                deployer: admin.pubkey(),
                system_program: system_program::ID,
            })
            .args(batch_instructions::Initialize {
                admin: admin.pubkey(),
            })
            .signer(admin)
            .send()?;
            
        println!("Bank account initialized: {}", sig);
        Ok(())
    }
    
    // 设置费用
    pub async fn set_fee(
        program: &Program<Rc<Keypair>>,
        admin: &Keypair,
        fee: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (bank_account, _) = get_bank_account(&program.id());
        
        let sig = program
            .request()
            .accounts(batch_accounts::SetFee {
                bank_account,
                admin: admin.pubkey(),
            })
            .args(batch_instructions::SetFee {
                fee,
            })
            .signer(admin)
            .send()?;
            
        println!("Fee set to {}: {}", fee, sig);
        Ok(())
    }
    
    // 批量转账SOL
    pub async fn batch_transfer_sol(
        program: &Program<Rc<Keypair>>,
        sender: &Keypair,
        recipients: &[Pubkey],
        amounts: &[u64],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (bank_account, _) = get_bank_account(&program.id());
        
        // 创建TransferInfo列表
        let transfers: Vec<TransferInfo> = recipients
            .iter()
            .zip(amounts.iter())
            .map(|(recipient, amount)| TransferInfo {
                recipient: *recipient,
                amount: *amount,
            })
            .collect();
        
        // 构建基本账户
        let mut accounts = vec![
            AccountMeta::new(sender.pubkey(), true),  // sender - 签名者且可变
            AccountMeta::new(bank_account, false),    // bank_account
            AccountMeta::new_readonly(system_program::ID, false), // system_program
        ];
        
        // 添加所有收款人账户
        for recipient in recipients {
            accounts.push(AccountMeta::new(*recipient, false));
        }
        
        let sig = program
            .request()
            .accounts(accounts)
            .args(batch_instructions::BatchTransferSol {
                transfers,
            })
            .signer(sender)
            .send()?;
            
        println!("Batch transfer completed: {}", sig);
        Ok(())
    }
    
    // 资金请求 - 使用 anchor_client 的 RPC 客户端
    pub async fn request_airdrop(
        program: &Program<Rc<Keypair>>,
        pubkey: &Pubkey,
        lamports: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 获取RPC客户端
        let rpc_client = program.rpc();
        
        // 请求空投
        let sig = rpc_client.request_airdrop(pubkey, lamports)?;
        
        println!("Airdrop of {} lamports to {}: {}", lamports, pubkey, sig);
        // 使用tokio::time::sleep代替同步等待确认
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        Ok(())
    }
    
    // 获取账户余额
    pub async fn get_account_balance(
        program: &Program<Rc<Keypair>>,
        pubkey: &Pubkey,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        // 获取RPC客户端
        let rpc_client = program.rpc();
        
        // 获取余额
        let balance = rpc_client.get_balance(pubkey)?;
        
        Ok(balance)
    }
    
    // 获取账户数据
    pub async fn get_account_data<T: AccountDeserialize>(
        program: &Program<Rc<Keypair>>,
        pubkey: &Pubkey,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let account_data = program.rpc().get_account(pubkey)?;
        let account = T::try_deserialize(&mut account_data.data.as_ref())?;
        
        Ok(account)
    }
    
    // 添加阻塞版本的函数
    pub fn request_airdrop_blocking(
        program: &Program<Rc<Keypair>>,
        pubkey: &Pubkey,
        lamports: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 获取RPC客户端
        let rpc_client = program.rpc();
        
        // 请求空投
        let sig = rpc_client.request_airdrop(pubkey, lamports)?;
        
        println!("Airdrop of {} lamports to {}: {}", lamports, pubkey, sig);
        // 增加等待时间，确保交易确认
        std::thread::sleep(std::time::Duration::from_secs(3));
        
        // 确认交易已完成
        let _confirmed = rpc_client.confirm_transaction(&sig)?;
        println!("交易确认完成");
        
        // 再次等待，确保账户余额已更新
        std::thread::sleep(std::time::Duration::from_secs(1));
        
        // 验证余额已增加
        let balance = rpc_client.get_balance(pubkey)?;
        println!("当前账户余额: {} lamports", balance);
        
        Ok(())
    }
    
    pub fn initialize_bank_account_blocking(
        program: &Program<Rc<Keypair>>,
        admin: &Keypair,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (bank_account, _) = get_bank_account(&program.id());
        
        let sig = program
            .request()
            .accounts(batch_accounts::Initialize {
                bank_account,
                deployer: admin.pubkey(),
                system_program: system_program::ID,
            })
            .args(batch_instructions::Initialize {
                admin: admin.pubkey(),
            })
            .signer(admin)
            .send()?;
            
        println!("Bank account initialized: {}", sig);
        std::thread::sleep(std::time::Duration::from_millis(500));
        Ok(())
    }
    
    pub fn set_fee_blocking(
        program: &Program<Rc<Keypair>>,
        admin: &Keypair,
        fee: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (bank_account, _) = get_bank_account(&program.id());
        
        let sig = program
            .request()
            .accounts(batch_accounts::SetFee {
                bank_account,
                admin: admin.pubkey(),
            })
            .args(batch_instructions::SetFee {
                fee,
            })
            .signer(admin)
            .send()?;
            
        println!("Fee set to {}: {}", fee, sig);
        std::thread::sleep(std::time::Duration::from_millis(500));
        Ok(())
    }
} 