// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/cryptography/SignatureChecker.sol";

contract L1XOpenSeaWallet is Ownable {
    address public L1X_SYSTEM_ACCOUNT;

    // mapping to keep track of the globalTxId's that have been used to prevent double spending
    mapping(bytes32 => bool) private _spentTransactions;

    struct TransferPayload {
        bytes32 globalTxId;
        address sellerAddress;
        uint256 amount;
    }

    event Transferred(bytes32 indexed globalTxId, address to, uint256 amount);

    constructor(address _L1XSystemAccount) {
        L1X_SYSTEM_ACCOUNT = _L1XSystemAccount;
    }

    function transferFundsToSeller(TransferPayload memory payload, bytes memory signature) public {
        require(!_spentTransactions[payload.globalTxId], "Transaction already spent");
        bytes32 hash = getTransferPayloadHash(payload);
        require(
            SignatureChecker.isValidSignatureNow(L1X_SYSTEM_ACCOUNT, hash, signature),
            "Signature is not valid."
        );

        // marking this globalTxId as spent
        _spentTransactions[payload.globalTxId] = true;

        // transferring the ETH
        (bool success, ) = payload.sellerAddress.call{value: payload.amount}("");
        require(success, "Transfer failed.");

        emit Transferred(payload.globalTxId, payload.sellerAddress, payload.amount);
    }

    function getTransferPayloadHash(TransferPayload memory payload) public pure returns (bytes32) {
        return keccak256(abi.encodePacked(payload.globalTxId, payload.sellerAddress, payload.amount));
    }

     // To receive ether directly to the contract
    receive() external payable {}
}
