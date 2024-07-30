mod programs;
#[cfg(test)]
mod tests {
    const RPC_URL: &str = "https://api.devnet.solana.com";
    use solana_client::rpc_client::RpcClient;
    use solana_sdk::{
        message::Message,
        pubkey::Pubkey,
        signature::{Keypair, Signer, read_keypair_file},
        transaction::Transaction,
    };
    use solana_program::system_instruction::transfer;
    use solana_sdk::system_program;
    use std::str::FromStr;
    use bs58;
    use std::io::{self, BufRead};
    use crate::programs::wba_prereq::{WbaPrereqProgram, CompleteArgs};

    #[test]
    fn keygen() {
        // Create a new keypair
        let kp = Keypair::new();
        println!("You've generated a new Solana wallet: {}", kp.pubkey().to_string());
        println!("");
        println!("To save your wallet, copy and paste the following into a JSON file:");
        println!("{:?}", kp.to_bytes());
    }

    #[test]
    fn airdrop() {
        // Import our keypair
        let keypair = read_keypair_file("dev-wallet.json").expect("Couldn't find wallet file");
        
        // Connected to Solana Devnet RPC Client
        let client = RpcClient::new(RPC_URL);
        
        // We're going to claim 2 devnet SOL tokens (2 billion lamports)
        match client.request_airdrop(&keypair.pubkey(), 2_000_000_000u64) {
            Ok(s) => {
                println!("Success! Check out your TX here:");
                println!("https://explorer.solana.com/tx/{}?cluster=devnet", s.to_string());
            },
            Err(e) => println!("Oops, something went wrong: {}", e.to_string())
        };
    }

    #[test]
    fn transfer_sol() {
        // Import our keypair
        let keypair = read_keypair_file("dev-wallet.json").expect("Couldn't find wallet file");
        
        // Define our WBA public key
        let to_pubkey = Pubkey::from_str("5EvTWpYhC3PFkiyuMzcTebKgT13S9BJwTtkXGiuiVrPf").unwrap();
        
        // Create a Solana devnet connection
        let rpc_client = RpcClient::new(RPC_URL);
        
        // Get recent blockhash
        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");
        
        // Create the transaction
        let transaction = Transaction::new_signed_with_payer(
            &[transfer(
                &keypair.pubkey(),
                &to_pubkey,
                1_000_000, // 0.1 SOL (1 million lamports)
            )],
            Some(&keypair.pubkey()),
            &[&keypair],
            recent_blockhash,
        );
        
        // Send the transaction
        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");
        
        // Print our transaction out
        println!(
            "Success! Check out your TX here: https://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }

    #[test]
    fn empty_wallet() {
        // Import our keypair
        let keypair = read_keypair_file("dev-wallet.json").expect("Couldn't find wallet file");
        
        // Define our WBA public key
        let to_pubkey = Pubkey::from_str("5EvTWpYhC3PFkiyuMzcTebKgT13S9BJwTtkXGiuiVrPf").unwrap();
        
        // Create a Solana devnet connection
        let rpc_client = RpcClient::new(RPC_URL);
        
        // Get the balance of the dev wallet
        let balance = rpc_client
            .get_balance(&keypair.pubkey())
            .expect("Failed to get balance");

        // Get recent blockhash
        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");
        
        // Create a test transaction to calculate fees
        let message = Message::new_with_blockhash(
            &[transfer(
                &keypair.pubkey(),
                &to_pubkey,
                balance,
            )],
            Some(&keypair.pubkey()),
            &recent_blockhash,
        );

        // Calculate the fee for this transaction
        let fee = rpc_client
            .get_fee_for_message(&message)
            .expect("Failed to get fee calculator");

        // Calculate the amount to send after fee
        let amount_to_send = balance - fee;

        // Create the actual transaction
        let transaction = Transaction::new_signed_with_payer(
            &[transfer(
                &keypair.pubkey(),
                &to_pubkey,
                amount_to_send,
            )],
            Some(&keypair.pubkey()),
            &[&keypair],
            recent_blockhash,
        );

        // Send the transaction
        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");

        // Print our transaction out
        println!(
            "Success! Check out your TX here: https://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }

    #[test]
    fn base58_to_wallet() {
        println!("Input your private key as base58:");
        let stdin = io::stdin();
        let base58 = stdin.lock().lines().next().unwrap().unwrap();
        let wallet = bs58::decode(base58).into_vec().unwrap();
        println!("Your wallet file is:");
        println!("{:?}", wallet);
    }

    #[test]
    fn wallet_to_base58() {
        println!("Input your private key as a wallet file byte array:");
        let stdin = io::stdin();
        let wallet = stdin.lock().lines().next().unwrap().unwrap()
            .trim_start_matches('[').trim_end_matches(']')
            .split(',')
            .map(|s| s.trim().parse::<u8>().unwrap())
            .collect::<Vec<u8>>();
        let base58 = bs58::encode(wallet).into_string();
        println!("Your private key is:");
        println!("{:?}", base58);
    }

    #[test]
    fn complete_prereq() {
        // Create a Solana devnet connection
        let rpc_client = RpcClient::new(RPC_URL);
    
        // Define the WBA wallet and read the keypair
        let signer = read_keypair_file("wba-wallet.json").expect("Couldn't find wallet file");
    
        // Create PDA for prereq account
        let (prereq, _bump) = Pubkey::find_program_address(
            &[b"prereq", signer.pubkey().as_ref()],
            &Pubkey::from_str("HC2oqz2p6DEWfrahenqdq2moUcga9c9biqRBcdK3XKU1").unwrap(),
        );
    
        // Define the complete args with your GitHub account
        let args = CompleteArgs {
            github: b"zsh28".to_vec() // Replace with your GitHub username
        };
    
        // Get recent blockhash
        let blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");
    
        // Now we can invoke the "complete" function
        let transaction = WbaPrereqProgram::complete(
            &[&signer.pubkey(), &prereq, &system_program::id()],
            &args,
            Some(&signer.pubkey()),
            &[&signer],
            blockhash,
        );
    
        // Send the transaction and handle errors
        let result = rpc_client.send_and_confirm_transaction(&transaction);
        match result {
            Ok(signature) => {
                println!(
                    "Success! Check out your TX here: https://explorer.solana.com/tx/{}/?cluster=devnet",
                    signature
                );
            }
            Err(e) => {
                println!("Failed to send transaction: {:?}", e);
            }
        }
    }
}
