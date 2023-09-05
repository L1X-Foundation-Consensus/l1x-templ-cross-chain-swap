// deploy.js
const hre = require("hardhat");

async function main() {
    const [deployer] = await hre.ethers.getSigners();

    console.log(
        "Deploying contracts with the account:",
        deployer.address
    );

    // getBalance is a method on the provider, not the signer
    console.log("Account balance:", (await deployer.provider.getBalance(deployer.address)).toString());

    const Token = await hre.ethers.getContractFactory("L1XNFT");
    const token = await Token.deploy("L1XNFT", "L1X", "https://my.tokenbase.io/", deployer.address);

    await token.waitForDeployment();

    console.log("Token contract deployed to:", await token.getAddress());
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
});

