# L1XNFT Project

This project allows you to deploy an L1XNFT contract and mint a new NFT.

## Prerequisites

Ensure that you have Node.js and npm installed on your local machine.

## Setup

1. **Install the required dependencies:**

    Open your terminal and run the following command:

    ```bash
    npm install
    ```

2. **Compile the smart contracts:**

    Run the following command in your terminal:

    ```bash
    npx hardhat compile
    ```

3. **Launch local node:**

    Run the following command in another terminal:

    ```bash
    npx hardhat node
    ```

 
4. **Deploy the contract to the local hardhat network:**

    Run the following command in your terminal:

    ```bash
    npx hardhat run --network hardhat scripts/deploy_l1xnft.js
    ```

    The script will output the deployed contract's address.

## Minting a new NFT

Before minting a new NFT, you will need to update the `mint.js` script with the deployed contract's address from the previous step.

1. **Update the contract's address in the mint script:**

    Open `scripts/mint.js` in a text editor and replace `"INSERT_CONTRACT_ADDRESS_HERE"` with the contract's address.

2. **Mint a new NFT:**

    After updating the contract's address, you can mint a new NFT by running the following command in your terminal:

    ```bash
    npx hardhat run --network hardhat scripts/mint.js
    ```
