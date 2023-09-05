// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC721/extensions/IERC721Metadata.sol";
import "@openzeppelin/contracts/token/ERC721/IERC721Receiver.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/Counters.sol";
import "@openzeppelin/contracts/utils/cryptography/SignatureChecker.sol";


contract L1XAdvertisementMultiSig is IERC721Receiver, Ownable {
    using Counters for Counters.Counter;

    // Transaction struct
    struct Transaction {
        address to;
        address tokenAddress;
        uint256 tokenId;
        bool confirmedByTokenOwner;
        bool confirmedByL1XSystemAccount;
    }

    struct Advertisement {
        address tokenOwner;
        uint256 price;
        bytes32 globalTxID;
    }

    struct BurnPayload {
        bytes32 globalTxID;
        address tokenAddress;
        uint256 tokenId;
    }

    Counters.Counter public txCount;
    // Mapping from transaction ID to transaction data
    mapping(uint256 => Transaction) public transactions;
    address public L1X_SYSTEM_ACCOUNT;
    // Mapping from token ID to advertisement data
    mapping(address => mapping(uint256 => Advertisement)) public _Advertisements;

    // Events
    event AdvertisementStarted(address indexed nftContract,
                               uint256 indexed tokenId,
                               string          tokenURI,
                               address         owner,
                               uint256         price);

    event AdvertisementFinished(bytes32 globalTxID,
                                address nftContract,
                                uint256 tokenId,
                                address oldOwner,
                                address newOwner,
                                uint256 price);

    event TransactionCreated(uint256 indexed txId,
                             address indexed tokenOwner,
                             address         nftContract,
                             uint256         tokenId);

    event TransactionExecuted(uint256 indexed txId,
                              address indexed executedBy);


    constructor(address _L1XSystemAccount) {
        require(
            _L1XSystemAccount != address(0),
            "L1XAdvertisementMultiSig: system account is the zero address"
        );
        L1X_SYSTEM_ACCOUNT = _L1XSystemAccount;
    }

    function onERC721Received(address, address, uint256, bytes calldata) external override returns (bytes4) {
        return this.onERC721Received.selector;
    }

    function advertise(address nftContractAddress,
                       uint256 tokenId,
                       uint256 price) external {

        IERC721Metadata nftContract = IERC721Metadata(nftContractAddress);
        require(
            nftContract.ownerOf(tokenId) == msg.sender,
            "Caller is not the owner of the NFT."
        );

        require(
            _Advertisements[nftContractAddress][tokenId].tokenOwner == address(0),
            "NFT is already advertised."
        );


        nftContract.safeTransferFrom(msg.sender, address(this), tokenId);

        _Advertisements[nftContractAddress][tokenId] =  Advertisement(msg.sender,
                                                                      price,
                                                                      bytes32(0));
        string memory tokenURI = nftContract.tokenURI(tokenId);

        emit AdvertisementStarted(nftContractAddress,
                                  tokenId,
                                  tokenURI,
                                  msg.sender,
                                  price);
    }

    function submitBurningTx(
        BurnPayload memory payload,
        bytes memory signature
    ) public {
        bytes32 hash = getBurningPayloadHash(payload);
        require(
                SignatureChecker.isValidSignatureNow(L1X_SYSTEM_ACCOUNT, hash, signature),
                "Signature is not valid."
        );

        // save globalTxID :)
        Advertisement memory advertisement = _Advertisements[payload.tokenAddress][payload.tokenId];
        advertisement.globalTxID = payload.globalTxID;
        _Advertisements[payload.tokenAddress][payload.tokenId] = advertisement;

        uint256 newTxId = txCount.current();
        transactions[newTxId] = Transaction(
            address(0),
            payload.tokenAddress,
            payload.tokenId,
            false,
            true
        );
        txCount.increment();

        emit TransactionCreated(newTxId, advertisement.tokenOwner, payload.tokenAddress, payload.tokenId);
    }

    function getBurningPayloadHash(BurnPayload memory payload) public pure returns (bytes32) {
        return keccak256(abi.encodePacked(payload.globalTxID,
                                          payload.tokenAddress,
                                          payload.tokenId));
    }

    function submitTx(
        address to,
        address tokenAddress,
        uint256 tokenId
    )
    public {
        Advertisement memory advertisement = _Advertisements[tokenAddress][tokenId];
        require(advertisement.tokenOwner != address(0), "NFT is has not been advertised yet");
        require(advertisement.tokenOwner == msg.sender, "Caller is not the owner of the NFT");

        uint256 newTxId = txCount.current();
        transactions[newTxId] = Transaction(
            to,
            tokenAddress,
            tokenId,
            advertisement.tokenOwner == msg.sender,
            owner() == msg.sender
        );
        txCount.increment();

        emit TransactionCreated(newTxId, advertisement.tokenOwner, tokenAddress, tokenId);
    }


    // TODO: make executeTx working with signature as well
    function executeTx(uint txId)
    public {
        address tokenOwner = _Advertisements[transactions[txId].tokenAddress][transactions[txId].tokenId].tokenOwner;

        if (msg.sender == owner()) {
            require(transactions[txId].confirmedByTokenOwner, "Transaction not confirmed by token owner");
        } else if (msg.sender == tokenOwner) {
            require(transactions[txId].confirmedByL1XSystemAccount, "Transaction not confirmed by L1X system account");
        } else {
            revert("Caller is not authorized");
        }

        IERC721 nftContract = IERC721(transactions[txId].tokenAddress);
        nftContract.safeTransferFrom(address(this), transactions[txId].to, transactions[txId].tokenId);
        emit TransactionExecuted(txId, msg.sender);

        uint256 price = _Advertisements[transactions[txId].tokenAddress][transactions[txId].tokenId].price;
        bytes32 globalTxID = _Advertisements[transactions[txId].tokenAddress][transactions[txId].tokenId].globalTxID;

        emit AdvertisementFinished(globalTxID,
                                   transactions[txId].tokenAddress,
                                   transactions[txId].tokenId,
                                   tokenOwner,
                                   transactions[txId].to,
                                   price);

        delete transactions[txId];
        delete _Advertisements[transactions[txId].tokenAddress][transactions[txId].tokenId];
    }
}