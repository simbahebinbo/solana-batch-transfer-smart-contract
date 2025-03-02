import * as anchor from "@coral-xyz/anchor";
import { BN } from "bn.js";

// 常量定义
export const LAMPORTS_PER_SOL = 1000000000;

/**
 * 等待一段时间（用于等待交易确认）
 * @param ms 等待的毫秒数
 */
export async function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

/**
 * 创建测试账户并提供初始SOL
 * @param connection Solana连接实例
 * @param count 需要创建的账户数量
 */
export async function initializeTestAccounts(
  connection: anchor.web3.Connection, 
  count: number
): Promise<anchor.web3.Keypair[]> {
  const keypairs: anchor.web3.Keypair[] = [];
  
  for (let i = 0; i < count; i++) {
    const keypair = anchor.web3.Keypair.generate();
    keypairs.push(keypair);
    
    // 为账户提供初始SOL
    await connection.requestAirdrop(keypair.publicKey, 10 * LAMPORTS_PER_SOL);
  }
  
  return keypairs;
}

/**
 * 创建测试代币
 * @param connection Solana连接实例
 * @param payer 支付者账户
 */
export async function createTestToken(
  connection: anchor.web3.Connection, 
  payer: anchor.web3.Keypair
): Promise<anchor.web3.PublicKey> {
  // 创建mint账户
  const mint = anchor.web3.Keypair.generate();
  
  // 获取Anchor内置的Token Program ID
  const TOKEN_PROGRAM_ID = new anchor.web3.PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
  
  // 获取租金豁免金额
  const lamports = await connection.getMinimumBalanceForRentExemption(82);
  
  // 创建mint账户的交易指令
  const createAccountInstruction = anchor.web3.SystemProgram.createAccount({
    fromPubkey: payer.publicKey,
    newAccountPubkey: mint.publicKey,
    lamports,
    space: 82, // Mint账户所需空间
    programId: TOKEN_PROGRAM_ID,
  });

  // 使用Anchor序列化创建初始化mint指令
  const dataLayout = {
    instruction: 0, // 初始化mint的指令码
    decimals: 9,    // 小数位数
    mintAuthority: payer.publicKey.toBuffer(),
    freezeAuthority: null,
    freezeAuthorityOption: 0
  };

  const initMintInstruction = new anchor.web3.TransactionInstruction({
    keys: [
      { pubkey: mint.publicKey, isSigner: false, isWritable: true },
      { pubkey: anchor.web3.SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false }
    ],
    programId: TOKEN_PROGRAM_ID,
    data: Buffer.from([
      0, // 初始化mint的指令码
      9, // 小数位数
      ...payer.publicKey.toBytes(),
      0, // 没有冻结授权
    ])
  });
  
  // 组合交易并发送
  const transaction = new anchor.web3.Transaction().add(
    createAccountInstruction,
    initMintInstruction
  );
  
  await anchor.web3.sendAndConfirmTransaction(
    connection,
    transaction,
    [payer, mint]
  );
  
  return mint.publicKey;
}

/**
 * 获取或创建代币账户
 * @param connection Solana连接实例
 * @param payer 支付者账户
 * @param mint 代币铸币账户
 * @param owner 代币所有者
 */
export async function getTestTokenAccount(
  connection: anchor.web3.Connection,
  payer: anchor.web3.Keypair,
  mint: anchor.web3.PublicKey,
  owner: anchor.web3.PublicKey
): Promise<anchor.web3.PublicKey> {
  const TOKEN_PROGRAM_ID = new anchor.web3.PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
  const ASSOCIATED_TOKEN_PROGRAM_ID = new anchor.web3.PublicKey("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
  
  // 查找相关Token账户地址
  const [associatedTokenAddress] = await anchor.web3.PublicKey.findProgramAddress(
    [
      owner.toBuffer(),
      TOKEN_PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
    ],
    ASSOCIATED_TOKEN_PROGRAM_ID
  );
  
  // 检查Token账户是否存在
  try {
    const tokenAccount = await connection.getAccountInfo(associatedTokenAddress);
    if (tokenAccount !== null) {
      return associatedTokenAddress;
    }
  } catch (error) {
    // 账户不存在，继续创建
  }
  
  // 创建相关Token账户的交易指令
  const transaction = new anchor.web3.Transaction().add(
    // 创建关联账户指令
    new anchor.web3.TransactionInstruction({
      keys: [
        { pubkey: payer.publicKey, isSigner: true, isWritable: true },
        { pubkey: associatedTokenAddress, isSigner: false, isWritable: true },
        { pubkey: owner, isSigner: false, isWritable: false },
        { pubkey: mint, isSigner: false, isWritable: false },
        { pubkey: anchor.web3.SystemProgram.programId, isSigner: false, isWritable: false },
        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
        { pubkey: anchor.web3.SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
      ],
      programId: ASSOCIATED_TOKEN_PROGRAM_ID,
      data: Buffer.from([]),
    })
  );
  
  // 发送交易
  await anchor.web3.sendAndConfirmTransaction(
    connection,
    transaction,
    [payer]
  );
  
  return associatedTokenAddress;
}

/**
 * 为代币账户铸造代币
 * @param connection Solana连接实例
 * @param payer 支付者账户
 * @param mint 代币铸币账户
 * @param destination 目标代币账户
 * @param authority 铸币权限账户
 */
export async function mintTestTokens(
  connection: anchor.web3.Connection,
  payer: anchor.web3.Keypair,
  mint: anchor.web3.PublicKey,
  destination: anchor.web3.PublicKey,
  authority: anchor.web3.Keypair
): Promise<void> {
  const TOKEN_PROGRAM_ID = new anchor.web3.PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
  
  // 铸币金额 - 减小到更合理的值
  const amount = new anchor.BN(1000 * LAMPORTS_PER_SOL);
  
  // 创建铸币指令数据
  const mintToInstruction = new anchor.web3.TransactionInstruction({
    keys: [
      { pubkey: mint, isSigner: false, isWritable: true },
      { pubkey: destination, isSigner: false, isWritable: true },
      { pubkey: authority.publicKey, isSigner: true, isWritable: false }
    ],
    programId: TOKEN_PROGRAM_ID,
    data: Buffer.from([
      7, // MintTo 的指令码
      ...amount.toArray("le", 8)
    ])
  });
  
  // 发送交易
  await anchor.web3.sendAndConfirmTransaction(
    connection,
    new anchor.web3.Transaction().add(mintToInstruction),
    [payer, authority]
  );
} 