#!/usr/bin/env node

const { Keypair, PublicKey, Connection } = require('@solana/web3.js');
const bip39 = require('bip39');
const { derivePath } = require('ed25519-hd-key');
const readline = require('readline');

// Phantom wallet derivation paths
const DERIVATION_PATHS = [
    "m/44'/501'/0'/0'",  // Account 1 (primary)
    "m/44'/501'/1'/0'",  // Account 2 (prawdopodobnie tutaj są 107 SOL!)
    "m/44'/501'/2'/0'",  // Account 3
    "m/44'/501'/3'/0'",  // Account 4
    "m/44'/501'/4'/0'",  // Account 5
    "m/501'/0'/0/0"      // Legacy path (starsze wersje)
];

const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});

function question(prompt) {
    return new Promise((resolve) => {
        rl.question(prompt, resolve);
    });
}

async function getKeypairFromSeed(seedPhrase, derivationPath) {
    try {
        // Validate seed phrase
        if (!bip39.validateMnemonic(seedPhrase)) {
            throw new Error('Invalid seed phrase');
        }
        
        // Generate seed from mnemonic
        const seed = await bip39.mnemonicToSeed(seedPhrase);
        
        // Derive key using the path
        const derivedSeed = derivePath(derivationPath, seed.toString('hex')).key;
        
        // Create keypair
        const keypair = Keypair.fromSeed(derivedSeed);
        
        return keypair;
    } catch (error) {
        console.error(`Error deriving key for path ${derivationPath}:`, error.message);
        return null;
    }
}

async function checkBalance(publicKey) {
    try {
        // Use mainnet-beta for real balances
        const connection = new Connection('https://api.mainnet-beta.solana.com');
        const balance = await connection.getBalance(new PublicKey(publicKey));
        return balance / 1e9; // Convert lamports to SOL
    } catch (error) {
        console.error('Error checking balance:', error.message);
        return 0;
    }
}

async function main() {
    console.log('🔍 PHANTOM WALLET KEY RECOVERY TOOL');
    console.log('===================================\n');
    
    console.log('⚠️  BEZPIECZEŃSTWO:');
    console.log('- Upewnij się, że jesteś w bezpiecznym środowisku');
    console.log('- Seed phrase będzie widoczny podczas wpisywania');
    console.log('- Po zakończeniu usuń historię terminala\n');
    
    const proceed = await question('Czy chcesz kontynuować? (tak/nie): ');
    if (proceed.toLowerCase() !== 'tak') {
        console.log('Anulowano.');
        rl.close();
        return;
    }
    
    console.log('\n📝 Wprowadź seed phrase z Phantom wallet:');
    console.log('(12 lub 24 słowa oddzielone spacjami)\n');
    
    const seedPhrase = await question('Seed phrase: ');
    
    if (!seedPhrase.trim()) {
        console.log('❌ Nie wprowadzono seed phrase');
        rl.close();
        return;
    }
    
    console.log('\n🔍 Sprawdzam wszystkie derivation paths...\n');
    
    const results = [];
    
    for (let i = 0; i < DERIVATION_PATHS.length; i++) {
        const path = DERIVATION_PATHS[i];
        console.log(`Sprawdzam path ${i + 1}/${DERIVATION_PATHS.length}: ${path}`);
        
        const keypair = await getKeypairFromSeed(seedPhrase, path);
        
        if (keypair) {
            const publicKey = keypair.publicKey.toString();
            console.log(`  📍 Adres: ${publicKey}`);
            
            // Check balance
            const balance = await checkBalance(publicKey);
            console.log(`  💰 Saldo: ${balance} SOL`);
            
            results.push({
                path,
                publicKey,
                balance,
                keypair
            });
            
            // Highlight if this looks like the target account
            if (balance > 100) {
                console.log(`  🎯 ZNALEZIONO! To może być konto z 107 SOL!`);
            }
        }
        
        console.log('');
    }
    
    console.log('\n📊 PODSUMOWANIE:');
    console.log('================');
    
    let totalBalance = 0;
    results.forEach((result, index) => {
        console.log(`\nKonto ${index + 1}:`);
        console.log(`  Path: ${result.path}`);
        console.log(`  Adres: ${result.publicKey}`);
        console.log(`  Saldo: ${result.balance} SOL`);
        totalBalance += result.balance;
        
        if (result.balance > 100) {
            console.log(`  🎯 TO PRAWDOPODOBNIE TWOJE 107 SOL!`);
        }
    });
    
    console.log(`\n💰 Łączne saldo: ${totalBalance} SOL`);
    
    // Ask if user wants to export any keys
    if (results.some(r => r.balance > 0)) {
        console.log('\n💾 Czy chcesz wyeksportować klucze prywatne dla kont z saldem?');
        const exportKeys = await question('(tak/nie): ');
        
        if (exportKeys.toLowerCase() === 'tak') {
            results.forEach((result, index) => {
                if (result.balance > 0) {
                    console.log(`\n🔑 Klucz prywatny dla konta ${index + 1} (${result.balance} SOL):`);
                    console.log(`Adres: ${result.publicKey}`);
                    console.log(`Klucz: [${result.keypair.secretKey.toString()}]`);
                    console.log(`Base58: ${result.keypair.secretKey.toString('base64')}`);
                }
            });
        }
    }
    
    console.log('\n✅ Gotowe! Pamiętaj o bezpieczeństwie swoich kluczy.');
    rl.close();
}

main().catch(console.error);
