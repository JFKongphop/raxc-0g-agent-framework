// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title RaxcAuditTask8183
/// @notice ERC-8183 autonomous audit task lifecycle for RAXCLAW
/// @dev Minimal implementation: createAuditTask + finalizeAuditTask + verifyTask only.
///      No marketplace, no bidding, no tokenomics — pure infrastructure.
contract RaxcAuditTask8183 {

  enum TaskState { Created, Completed }

  struct AuditTask {
    address requester;
    string contractName;
    TaskState state;
    // Proof fields — populated on finalization
    string verdict;
    uint256 confidence;  // basis points: 7750 = 77.50%
    bytes32 rootHash;    // 0G Storage merkle root hash of audit report
    string replayId;     // deterministic replay identifier (RAXC Replay Engine)
    bytes32 traceHash;   // cryptographic execution trace hash
    uint256 createdAt;
    uint256 completedAt;
  }

  mapping(uint256 => AuditTask) private _tasks;
  uint256 public taskCount;

  event AuditTaskCreated(
    uint256 indexed taskId,
    address indexed requester,
    string contractName,
    uint256 timestamp
  );

  event AuditTaskCompleted(
    uint256 indexed taskId,
    string verdict,
    bytes32 indexed rootHash,
    string replayId,
    uint256 timestamp
  );

  /// @notice Submit a new audit task on-chain
  /// @param contractName Name of the contract being audited
  /// @return taskId The assigned task ID (monotonically increasing)
  function createAuditTask(string calldata contractName) external returns (uint256 taskId) {
    taskId = taskCount++;
    _tasks[taskId] = AuditTask({
      requester: msg.sender,
      contractName: contractName,
      state: TaskState.Created,
      verdict: "",
      confidence: 0,
      rootHash: bytes32(0),
      replayId: "",
      traceHash: bytes32(0),
      createdAt: block.timestamp,
      completedAt: 0
    });
    emit AuditTaskCreated(taskId, msg.sender, contractName, block.timestamp);
  }

  /// @notice Finalize an audit task with cryptographic proof
  /// @param taskId        Task ID returned by createAuditTask
  /// @param verdict       Risk verdict (e.g. "HIGH_RISK", "MEDIUM_RISK")
  /// @param confidence    Confidence in basis points (7750 = 77.50%)
  /// @param rootHash      0G Storage merkle root hash of the full audit report
  /// @param replayId      Deterministic replay ID from RAXC Replay Engine
  /// @param traceHash     Cryptographic execution trace hash
  function finalizeAuditTask(
    uint256 taskId,
    string calldata verdict,
    uint256 confidence,
    bytes32 rootHash,
    string calldata replayId,
    bytes32 traceHash
  ) external {
    AuditTask storage task = _tasks[taskId];
    require(task.createdAt != 0, "Task does not exist");
    require(task.state == TaskState.Created, "Task already finalized");

    task.state = TaskState.Completed;
    task.verdict = verdict;
    task.confidence = confidence;
    task.rootHash = rootHash;
    task.replayId = replayId;
    task.traceHash = traceHash;
    task.completedAt = block.timestamp;

    emit AuditTaskCompleted(taskId, verdict, rootHash, replayId, block.timestamp);
  }

  /// @notice Verify that an audit task has valid proof attached
  /// @param taskId Task ID to verify
  /// @return valid True if task is completed with a non-zero root hash
  function verifyTask(uint256 taskId) external view returns (bool valid) {
    AuditTask storage task = _tasks[taskId];
    return task.state == TaskState.Completed && task.rootHash != bytes32(0);
  }

  /// @notice Get full task details
  function getTask(uint256 taskId) external view returns (AuditTask memory) {
    require(_tasks[taskId].createdAt != 0, "Task does not exist");
    return _tasks[taskId];
  }
}
