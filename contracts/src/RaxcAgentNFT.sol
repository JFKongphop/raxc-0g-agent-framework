// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ERC721} from "openzeppelin/token/ERC721/ERC721.sol";
import {Ownable} from "openzeppelin/access/Ownable.sol";
import {MerkleProof} from "openzeppelin/utils/cryptography/MerkleProof.sol";

/**
 * @notice ERC-7857 IntelligentData struct — matches 0G spec exactly.
 * @param dataDescription Human-readable label (e.g. "RAXC Audit: DeFiVault 2026-05-12")
 * @param dataHash        0G Storage root hash of the uploaded audit JSON (bytes32)
 */
struct IntelligentData {
  string dataDescription;
  bytes32 dataHash;
}

/**
 * @title RaxcAgentNFT
 * @notice ERC-7857 Intelligent NFT for RAXC autonomous security agents.
 *
 * Each token = one persistent agent identity.
 * After every audit, the Rust agent uploads its reasoning trace to 0G Storage,
 * gets a root hash, and calls update() to record it on-chain.
 *
 * ERC-7857 key functions:
 *   intelligentDatasOf(tokenId)        → current IntelligentData[]
 *   update(tokenId, IntelligentData[]) → update intelligence pointer
 *   mint(iDatas, to, agent)            → mint new agent identity
 *
 * Deployed on 0G Galileo testnet (chain ID 16602).
 */
contract RaxcAgentNFT is ERC721, Ownable {
  uint256 private _nextTokenId;

  /// @notice Current IntelligentData (0G Storage pointers) per token
  mapping(uint256 => IntelligentData[]) private _iDatas;

  /// @notice Full history of all past IntelligentData snapshots per token
  mapping(uint256 => IntelligentData[][]) private _iDataHistory;

  /// @notice Optional metadata URI per token
  mapping(uint256 => string) private _tokenURIs;

  /// @notice Authorized agent wallet per token (Rust process that calls update)
  mapping(uint256 => address) public agentAddress;

  // ── Audit Merkle Tree ─────────────────────────────────────────────────────
  // Every call to update() records a leaf: keccak256(tokenId, dataHash, timestamp).
  // The contract recomputes the Merkle root from all leaves after each update.
  // Anyone can call verifyAudit(leaf, proof) to prove a specific 0G upload happened.

  /// @notice All accumulated audit leaves (one per update() call across all tokens)
  bytes32[] public auditLeaves;

  /// @notice Current Merkle root over all auditLeaves — updated after every update()
  bytes32 public auditMerkleRoot;

  // ─────────────────────────────────────────────────────────────────────────

  /// @notice Emitted on every intelligence update — matches 0G ERC-7857 event spec
  event Updated(uint256 indexed tokenId, IntelligentData[] oldDatas, IntelligentData[] newDatas);
  event AgentMinted(uint256 indexed tokenId, address indexed owner, address indexed agent);
  /// @notice Emitted when a new audit leaf is added to the on-chain Merkle tree
  event AuditLeafAdded(uint256 indexed tokenId, bytes32 leaf, uint256 leafIndex, bytes32 newMerkleRoot);

  // OZ v4: Ownable uses msg.sender as owner in constructor, no argument needed
  constructor() ERC721("RAXC Agent Intelligence", "RAXC-AI") {}

  /**
   * @notice Mint a new agent identity with initial IntelligentData.
   * @param iDatas  Initial intelligence pointer (dataDescription + 0G Storage dataHash).
   * @param to      Owner address.
   * @param agent   Authorized agent wallet (Rust process) that will call update().
   */
  function mint(
    IntelligentData[] calldata iDatas,
    address to,
    address agent
  )
    external
    onlyOwner
    returns (uint256 tokenId)
  {
    require(to != address(0), "Zero address");
    require(iDatas.length > 0, "Empty data array");
    tokenId = _nextTokenId++;
    _safeMint(to, tokenId);
    agentAddress[tokenId] = agent;
    _setData(tokenId, iDatas);
    emit AgentMinted(tokenId, to, agent);
  }

  /**
   * @notice Update the intelligence pointers after an audit.
   *         Called by the RAXC Rust agent after uploading audit JSON to 0G Storage.
   *         Appends a leaf (keccak256(tokenId, dataHash, timestamp)) to the audit
   *         Merkle tree and recomputes auditMerkleRoot on-chain.
   * @param tokenId  Agent NFT token ID.
   * @param newDatas New IntelligentData[] — dataHash is the 0G Storage root hash.
   */
  function update(uint256 tokenId, IntelligentData[] calldata newDatas) external {
    require(msg.sender == agentAddress[tokenId] || msg.sender == owner(), "Not authorized");
    require(_exists(tokenId), "Token does not exist");
    require(newDatas.length > 0, "Empty data array");
    _setData(tokenId, newDatas);

    // Record the 0G Storage root hash as a leaf in our audit Merkle tree.
    // Leaf = keccak256(tokenId || dataHash || block.timestamp) — unique per audit.
    bytes32 leaf = keccak256(abi.encodePacked(tokenId, newDatas[0].dataHash, block.timestamp));
    auditLeaves.push(leaf);
    auditMerkleRoot = _computeMerkleRoot();
    emit AuditLeafAdded(tokenId, leaf, auditLeaves.length - 1, auditMerkleRoot);
  }

  function _setData(uint256 tokenId, IntelligentData[] calldata newDatas) internal {
    // Snapshot old for event + history
    IntelligentData[] memory oldDatas = _iDatas[tokenId];
    if (oldDatas.length > 0) {
      _iDataHistory[tokenId].push(oldDatas);
    }
    delete _iDatas[tokenId];
    for (uint256 i = 0; i < newDatas.length; i++) {
      require(newDatas[i].dataHash != bytes32(0), "Invalid data hash");
      _iDatas[tokenId].push(newDatas[i]);
    }
    emit Updated(tokenId, oldDatas, newDatas);
  }

  /// @notice Current IntelligentData for a token (ERC-7857 standard read)
  function intelligentDatasOf(uint256 tokenId) external view returns (IntelligentData[] memory) {
    return _iDatas[tokenId];
  }

  /// @notice Full audit trail (all past IntelligentData snapshots)
  function intelligenceHistory(uint256 tokenId) external view returns (IntelligentData[][] memory) {
    return _iDataHistory[tokenId];
  }

  /// @notice Number of intelligence updates for a token
  function intelligenceCount(uint256 tokenId) external view returns (uint256) {
    return _iDataHistory[tokenId].length;
  }

  function setAgentAddress(uint256 tokenId, address newAgent) external {
    require(msg.sender == ownerOf(tokenId) || msg.sender == owner(), "Not authorized");
    agentAddress[tokenId] = newAgent;
  }

  function setTokenURI(uint256 tokenId, string calldata uri) external onlyOwner {
    _tokenURIs[tokenId] = uri;
  }

  function tokenURI(uint256 tokenId) public view override returns (string memory) {
    require(_exists(tokenId), "Token does not exist");
    return _tokenURIs[tokenId];
  }

  // ── Audit Merkle Tree helpers ─────────────────────────────────────────────

  /**
   * @notice All raw audit leaves — one per update() call.
   *         Leaf = keccak256(tokenId, 0G-Storage-dataHash, block.timestamp)
   */
  function getAuditLeaves() external view returns (bytes32[] memory) {
    return auditLeaves;
  }

  /**
   * @notice Verify that a specific audit leaf belongs to the current auditMerkleRoot.
   * @param leaf  keccak256(tokenId, dataHash, timestamp) — same encoding as update()
   * @param proof Sibling hashes from off-chain Merkle proof generation
   * @return true if the leaf is in the tree
   */
  function verifyAudit(bytes32 leaf, bytes32[] calldata proof) external view returns (bool) {
    return MerkleProof.verify(proof, auditMerkleRoot, leaf);
  }

  /**
   * @notice Recompute the Merkle root from all stored leaves.
   *         Uses sorted-pair hashing — compatible with OZ MerkleProof and standard tooling.
   *         Gas cost is O(n log n) in leaf count; fine for hackathon-scale usage.
   */
  function _computeMerkleRoot() internal view returns (bytes32) {
    uint256 n = auditLeaves.length;
    if (n == 0) return bytes32(0);

    bytes32[] memory layer = new bytes32[](n);
    for (uint256 i = 0; i < n; i++) {
      layer[i] = auditLeaves[i];
    }

    while (layer.length > 1) {
      uint256 len = layer.length;
      uint256 newLen = (len + 1) / 2;
      bytes32[] memory next = new bytes32[](newLen);
      for (uint256 i = 0; i < newLen; i++) {
        if (2 * i + 1 < len) {
          // Sort pairs so the proof is order-independent (OZ standard)
          bytes32 a = layer[2 * i];
          bytes32 b = layer[2 * i + 1];
          next[i] = a < b ? keccak256(abi.encodePacked(a, b)) : keccak256(abi.encodePacked(b, a));
        } else {
          next[i] = layer[2 * i]; // odd leaf carries up
        }
      }
      layer = next;
    }
    return layer[0];
  }

  function totalSupply() external view returns (uint256) {
    return _nextTokenId;
  }
}
