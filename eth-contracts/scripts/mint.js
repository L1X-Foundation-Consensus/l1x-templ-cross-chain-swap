// mint.js
const hre = require("hardhat");
const { ethers } = hre;

const TOKEN_ADDRESS = "INSERT_CONTRACT_ADDRESS_HERE";
async function main() {
    const [deployer] = await ethers.getSigners();

    const Token = await ethers.getContractFactory("L1XNFT");
    const token = Token.attach(TOKEN_ADDRESS);

    // Prepare mint payload
    const mintPayload = {
        globalTxID: ethers.encodeBytes32String("Unique ID"), // make sure it's unique for each token
        to: deployer.address, // address to which the token will be minted
        tokenURI: "https://my.tokenbase.io/token/1", // token URI
    };

    // Get hash of the payload
    // Get hash of the payload
    const payloadHash = ethers.solidityPackedKeccak256(
        ["bytes32", "address", "string"],
        [mintPayload.globalTxID, mintPayload.to, mintPayload.tokenURI]
    );


    // Sign the payload hash
    const signature = await deployer.signMessage(payloadHash);

    // Mint the token
    const tx = await token.mint(mintPayload, signature);
    await tx.wait();

    console.log("Token minted tx:", tx.hash);
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });
