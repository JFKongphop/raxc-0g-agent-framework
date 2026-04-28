/*!
RAXC Analyzer — RAG-powered smart contract vulnerability scanner.

Usage:
  cargo run --bin analyze                          # uses built-in sample contract
  cargo run --bin analyze -- path/to/contract.sol  # analyze a specific file
*/

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use raxc::{analyze, build_qdrant, load_env, match_functions, payload_str, save_markdown, TOP_K};
use reqwest::Client;

// ─── Entry point ──────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
  load_env();

  let api_key = std::env::var("OPENAI_API_KEY").context("OPENAI_API_KEY not set")?;
  let http = Client::new();
  let qdrant = build_qdrant()?;

  let args: Vec<String> = std::env::args().collect();

  let (contract_code, contract_name): (String, String) = if args.len() > 1 {
    let path = &args[1];
    let code = fs::read_to_string(path)
      .with_context(|| format!("Failed to read contract file: {}", path))?;
    let name = Path::new(path)
      .file_stem()
      .unwrap_or_default()
      .to_string_lossy()
      .to_string();
    println!("[*] Analyzing: {}\n", path);
    (code, name)
  } else {
    println!("[*] No file specified — using built-in sample.\n");
    (SAMPLE_CONTRACT.to_string(), "sample".to_string())
  };

  let (report, results) = analyze(&http, &qdrant, &api_key, &contract_code).await?;

  println!("[*] Running function-level exploit matching...");
  let func_matches = match_functions(&http, &qdrant, &api_key, &contract_code, 3).await?;

  // ── Print summary ─────────────────────────────────────────────────────────
  println!("{}", "=".repeat(60));
  println!("TOP {} SIMILAR EXPLOITS (from DeFiHackLabs)", TOP_K);
  println!("{}", "=".repeat(60));
  for (i, r) in results.iter().enumerate() {
    let p = &r.payload;
    println!("  #{}  {}", i + 1, payload_str(p, "exploit_name"));
    println!("       Date    : {}", payload_str(p, "date"));
    println!("       Chain   : {}", payload_str(p, "chain"));
    println!("       Lost    : {}", payload_str(p, "total_lost"));
    println!("       Tx      : {}", payload_str(p, "attack_tx"));
    println!("       Score   : {:.3}", r.score);
    println!();
  }

  println!("{}", "=".repeat(60));
  println!("FUNCTION EXPLOIT MATCHING");
  println!("{}", "=".repeat(60));
  for fm in &func_matches {
    println!("\n  [{}]", fm.function);
    for (j, r) in fm.matches.iter().enumerate() {
      let p = &r.payload;
      let src = payload_str(p, "source");
      println!(
        "    #{}  {:<30}  score={:.3}  chain={}  lost={}  [{}]",
        j + 1,
        payload_str(p, "exploit_name"),
        r.score,
        payload_str(p, "chain"),
        payload_str(p, "total_lost"),
        src
      );
    }
  }
  println!();

  println!("{}", "=".repeat(60));
  println!("RAXC SECURITY REPORT");
  println!("{}", "=".repeat(60));
  println!("{}", report);
  println!("{}", "=".repeat(60));

  let md_path = save_markdown(&report, &results, &contract_name, Some(&func_matches))?;
  println!("\n[*] Report saved → {}", md_path);

  Ok(())
}

// ─── Built-in sample contract ─────────────────────────────────────────────────

const SAMPLE_CONTRACT: &str = r#"
// https://tornado.cash
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {MerkleTreeWithHistory} from "./MerkleTreeWithHistory.sol";
import {ReentrancyGuard} from "./ReentrancyGuard.sol";
import {MockToken} from "./MockToken.sol";
import {Groth16Verifier} from "./Groth16Verifier.sol";

interface IVerifier {
  function verifyProof(
    uint256[2] calldata _pA,
    uint256[2][2] calldata _pB,
    uint256[2] calldata _pC,
    uint256[26] calldata _pubSignals
  ) external view returns (bool);
}

contract Zringotts is MerkleTreeWithHistory, ReentrancyGuard {
  IVerifier public immutable verifier;
  MockToken public weth;
  MockToken public usdc;

  struct State {
    int256 weth_deposit_amount;
    int256 weth_borrow_amount;
    int256 usdc_deposit_amount;
    int256 usdc_borrow_amount;
  }
  State public state;

  struct Liquidated {
    uint256 liq_price;
    uint256 timestamp;
  }

  uint256 public constant LIQUIDATED_ARRAY_NUMBER = 10;
  Liquidated[] public liquidated_array;

  mapping(bytes32 => bool) public nullifierHashes;
  mapping(bytes32 => bool) public commitments;

  event Deposit(bytes32 nullifierHash, uint256 timestamp);
  event Borrow(address to, bytes32 nullifierHash, uint256 timestamp);
  event Repay(bytes32 nullifierHash, uint256 timestamp);
  event Withdraw(address to, bytes32 nullifierHash, uint256 timestamp);
  event Claim(address to, bytes32 nullifierHash, uint256 timestamp);
  event CommitmentAdded(bytes32 indexed commitment, uint32 indexed leafIndex);

  constructor(
    IVerifier _verifier,
    uint32 _merkleTreeHeight,
    MockToken _weth,
    MockToken _usdc
  ) MerkleTreeWithHistory(_merkleTreeHeight) {
    verifier = _verifier;
    for (uint256 i = 0; i < LIQUIDATED_ARRAY_NUMBER; i++) {
      liquidated_array.push(Liquidated({liq_price: i + 1, timestamp: 0}));
    }
    weth = _weth;
    usdc = _usdc;
  }

  modifier isWethOrUsdc(MockToken _token) {
    require(address(_token) == address(weth) || address(_token) == address(usdc), "Token must be weth or usdc");
    _;
  }

  function update_liquidated_array(uint8 index, uint256 _liq_price, uint256 _timestamp) public {
    require(index < LIQUIDATED_ARRAY_NUMBER, "Index exceeds number of possible liquidated position buckets");
    liquidated_array[index].liq_price = _liq_price;
    liquidated_array[index].timestamp = _timestamp;
  }

  function deposit(
    bytes32 _new_note_hash, bytes32, uint256 _new_timestamp, bytes32 _root,
    bytes32 _old_nullifier, uint256[2] calldata _pA, uint256[2][2] calldata _pB,
    uint256[2] calldata _pC, uint256 _lend_amt, MockToken _lend_token
  ) external payable nonReentrant isWethOrUsdc(_lend_token) {
    require(_new_timestamp > block.timestamp - 5 minutes, "Invalid timestamp");
    require(_new_timestamp <= block.timestamp, "Invalid timestamp");
    require(_lend_token.transferFrom(msg.sender, address(this), _lend_amt), "Token lend failed");
    uint256[26] memory public_inputs = constructPublicInputs(_new_note_hash, _root, 0, 0, _lend_amt, 0);
    require(verifier.verifyProof(_pA, _pB, _pC, public_inputs), "Invalid deposit proof");
    require(!commitments[_new_note_hash], "The commitment has been submitted");
    uint32 inserted_index = _insert(_new_note_hash);
    commitments[_new_note_hash] = true;
    if (_old_nullifier != bytes32(0)) {
      require(isKnownRoot(_root), "Cannot find your merkle root");
      require(!nullifierHashes[_old_nullifier], "The note has been already spent");
      nullifierHashes[_old_nullifier] = true;
    }
    if (address(_lend_token) == address(weth)) { state.weth_deposit_amount += int256(_lend_amt); }
    else { state.usdc_deposit_amount += int256(_lend_amt); }
    emit CommitmentAdded(_new_note_hash, inserted_index);
    emit Deposit(_old_nullifier, _new_timestamp);
  }

  function borrow(
    bytes32 _new_note_hash, bytes32, uint256 _new_timestamp, bytes32 _root,
    bytes32 _old_nullifier, uint256[2] calldata _pA, uint256[2][2] calldata _pB,
    uint256[2] calldata _pC, uint256 _borrow_amt, MockToken _borrow_token, address _to
  ) external payable nonReentrant isWethOrUsdc(_borrow_token) {
    require(_new_timestamp > block.timestamp - 5 minutes, "Invalid timestamp");
    require(_new_timestamp <= block.timestamp, "Invalid timestamp");
    _borrow_token.transfer(_to, _borrow_amt);
    uint256[26] memory public_inputs = constructPublicInputs(_new_note_hash, _root, 0, 0, 0, _borrow_amt);
    require(verifier.verifyProof(_pA, _pB, _pC, public_inputs), "Invalid borrow proof");
    require(!commitments[_new_note_hash], "The commitment has been submitted");
    uint32 inserted_index = _insert(_new_note_hash);
    commitments[_new_note_hash] = true;
    require(isKnownRoot(_root), "Cannot find your merkle root");
    require(_old_nullifier != bytes32(0), "Old nullifier must not be zero");
    require(!nullifierHashes[_old_nullifier], "The note has been already spent");
    nullifierHashes[_old_nullifier] = true;
    if (address(_borrow_token) == address(weth)) { state.weth_borrow_amount += int256(_borrow_amt); }
    else { state.usdc_borrow_amount += int256(_borrow_amt); }
    emit CommitmentAdded(_new_note_hash, inserted_index);
    emit Borrow(_to, _old_nullifier, _new_timestamp);
  }

  function repay(
    bytes32 _new_note_hash, bytes32, uint256 _new_timestamp, bytes32 _root,
    bytes32 _old_nullifier, uint256[2] calldata _pA, uint256[2][2] calldata _pB,
    uint256[2] calldata _pC, uint256 _repay_amt, MockToken _repay_token
  ) external payable nonReentrant isWethOrUsdc(_repay_token) {
    require(_new_timestamp > block.timestamp - 5 minutes, "Invalid timestamp");
    require(_new_timestamp <= block.timestamp, "Invalid timestamp");
    _repay_token.transferFrom(msg.sender, address(this), _repay_amt);
    uint256[26] memory public_inputs = constructPublicInputs(_new_note_hash, _root, 0, _repay_amt, 0, 0);
    require(verifier.verifyProof(_pA, _pB, _pC, public_inputs), "Invalid repay proof");
    require(!commitments[_new_note_hash], "The commitment has been submitted");
    uint32 inserted_index = _insert(_new_note_hash);
    commitments[_new_note_hash] = true;
    require(isKnownRoot(_root), "Cannot find your merkle root");
    require(_old_nullifier != bytes32(0), "Old nullifier must not be zero");
    require(!nullifierHashes[_old_nullifier], "The note has been already spent");
    nullifierHashes[_old_nullifier] = true;
    if (address(_repay_token) == address(weth)) { state.weth_borrow_amount -= int256(_repay_amt); }
    else { state.usdc_borrow_amount -= int256(_repay_amt); }
    emit CommitmentAdded(_new_note_hash, inserted_index);
    emit Repay(_old_nullifier, _new_timestamp);
  }

  function withdraw(
    bytes32 _new_note_hash, bytes32, uint256 _new_timestamp, bytes32 _root,
    bytes32 _old_nullifier, uint256[2] calldata _pA, uint256[2][2] calldata _pB,
    uint256[2] calldata _pC, uint256 _withdraw_amt, MockToken _withdraw_token, address _to
  ) external payable nonReentrant isWethOrUsdc(_withdraw_token) {
    require(_new_timestamp > block.timestamp - 5 minutes, "Invalid timestamp");
    require(_new_timestamp <= block.timestamp, "Invalid timestamp");
    _withdraw_token.transfer(_to, _withdraw_amt);
    uint256[26] memory public_inputs = constructPublicInputs(_new_note_hash, _root, _withdraw_amt, 0, 0, 0);
    require(verifier.verifyProof(_pA, _pB, _pC, public_inputs), "Invalid withdraw proof");
    require(!commitments[_new_note_hash], "The commitment has been submitted");
    uint32 inserted_index = _insert(_new_note_hash);
    commitments[_new_note_hash] = true;
    require(isKnownRoot(_root), "Cannot find your merkle root");
    require(_old_nullifier != bytes32(0), "Old nullifier must not be zero");
    require(!nullifierHashes[_old_nullifier], "The note has been already spent");
    nullifierHashes[_old_nullifier] = true;
    if (address(_withdraw_token) == address(weth)) { state.weth_deposit_amount -= int256(_withdraw_amt); }
    else { state.usdc_deposit_amount -= int256(_withdraw_amt); }
    emit CommitmentAdded(_new_note_hash, inserted_index);
    emit Withdraw(_to, _old_nullifier, _new_timestamp);
  }

  function constructPublicInputs(
    bytes32 _new_note_hash, bytes32 _root,
    uint256 _lend_token_out, uint256 _borrow_token_out,
    uint256 _lend_token_in, uint256 _borrow_token_in
  ) public view returns (uint256[26] memory) {
    uint256[26] memory public_inputs;
    public_inputs[0] = uint256(_new_note_hash);
    public_inputs[1] = uint256(_root);
    for (uint256 i = 0; i < 10; i++) { public_inputs[2 + i] = liquidated_array[i].liq_price; }
    for (uint256 i = 0; i < 10; i++) { public_inputs[12 + i] = liquidated_array[i].timestamp; }
    public_inputs[22] = _lend_token_out;
    public_inputs[23] = _borrow_token_out;
    public_inputs[24] = _lend_token_in;
    public_inputs[25] = _borrow_token_in;
    return public_inputs;
  }
}
"#;
