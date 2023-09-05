// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import "@openzeppelin/contracts/token/ERC721/extensions/ERC721Burnable.sol";
import "@openzeppelin/contracts/token/ERC721/extensions/ERC721URIStorage.sol";
import "@openzeppelin/contracts/utils/Counters.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/cryptography/SignatureChecker.sol";

contract L1XNFT is ERC721, ERC721URIStorage, Ownable {
    using Counters for Counters.Counter;
    Counters.Counter private _tokenIdCounter;
    string private _baseTokenURI;

    mapping(uint256 => bytes32) public globalTxID;
    address public L1X_SYSTEM_ACCOUNT;

    event AdvertisementTransferred(bytes32 indexed globalTxID, address to);
    event MirroredTokenMinted(bytes32 indexed globalTxID, uint256 tokenId, address to);

    struct MintPayload {
        bytes32 globalTxID;
        address to;
        string tokenURI;
    }

    constructor(string memory name,
                string memory symbol,
                string memory baseTokenURI,
                address _L1XSystemAccount) ERC721(name, symbol) {
        _baseTokenURI = baseTokenURI;
        L1X_SYSTEM_ACCOUNT = _L1XSystemAccount;
    }

    function _baseURI() internal view virtual override returns (string memory) {
        return _baseTokenURI;
    }

    function setBaseURI(string memory newBaseTokenURI) public onlyOwner {
        _baseTokenURI = newBaseTokenURI;
    }

    function mint(MintPayload memory payload,
                  bytes memory signature) public {
        bytes32 hash = getMintingPayloadHash(payload);
        require(
                SignatureChecker.isValidSignatureNow(L1X_SYSTEM_ACCOUNT, hash, signature),
                "Signature is not valid."
        );

        uint256 newTokenId = _tokenIdCounter.current();
        _safeMint(payload.to, newTokenId);
        _setTokenURI(newTokenId, payload.tokenURI);
        _tokenIdCounter.increment();
        globalTxID[newTokenId] = payload.globalTxID;
    }

    function getMintingPayloadHash(MintPayload memory payload) public pure returns (bytes32) {
        return keccak256(abi.encodePacked(payload.globalTxID,
                                          payload.to,
                                          payload.tokenURI));
    }

    function tokenURI(uint256 tokenId) public view override(ERC721, ERC721URIStorage) returns (string memory) {
        return super.tokenURI(tokenId);
    }

    function _burn(uint256 tokenId) internal override(ERC721, ERC721URIStorage) {
        require(ownerOf(tokenId) == msg.sender, "Caller is not the owner of the NFT");
        super._burn(tokenId);
    }

    function transferFrom(
        address from,
        address to,
        uint256 tokenId
    ) public virtual override(ERC721, IERC721) {
        super.transferFrom(from, to, tokenId);

        bytes32 _globalTxID = globalTxID[tokenId];
        if (_globalTxID.length > 0) {
            emit AdvertisementTransferred(_globalTxID, to);
            delete globalTxID[tokenId];
        }
    }

    function supportsInterface(bytes4 interfaceId) public view virtual override(ERC721, ERC721URIStorage) returns (bool) {
        return super.supportsInterface(interfaceId);
    }
}
